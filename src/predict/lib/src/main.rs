use fst::{Map, Set, IntoStreamer};
use fst::automaton::{Automaton, Str, Levenshtein};
use lazy_static::lazy_static;

struct Predictor {
    dictionary: Set<Vec<u8>>,
    shortcode_dictionary: Map<Vec<u8>>,
    symbols: Vec<String>
}

impl Predictor {
    // fn word(&self, context: &str) {
    //
    // }

    fn symbol(&self, context: &str) -> Result<Vec<(String, &String)>,  fst::Error> {
        let context_matcher = Str::new(context).starts_with();
        let search_results  = self.shortcode_dictionary.search(context_matcher)
            .into_stream()
            .into_str_vec()?;

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
    let symbol_results = PREDICTOR.symbol("ang").unwrap();
    for (shortcode, symbol) in symbol_results {
        println!("{shortcode} : {symbol}", shortcode=shortcode, symbol=symbol);
    }

}