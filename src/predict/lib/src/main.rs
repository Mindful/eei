use fst::{Map, Set};


struct Predictor {
    dictionary: Set<Vec<u8>>,
    shortcode_dictionary: Map<Vec<u8>>,
    symbols: Vec<String>
}



fn main() {
    println!("lib main");
}