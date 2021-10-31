extern crate cbindgen;

use std::env;static GENERATED_HEADER: &str = "../../predict.h";

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(GENERATED_HEADER);
}