use fst::{Map, IntoStreamer};
use fst::automaton::{Automaton, Str};
use lazy_static::lazy_static;
use crate::predict::PredictionError::*;
use std::str::Utf8Error;
use std::fmt;

pub struct Predictor {
    dictionary: Map<Vec<u8>>,
    shortcode_dictionary: Map<Vec<u8>>,
    symbols: Vec<String>
}

#[derive(Debug)]
pub enum PredictionError {
    FstError(fst::Error),
    LevenshteinError(fst::automaton::LevenshteinError),
    MissingSymbol(String, u64),
    FailedStringConversion(Utf8Error)
}

impl fmt::Display for PredictionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FstError(err) => write!(f, "FST error: {}", err),
            LevenshteinError(err) => write!(f, "Levenshtein error: {}", err),
            MissingSymbol(sym, codepoint) => write!(f, "Missing shortcode: {}, for codepoint {}", sym, codepoint),
            FailedStringConversion(err) => write!(f, "String conversion error: {}", err)
        }
    }
}


impl Predictor {
    const returned_word_count: usize = 25;

    fn is_title_cased(context: &str) -> bool {
        let mut chars = context.chars();

        let first_letter_capitalized = chars.next().map(|char| char.is_ascii_uppercase()).unwrap_or(false);
        let other_capitals: i8 = chars.map(|char| char.is_ascii_uppercase() as i8).sum();
        first_letter_capitalized & (other_capitals == 0)
    }

    fn title_case(word: String) -> String {
        let mut chars = word.chars();
        chars.next().unwrap().to_ascii_uppercase().to_string() + chars.as_str()
    }

    pub fn word(&self, context: &str) -> Result<Vec<String>,  PredictionError>  {
        let title_cased = Predictor::is_title_cased(context);
        let lowercase_context = context.to_ascii_lowercase();
        let matcher = Str::new(lowercase_context.as_str()).starts_with();

        let mut search_results = self.dictionary.search(matcher)
            .into_stream()
            .into_str_vec().map_err(FstError)?;

        search_results.sort_by(|(_w1, f1), (_w2, f2)| f2.cmp(f1));
        let final_results = search_results
            .into_iter()
            .map(|(word, _freq)| {
                if title_cased {
                    Predictor::title_case(word)
                } else {
                    word
                }
            })
            .take(Predictor::returned_word_count).collect();
        Ok(final_results)
    }

    pub fn symbol(&self, context: &str) -> Result<Vec<(String, String)>,  PredictionError> {
        let matcher = Str::new(context).starts_with();
        let search_results  = self.shortcode_dictionary.search(matcher)
            .into_stream()
            .into_str_vec().map_err(FstError)?;

        //must be into_iter() and not iter() - the latter iterates over references, but we need
        //to take ownership to return the shortcode data without clone()
        Ok(search_results.into_iter().map(|(shortcode, ident)| {
            match self.symbols.get(ident as usize) {
                Some(symbol) => Ok((shortcode, symbol.clone())),
                None => Err(MissingSymbol(shortcode, ident))
            }
        }).collect::<Result<Vec<_>, _>>()?)
    }
}



lazy_static! {
    pub static ref PREDICTOR: Predictor = Predictor {
        dictionary: Map::new(include_bytes!("../../dictionary.fst").to_vec()).unwrap(),
        shortcode_dictionary: Map::new(include_bytes!("../../shortcodes.fst").to_vec()).unwrap(),
        symbols: bincode::deserialize(include_bytes!("../../symbols.bin")).unwrap()
    };
}
