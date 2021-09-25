mod predict;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::{ptr, mem};
use std::mem::ManuallyDrop;

use predict::PREDICTOR;


#[repr(C)]
struct WordPredictions {
    len: c_int,
    words: *mut *mut c_char
}

#[repr(C)]
struct SymbolPredictions {
    len: c_int,
    symbols: *mut *mut c_char,
    shortcodes: *mut *mut c_char
}

#[no_mangle]
pub extern "C" fn rust_function() -> * mut WordPredictions {
    println!("called rust function");
    let mut x = Vec::new();
    let y = WordPredictions {
        len: 0,
        words: x.as_mut_ptr()
    };
    let res = *y;
    mem::forget(y);
    res
}


#[no_mangle]
pub unsafe extern "C" fn get_word_predictions(characters: *mut c_char) -> * mut WordPredictions {
    //TODO: error handling
    let context = CString::from_raw(characters).into_string().unwrap();
    let word_predictions = PREDICTOR.word(context.as_str()).unwrap();

    //TODO: return actual data
    let res = WordPredictions {
        len: word_predictions.len() as c_int,
        words: Vec::new().as_mut_ptr()
    };
    let r = *res;
    mem::forget(res);
    r
}




//https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=d0e44ce1f765ce89523ef89ccd864e54
#[no_mangle]
unsafe extern "C" fn get_strings(length_out: *mut c_int) -> *mut *mut c_char {
    let mut v = vec![];

    // Let's fill a vector with null-terminated strings
    v.push(CString::new("Hello").unwrap());
    v.push(CString::new("World").unwrap());
    v.push(CString::new("!").unwrap());

    // Turning each null-terminated string into a pointer.
    // `into_raw` takes ownershop, gives us the pointer and does NOT drop the data.
    let mut out = v
        .into_iter()
        .map(|s| s.into_raw())
        .collect::<Vec<_>>();

    // Make sure we're not wasting space.
    out.shrink_to_fit();
    assert!(out.len() == out.capacity());

    // Get the pointer to our vector.
    let len = out.len();
    let ptr = out.as_mut_ptr();
    mem::forget(out);

    // Let's write back the length the caller can expect
    ptr::write(length_out, len as c_int);

    // Finally return the data
    ptr
}

#[no_mangle]
unsafe extern "C" fn free_string_array(ptr: *mut *mut c_char, len: c_int) {
    let len = len as usize;

    // Get back our vector.
    // Previously we shrank to fit, so capacity == length.
    let v = Vec::from_raw_parts(ptr, len, len);

    // Now drop one string at a time.
    for elem in v {
        let s = CString::from_raw(elem);
        mem::drop(s);
    }

    // Afterwards the vector will be dropped and thus freed.
}



