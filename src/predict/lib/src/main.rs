use fst::{Map, Set, IntoStreamer};
use fst::automaton::{Automaton, Str, Levenshtein};
use lazy_static::lazy_static;
use crate::PredictionError::*;

struct Predictor {
    dictionary: Set<Vec<u8>>,
    shortcode_dictionary: Map<Vec<u8>>,
    symbols: Vec<String>
}

#[derive(Debug)]
enum PredictionError {
    FstError(fst::Error),
    LevenshteinError(fst::automaton::LevenshteinError)
}


impl Predictor {
    fn word(&self, context: &str) -> Result<Vec<String>,  PredictionError>  {
        let acceptable_edit_distance: u32 = (context.len() as f64).log2().floor() as u32;
        let matcher = Str::new(context).starts_with()
            .union(Levenshtein::new(context, acceptable_edit_distance).map_err(LevenshteinError)?);

        Ok(self.dictionary.search(matcher)
            .into_stream()
            .into_strs().map_err(FstError)?)
    }

    fn symbol(&self, context: &str) -> Result<Vec<(String, &String)>,  PredictionError> {
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
    static ref PREDICTOR: Predictor = Predictor {
        dictionary: Set::new(include_bytes!("../../dictionary.fst").to_vec()).unwrap(),
        shortcode_dictionary: Map::new(include_bytes!("../../shortcodes.fst").to_vec()).unwrap(),
        symbols: bincode::deserialize(include_bytes!("../../symbols.bin")).unwrap()
    };
}





fn main() {
    let word_pref = "lit";

    let symbol_results = PREDICTOR.symbol("ang").unwrap();
    let word_results = PREDICTOR.word(word_pref).unwrap();
    for (shortcode, symbol) in symbol_results {
        println!("{shortcode} : {symbol}", shortcode=shortcode, symbol=symbol);
    }
    println!("words for {pref}:", pref=word_pref);
    for word in word_results {
        println!("{word}", word=word);
    }

}