mod predict;

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
use ibus::*;

pub struct EngineState {
    lookup_visible: bool,
    emoji_input: bool,
    preedit: String,
    cursor_pos: u32
}

#[no_mangle]
pub unsafe extern "C" fn new_engine_state() -> *mut EngineState {
    Box::into_raw(Box::new(EngineState {
        lookup_visible: false,
        emoji_input: false,
        preedit: "".to_string(),
        cursor_pos: 0
    }))
}

#[no_mangle]
pub unsafe extern "C" fn free_engine_state(engine_state: *mut EngineState) {
    std::mem::drop(Box::from_raw(engine_state));
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
pub unsafe extern "C" fn ribus_eei_engine_process_key_event(engine: *mut IBusEngine, keyval: guint,
    keycode: guint, modifiers: guint, ) -> gboolean {
    if (modifiers & IBusModifierType_IBUS_RELEASE_MASK) != 0 {
        return GBOOL_FALSE;
    }

    if (modifiers & IBusModifierType_IBUS_CONTROL_MASK) == IBusModifierType_IBUS_CONTROL_MASK
        && keyval == IBUS_s {
        //TODO: turn emoji input mode on
        return GBOOL_TRUE;
    }

    match keyval {
        //TODO: handle other keyvals
        IBUS_space => {
            //TODO: reset word buffer, exit emoji mode
            GBOOL_TRUE
        }
        IBUS_Return => {
            //TODO: if we are selecting words or in emoji mode, commit our current selection
            //otherwise reset word buffer
            GBOOL_TRUE
        }
        IBUS_space..=IBUS_asciitilde => {
            //TODO: add this char to the buffer or emoji editing
            GBOOL_TRUE
        }
        _ => GBOOL_FALSE
    }
}

#[no_mangle]
pub unsafe extern "C" fn ibus_eei_engine_hide_lookup_table(engine: *mut IBusEEIEngine) {
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




