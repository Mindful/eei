mod predict;

use predict::PREDICTOR;

fn symbol_test(head: &str) {
    let symbol_results = PREDICTOR.symbol(head).unwrap();
    println!("symbols for {head}", head=head);
    for (shortcode, symbol) in symbol_results {
        println!("{shortcode} : {symbol}", shortcode=shortcode, symbol=symbol);
    }
}

fn word_test(head: &str) {
    let word_results = PREDICTOR.word(head).unwrap();

    println!("words for {head}:", head=head);
    for word in word_results {
        println!("{word}", word=word);
    }
}

fn main() {
    symbol_test("eq");
    symbol_test("u");
    word_test("lit");
    word_test("ang");
}