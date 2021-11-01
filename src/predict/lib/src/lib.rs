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
use ibus::{IBusEEIEngine, gboolean, GBOOL_FALSE, ibus_engine_update_lookup_table, IBusEngine, GBOOL_TRUE, ibus_engine_hide_lookup_table, guint, IBusModifierType_IBUS_RELEASE_MASK, IBusModifierType_IBUS_CONTROL_MASK, IBUS_s, IBUS_asciitilde, IBUS_space, IBUS_Return, ibus_engine_commit_text, ibus_text_new_from_unichar, ibus_text_new_from_string, gchar};

pub struct EngineCore {
    lookup_visible: bool,
    word_buffer: String,
    cursor_pos: i32,
    symbol_input: bool,
    symbol_preedit: String,
    preedit_cursor_pos: i32,
    parent_engine: *mut IBusEEIEngine
}


impl EngineCore {
    fn symbol_input_enable(&mut self) -> gboolean {
        if self.symbol_input {
            return GBOOL_FALSE;
        }

        self.symbol_input = true;
        unsafe {
            // ibus_lookup_table_clear((*self.parent_engine).table);
            // ibus_engine_show_lookup_table(engine as *mut IBusEngine);
            ibus_engine_update_lookup_table(self.parent_engine as *mut IBusEngine, (*self.parent_engine).table, GBOOL_TRUE);
        }
        GBOOL_TRUE
    }

    fn symbol_input_disable(&mut self) -> gboolean {
        if !self.symbol_input {
            return GBOOL_FALSE;
        }

        self.symbol_input = false;
        self.symbol_preedit.clear();
        unsafe {
            ibus_engine_hide_lookup_table(self.parent_engine as *mut IBusEngine)
        }
        GBOOL_TRUE
    }

    fn symbol_input_char(&mut self, character: char) -> gboolean {
        if !self.symbol_input {
            log::error!("Symbol input char called outside symbol input mode");
            return GBOOL_FALSE;
        }

        //TODO: append char, display preedit, run search and update aux text/search results

        GBOOL_TRUE
    }

    fn symbol_input_commit(&mut self) -> gboolean {
        if !self.symbol_input {
            log::error!("Symbol input commit called outside symbol input mode");
            return GBOOL_FALSE;
        }

        //TODO: this impl is wrong - don't commit the preedit string, get the current index of selection
        //and then commit the symbol at that index

        let converted_text = CString::new(self.symbol_preedit.as_bytes()).map(|cstr| cstr.into_raw() as *const gchar);
        let result = match converted_text {
            Ok(gchar_pointer) => unsafe {
                ibus_engine_commit_text(self.parent_engine as *mut IBusEngine, ibus_text_new_from_string(gchar_pointer));
                GBOOL_TRUE
            }
            Err(error) => {
                log::error!("Error comitting symbol: {}", error);
                GBOOL_FALSE
            }
        };

        self.symbol_input_disable();
        result
    }
}

#[no_mangle]
pub unsafe extern "C" fn new_engine_core(parent_engine: *mut IBusEEIEngine) -> *mut EngineCore {
    Box::into_raw(Box::new(EngineCore {
        lookup_visible: false,
        word_buffer: String::new(),
        cursor_pos: 0,
        symbol_input: false,
        symbol_preedit: String::new(),
        preedit_cursor_pos: 0,
        parent_engine: parent_engine
    }))
}

#[no_mangle]
pub unsafe extern "C" fn free_engine_core(engine_state: *mut EngineCore) {
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

unsafe fn get_engine_core(engine: *mut IBusEngine) -> Option<&'static mut EngineCore> {
    ((*(engine as *mut IBusEEIEngine)).engine_core as *mut EngineCore).as_mut()
}


#[no_mangle]
pub unsafe extern "C" fn ibus_eei_engine_process_key_event(engine: *mut IBusEngine, keyval: guint,
    keycode: guint, modifiers: guint, ) -> gboolean {

    let engine_core = match get_engine_core(engine) {
        Some(engine_ref) => engine_ref,
        None => {
            log::error!("Could not retrieve engine core");
            return GBOOL_FALSE
        }
    };


    if (modifiers & IBusModifierType_IBUS_RELEASE_MASK) != 0 {
        return GBOOL_FALSE;
    }

    if (modifiers & IBusModifierType_IBUS_CONTROL_MASK) == IBusModifierType_IBUS_CONTROL_MASK
        && keyval == IBUS_s {
        return engine_core.symbol_input_enable();
    }

    match keyval {
        //TODO: handle other keyvals
        IBUS_space => {
            if engine_core.symbol_input {
                return engine_core.symbol_input_disable();
            }
            //TODO: reset word buffer in normal typing
            GBOOL_TRUE
        }
        IBUS_Return => {
            if engine_core.symbol_input {
                //TODO: commit symbol input
            }
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




