mod predict;

#[no_mangle]
pub extern "C" fn rust_function() {
    println!("called rust function");
}
