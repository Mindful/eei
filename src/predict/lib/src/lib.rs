mod predict;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::mem;
use std::io::Write;
use chrono::Local;
use env_logger::Builder;

use predict::PREDICTOR;
use crate::predict::PredictionError::{FailedStringConversion, FstError, LevenshteinError, MissingSymbol};
use crate::predict::PredictionError;

impl PredictionError {
    fn error_message(&self) -> String {
        match self {
            FstError(err) => format!("FST error: {}", err),
            LevenshteinError(err) => format!("Levenshtein error: {}", err),
            MissingSymbol(sym, codepoint) => format!("Missing shortcode: {}, for codepoint {}", sym, codepoint),
            FailedStringConversion(err) => format!("String conversion error: {}", err)
        }
    }
}

#[repr(C)]
pub struct WordPredictions {
    len: c_int,
    words: *mut *mut c_char
}

#[repr(C)]
pub struct SymbolPredictions {
    len: c_int,
    symbols: *mut *mut c_char,
    shortcodes: *mut *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn get_word_predictions(characters: *mut c_char) -> WordPredictions {
    CString::from_raw(characters)
        .into_string()
        .map_err(FailedStringConversion)
        .and_then(|cstring| {
            PREDICTOR.word(cstring.as_str())
        }).map(|word_predictions| {
            WordPredictions {
                len: word_predictions.len() as c_int,
                words: convert_string_vector(word_predictions)
            }
        }).unwrap_or_else(|err| {
            let err_msg = err.error_message();
            println!("{}", err_msg);

            WordPredictions {
                len: 1,
                words: convert_string_vector(vec![err_msg])
            }
        })
}

#[no_mangle]
pub unsafe extern "C" fn free_word_predictions(predictions: WordPredictions) {
    free_string_array(predictions.words, predictions.len);
}


//based on
////https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=d0e44ce1f765ce89523ef89ccd864e54
fn convert_string_vector(str_vec: Vec<String>) -> *mut *mut c_char {
    str_vec.into_iter().map(|s| {
        CString::new(s.into_bytes()).map(|cstr| cstr.into_raw())
    }).collect::<Result<Vec<_>, _>>().map(|mut cstring_vec| {
        let ptr = cstring_vec.as_mut_ptr();
        mem::forget(cstring_vec);
        ptr
    }).unwrap_or(std::ptr::null_mut())
}

unsafe fn free_string_array(ptr: *mut *mut c_char, len: c_int) {
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



