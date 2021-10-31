extern crate cbindgen;

use std::env;
use std::path::Path;

static ENGINE_HEADER: &str = "../../engine.h";
static BINDING_FILE: &str = "src/ibus.rs";
static GENERATED_HEADER: &str = "../../predict.h";

fn generate_ibus_bindings() {
    let ibus_lib = pkg_config::Config::new().atleast_version("1.5.0").probe("ibus-1.0").unwrap();

    let bindings = ibus_lib.include_paths.iter().fold(
        bindgen::Builder::default().header(ENGINE_HEADER),
        | b, path| {
                b.clang_arg("-I".to_string()+path.to_str().unwrap())
            }
        )
        .allowlist_function("ibus_engine_.*")
        .allowlist_type("IBusEEIEngine.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings.write_to_file(Path::new(BINDING_FILE))
        .expect("Couldn't write bindings!");
}

fn main() {
    if !Path::new(BINDING_FILE).exists() {
        generate_ibus_bindings();
    }
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(GENERATED_HEADER);
}