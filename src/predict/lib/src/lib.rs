mod predict;
#[allow(warnings)]
mod ibus;

use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};
use std::mem;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

use crate::predict::PredictionError::{FailedStringConversion, FstError, LevenshteinError, MissingSymbol};
use crate::predict::PredictionError;
use crate::predict::PREDICTOR;
use crate::ibus::{ibus_engine_hide_lookup_table, IBusEngine};

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

/*
static gboolean
			ibus_eei_engine_process_key_event
                                            (IBusEngine             *engine,
                                             guint               	 keyval,
                                             guint               	 keycode,
                                             guint               	 modifiers);
 */

// #[no_mangle]
// pub unsafe extern "C" fn ribus_eei_engine_process_key_event(
//     engine: *mut ibus::IBusEngine,
//     keyval: ibus::guint,
//     keycode: ibus::guint,
//     modifiers: ibus::guint,
// ) {
//
// }

/*

static void
ibus_eei_engine_hide_lookup_table(IBusEEIEngine *eei) {
    ibus_engine_hide_lookup_table ((IBusEngine *) eei);
    eei->lookup_table_visible = FALSE;
}
 */

#[no_mangle]
pub unsafe extern "C" fn ibus_eei_engine_hide_lookup_table(engine: *mut ibus::IBusEEIEngine) {
    ibus_engine_hide_lookup_table(engine as *mut IBusEngine);
    (*engine).lookup_table_visible = 0;
}

#[no_mangle]
pub unsafe extern "C" fn configure_logging() {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
        .build("/home/josh/scrapbox/eei.log").unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info)).unwrap();

    log4rs::init_config(config).unwrap();

    log::info!("Logging initialized");
}

#[no_mangle]
pub unsafe extern "C" fn get_word_predictions(characters: *mut c_char) -> WordPredictions {
    CStr::from_ptr(characters)
        .to_str()
        .map_err(FailedStringConversion)
        .and_then(|cstring| {
            PREDICTOR.word(cstring)
        }).map(|word_predictions| {
            WordPredictions {
                len: word_predictions.len() as c_int,
                words: convert_string_vector(word_predictions)
            }
        }).unwrap_or_else(|err| {
            let err_msg = err.error_message();
            log::error!("{}", err_msg);

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
    }).unwrap_or_else(|_| {
        log::error!("Failed to convert string vector");
        std::ptr::null_mut()
    })
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




