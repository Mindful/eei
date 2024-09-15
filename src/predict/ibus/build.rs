use std::path::Path;

static ENGINE_HEADER: &str = "../../../include/eei/engine.h";
static BINDING_FILE: &str = "src/ibus_bindings.rs";

fn main() {
    let ibus_lib = pkg_config::Config::new().atleast_version("1.5.0").probe("ibus-1.0").unwrap();


    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed={}", ENGINE_HEADER);

    let bindings = ibus_lib.include_paths.iter().fold(
        bindgen::Builder::default().header(ENGINE_HEADER),
        | b, path| {
            b.clang_arg("-I".to_string()+path.to_str().unwrap())
        }
    )
        .allowlist_function("ibus_.*")
        .allowlist_type("IBus.*")
        .allowlist_var("(IBUS_.*|GBOOL_.*)")
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
