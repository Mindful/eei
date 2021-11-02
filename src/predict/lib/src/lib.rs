mod predict;

use std::ffi::{CString, CStr, NulError};
use std::os::raw::{c_char, c_int};
use std::mem;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

use crate::predict::PredictionError::{FailedStringConversion, FstError, LevenshteinError, MissingSymbol};
use crate::predict::PredictionError;
use crate::predict::PREDICTOR;
use ibus::{IBusEEIEngine, gboolean, GBOOL_FALSE, ibus_engine_update_lookup_table, IBusEngine, GBOOL_TRUE, ibus_engine_hide_lookup_table, guint, IBusModifierType_IBUS_RELEASE_MASK, IBusModifierType_IBUS_CONTROL_MASK, IBUS_s, IBUS_asciitilde, IBUS_space, IBUS_Return, ibus_engine_commit_text, ibus_text_new_from_unichar, ibus_text_new_from_string, gchar, ibus_lookup_table_clear, ibus_lookup_table_append_candidate, IBusText, ibus_lookup_table_append_label, ibus_engine_update_auxiliary_text, IBUS_Up, IBUS_Down, IBUS_Left, IBUS_Right, ibus_lookup_table_get_cursor_pos, IBusLookupTable, ibus_lookup_table_get_label, ibus_lookup_table_cursor_up, ibus_lookup_table_cursor_down};


pub struct EngineCore {
    lookup_visible: bool,
    word_buffer: String,
    cursor_pos: i32,
    symbol_input: bool,
    symbol_preedit: String,
    parent_engine: *mut IBusEEIEngine
}

unsafe fn into_ibus_string(input: String) -> Result<*mut IBusText, NulError> {
    CString::new(input.into_bytes()).map(|cstr| ibus_text_new_from_string(cstr.into_raw() as *const gchar))
}

impl EngineCore {
    fn parent_engine_as_ibus_engine(&self) -> *mut IBusEngine {
        self.parent_engine as *mut IBusEngine
    }

    unsafe fn get_table(&self) -> *mut IBusLookupTable {
        (*self.parent_engine).table
    }

    unsafe fn get(engine: *mut IBusEngine) -> Option<&'static mut EngineCore> {
        ((*(engine as *mut IBusEEIEngine)).engine_core as *mut EngineCore).as_mut()
    }



    unsafe fn symbol_input_enable(&mut self) -> gboolean {
        if self.symbol_input {
            return GBOOL_FALSE;
        }

        self.symbol_input = true;
        // ibus_lookup_table_clear((*self.parent_engine).table);
        // ibus_engine_show_lookup_table(engine as *mut IBusEngine);
        ibus_engine_update_lookup_table(self.parent_engine as *mut IBusEngine, (*self.parent_engine).table, GBOOL_TRUE);
        GBOOL_TRUE
    }

    unsafe fn symbol_input_disable(&mut self) -> gboolean {
        if !self.symbol_input {
            return GBOOL_FALSE;
        }

        self.symbol_input = false;
        self.symbol_preedit.clear();
        ibus_engine_hide_lookup_table(self.parent_engine as *mut IBusEngine);
        GBOOL_TRUE
    }

    unsafe fn commit_char(&mut self, keyval: guint) -> gboolean {
        self.word_buffer.push((keyval as u8) as char);
        ibus_engine_commit_text(self.parent_engine_as_ibus_engine(), ibus_text_new_from_string(&(keyval as gchar)));
        GBOOL_TRUE
    }

    unsafe fn symbol_input_char(&mut self, keyval: guint) -> gboolean {
        if !self.symbol_input {
            log::error!("Symbol input char called outside symbol input mode");
            return GBOOL_FALSE;
        }

        self.symbol_preedit.push((keyval as u8) as char);
        match into_ibus_string(self.symbol_preedit.clone()) {
            Ok(ibus_string) => {
                ibus_engine_update_auxiliary_text(self.parent_engine_as_ibus_engine(), ibus_string, GBOOL_TRUE);
            }
            Err(err) => {
                log::error!("Failed string conversion for symbol aux text update");
            }
        }


        let search_result  = PREDICTOR.symbol(self.symbol_preedit.as_str());
        match search_result {
            Ok(candidates) => {
                let table = self.get_table();
                ibus_lookup_table_clear(table);
                for (idx, (ident, shortcode)) in candidates.into_iter().enumerate() {
                    match (into_ibus_string(ident), into_ibus_string(shortcode)) {
                        (Ok(ident_ibus_string), Ok(shortcode_ibus_string)) => {
                            ibus_lookup_table_append_candidate(table, shortcode_ibus_string);
                            ibus_lookup_table_append_label(table, ident_ibus_string);
                        }
                        _ => {
                            log::error!("Failed string conversion for symbol lookup");
                        }
                    }
                }
                ibus_engine_update_lookup_table(self.parent_engine_as_ibus_engine(), table, GBOOL_TRUE);
            },
            Err(err) => {
                log::error!("{}", err);
                return GBOOL_FALSE;
            }
        }

        GBOOL_TRUE
    }

    unsafe fn symbol_input_commit(&mut self) -> gboolean {
        if !self.symbol_input {
            log::error!("Symbol input commit called outside symbol input mode");
            return GBOOL_FALSE;
        }

        let idx = ibus_lookup_table_get_cursor_pos(self.get_table());
        let symbol = ibus_lookup_table_get_label(self.get_table(), idx);
        ibus_engine_commit_text(self.parent_engine as *mut IBusEngine, symbol);

        self.symbol_input_disable()
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


#[no_mangle]
pub unsafe extern "C" fn ibus_eei_engine_process_key_event(engine: *mut IBusEngine, keyval: guint,
    keycode: guint, modifiers: guint) -> gboolean {

    log::info!("Process key {}", keyval);

    let engine_core = match EngineCore::get(engine) {
        Some(engine_ref) => engine_ref,
        None => {
            log::error!("Could not retrieve engine core");
            return GBOOL_FALSE
        }
    };


    if (modifiers & IBusModifierType_IBUS_RELEASE_MASK) != 0 {
        log::info!("release");
        return GBOOL_FALSE;
    }

    if (modifiers & IBusModifierType_IBUS_CONTROL_MASK) == IBusModifierType_IBUS_CONTROL_MASK
        && keyval == IBUS_s {
        log::info!("enable symbol input");
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
                return engine_core.symbol_input_commit();
            }
            //TODO: if we are selecting words or in emoji mode, commit our current selection
            //otherwise reset word buffer
            GBOOL_TRUE
        }
        IBUS_Up => {
            if engine_core.lookup_visible {
                ibus_lookup_table_cursor_up(engine_core.get_table());
                ibus_engine_update_lookup_table(engine_core.parent_engine_as_ibus_engine(), engine_core.get_table(), GBOOL_TRUE);
                GBOOL_TRUE
            } else {
                GBOOL_FALSE
            }
        }
        IBUS_Down => {
            if engine_core.lookup_visible {
                ibus_lookup_table_cursor_down(engine_core.get_table());
                ibus_engine_update_lookup_table(engine_core.parent_engine_as_ibus_engine(), engine_core.get_table(), GBOOL_TRUE);
                GBOOL_TRUE
            } else {
                GBOOL_FALSE
            }
        }
        IBUS_space..=IBUS_asciitilde => {
            if engine_core.symbol_input {
                return engine_core.symbol_input_char(keyval);
            } else {
                return engine_core.commit_char(keyval);
            }
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




