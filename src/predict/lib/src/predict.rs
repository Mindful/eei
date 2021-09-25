use fst::{Map, IntoStreamer};
use fst::automaton::{Automaton, Str};
use lazy_static::lazy_static;
use crate::predict::PredictionError::*;

pub struct Predictor {
    dictionary: Map<Vec<u8>>,
    shortcode_dictionary: Map<Vec<u8>>,
    symbols: Vec<String>
}

#[derive(Debug)]
pub enum PredictionError {
    FstError(fst::Error),
    LevenshteinError(fst::automaton::LevenshteinError)
}


impl Predictor {
    pub fn word(&self, context: &str) -> Result<Vec<String>,  PredictionError>  {
        let matcher = Str::new(context).starts_with();

        let mut search_results = self.dictionary.search(matcher)
            .into_stream()
            .into_str_vec().map_err(FstError)?;

        search_results.sort_by(|(_w1, f1), (_w2, f2)| f2.cmp(f1));
        Ok(search_results.into_iter().map(|(word, _freq)| {word}).take(10).collect())
    }

    pub fn symbol(&self, context: &str) -> Result<Vec<(String, &String)>,  PredictionError> {
        let matcher = Str::new(context).starts_with();
        let search_results  = self.shortcode_dictionary.search(matcher)
            .into_stream()
            .into_str_vec().map_err(FstError)?;

        //must be into_iter() and not iter() - the latter iterates over references, but we need
        //to take ownership to return the data without clone()
        Ok(search_results.into_iter().map(|(shortcode, ident)| {
            (shortcode, self.symbols.get(ident as usize).unwrap())
        }).collect())
    }
}


lazy_static! {
    pub static ref PREDICTOR: Predictor = Predictor {
        dictionary: Map::new(include_bytes!("../../dictionary.fst").to_vec()).unwrap(),
        shortcode_dictionary: Map::new(include_bytes!("../../shortcodes.fst").to_vec()).unwrap(),
        symbols: bincode::deserialize(include_bytes!("../../symbols.bin")).unwrap()
    };
}
