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

    fn symbol(&self, context: &str) {
        let context_matcher = Str::new(context).starts_with();
        self.shortcode_dictionary.search(context_matcher)
            .into_stream();
    }
}


lazy_static! {
    static PREDICTOR: Predictor = Predictor {
        dictionary: Set::new(include_bytes!("../../dictionary.fst").to_vec()).unwrap(),
        shortcode_dictionary: Map::new(include_bytes!("../../shortcodes.fst").to_vec()).unwrap(),
        symbols: bincode::deserialize(include_bytes!("../../symbols.bin")).unwrap()
    };
}





fn main() {
    // File written from a build script using MapBuilder.
    static FST: &[u8] = include_bytes!("../../dictionary.fst");

    let map : Map<&[u8]> = Map::new(FST).unwrap();

    println!("lib main");
}