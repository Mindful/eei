#![allow(non_upper_case_globals)]
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
use ibus::{IBusEEIEngine, gboolean, GBOOL_FALSE, ibus_engine_update_lookup_table, IBusEngine, GBOOL_TRUE, ibus_engine_hide_lookup_table, guint, IBusModifierType_IBUS_RELEASE_MASK, IBusModifierType_IBUS_CONTROL_MASK, IBUS_e, IBUS_asciitilde, IBUS_space, IBUS_Return, IBUS_BackSpace, IBUS_Escape, IBUS_Page_Down, IBUS_Page_Up, ibus_engine_commit_text, ibus_text_new_from_unichar, ibus_text_new_from_string, gchar, ibus_lookup_table_clear, ibus_lookup_table_append_candidate, IBusText, ibus_lookup_table_append_label, ibus_engine_update_auxiliary_text, IBUS_Up, IBUS_Down, IBUS_Left, IBUS_Right, ibus_lookup_table_get_cursor_pos, IBusLookupTable, ibus_lookup_table_get_label, ibus_lookup_table_cursor_up, ibus_lookup_table_cursor_down, ibus_engine_hide_auxiliary_text, ibus_lookup_table_set_label, ibus_lookup_table_page_down, ibus_lookup_table_page_up, ibus_lookup_table_get_number_of_candidates, ibus_text_new_from_static_string, ibus_engine_show_lookup_table, ibus_lookup_table_get_cursor_in_page, gunichar, IBusModifierType_IBUS_SHIFT_MASK};
use std::cmp::min;


pub struct EngineCore {
    lookup_visible: bool,
    word_buffer: String,
    cursor_pos: i32,
    symbol_input: bool,
    symbol_preedit: String,
    symbol_label_vec: Vec<CString>,
    symbol_last_page: guint,
    parent_engine: *mut IBusEEIEngine
}

#[no_mangle]
pub unsafe extern "C" fn new_engine_core(parent_engine: *mut IBusEEIEngine) -> *mut EngineCore {
    Box::into_raw(Box::new(EngineCore {
        lookup_visible: false,
        word_buffer: String::new(),
        cursor_pos: 0,
        symbol_input: false,
        symbol_preedit: String::new(),
        symbol_label_vec: Vec::new(),
        symbol_last_page: 0,
        parent_engine: parent_engine
    }))
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

    unsafe fn update_lookup_table(&mut self) {
        if self.symbol_input {
            let page_size = (*self.get_table()).page_size;
            let idx = ibus_lookup_table_get_cursor_pos(self.get_table());
            let page_num = idx / page_size;
            if self.symbol_last_page != page_num {
                self.symbol_last_page = page_num;
                for (idx, table_idx) in (page_num * page_size..min((page_size * (page_size+1)), self.symbol_label_vec.len() as u32)).enumerate() {
                    ibus_lookup_table_set_label(self.get_table(), idx as guint, ibus_text_new_from_static_string(self.symbol_label_vec.get_unchecked(table_idx as usize).as_ptr()))
                }
            }
        }
        ibus_engine_update_lookup_table(self.parent_engine_as_ibus_engine(), self.get_table(), GBOOL_TRUE);
    }

    unsafe fn symbol_input_enable(&mut self) -> gboolean {
        if self.symbol_input {
            return GBOOL_FALSE;
        }

        self.symbol_input = true;
        self.lookup_visible = true;
        ibus_engine_show_lookup_table(self.parent_engine_as_ibus_engine());
        GBOOL_TRUE
    }

    unsafe fn symbol_input_disable(&mut self) -> gboolean {
        if !self.symbol_input {
            return GBOOL_FALSE;
        }

        self.symbol_input = false;
        self.lookup_visible = false;
        self.symbol_preedit.clear();
        ibus_engine_hide_lookup_table(self.parent_engine_as_ibus_engine());
        ibus_engine_hide_auxiliary_text(self.parent_engine_as_ibus_engine());
        GBOOL_TRUE
    }

    unsafe fn commit_char(&mut self, keyval: guint) -> gboolean {
        self.word_buffer.push((keyval as u8) as char);
        ibus_engine_commit_text(self.parent_engine_as_ibus_engine(), ibus_text_new_from_unichar(keyval as gunichar));
        GBOOL_TRUE
    }

    unsafe fn symbol_input_update(&mut self) -> gboolean {
        match into_ibus_string(self.symbol_preedit.clone()) {
            Ok(ibus_string) => {
                ibus_engine_update_auxiliary_text(self.parent_engine_as_ibus_engine(), ibus_string, GBOOL_TRUE);
            }
            Err(err) => {
                log::error!("Failed string conversion for symbol aux text update");
            }
        }

        if self.symbol_preedit.is_empty() {
            return GBOOL_TRUE;
        }

        let search_result  = PREDICTOR.symbol(self.symbol_preedit.as_str());
        match search_result {
            Ok(candidates) => {
                log::info!("Symbol search for {} and got {:?}", self.symbol_preedit, candidates);
                let table = self.get_table();
                // Must clear table first, since the table may have IBusText referencing the
                // symbol_label_vec strings
                ibus_lookup_table_clear(table);
                self.symbol_label_vec.clear();
                for (idx, (shortcode, ident)) in candidates.into_iter().enumerate() {
                    match (CString::new(shortcode.into_bytes()),  CString::new(ident.into_bytes())) {
                        (Ok(shortcode_cstring), Ok(ident_cstring)) => {
                            ibus_lookup_table_append_candidate(table, ibus_text_new_from_string(shortcode_cstring.into_raw() as *mut gchar));
                            self.symbol_label_vec.push(ident_cstring);
                            if idx < (*table).page_size as usize {
                                ibus_lookup_table_set_label(table, idx as guint, ibus_text_new_from_static_string(self.symbol_label_vec.get_unchecked(idx).as_ptr()));
                            }
                        }
                        _ => {
                            log::error!("Failed string conversion for symbol lookup");
                        }
                    }
                }
                log::info!("{} candidates and {} labels", ibus_lookup_table_get_number_of_candidates(self.get_table()), (*(*self.get_table()).labels).len);
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

        let idx = ibus_lookup_table_get_cursor_in_page(self.get_table());
        let symbol = ibus_lookup_table_get_label(self.get_table(), idx);
        ibus_engine_commit_text(self.parent_engine as *mut IBusEngine, symbol);

        self.symbol_input_disable()
    }
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


    if (modifiers & IBusModifierType_IBUS_CONTROL_MASK) == IBusModifierType_IBUS_CONTROL_MASK && keyval == IBUS_e {
        log::info!("enable symbol input");
        return engine_core.symbol_input_enable();
    } else if (modifiers & !IBusModifierType_IBUS_SHIFT_MASK) != 0 {
        return GBOOL_FALSE; //This also covers released keys with IBUS_RELEASE_MASK
    }

    match keyval {
        IBUS_space => {
            //TODO: handle aborting words, not just symbols
            if engine_core.symbol_input {
                engine_core.symbol_input_disable();
            }
            engine_core.commit_char(keyval)
        }
        IBUS_Return => {
            //TODO also handle committing words, not just symbols
            if engine_core.symbol_input {
                engine_core.symbol_input_commit()
            } else {
                engine_core.commit_char(keyval)
            }
        }
        IBUS_Up => {
            if engine_core.lookup_visible {
                ibus_lookup_table_cursor_up(engine_core.get_table());
                engine_core.update_lookup_table();
                GBOOL_TRUE
            } else {
                GBOOL_FALSE
            }
        }
        IBUS_Down => {
            if engine_core.lookup_visible {
                ibus_lookup_table_cursor_down(engine_core.get_table());
                engine_core.update_lookup_table();
                GBOOL_TRUE
            } else {
                GBOOL_FALSE
            }
        }
        IBUS_BackSpace => {
            if engine_core.symbol_input {
                engine_core.symbol_preedit.pop();
                return engine_core.symbol_input_update();
            }
            GBOOL_FALSE
        }
        IBUS_Page_Down => {
            if engine_core.lookup_visible {
                log::info!("pageup");
                let res = ibus_lookup_table_page_down(engine_core.get_table());
                engine_core.update_lookup_table();
                return res
            }
            GBOOL_FALSE
        }
        IBUS_Page_Up => {
            if engine_core.lookup_visible {
                log::info!("pagedown");
                let res = ibus_lookup_table_page_up(engine_core.get_table());
                engine_core.update_lookup_table();
                return res
            }
            GBOOL_FALSE
        }
        IBUS_Escape => {
            if engine_core.symbol_input {
                return engine_core.symbol_input_disable();
            }
            GBOOL_FALSE
        }
        IBUS_space..=IBUS_asciitilde => {
            return if engine_core.symbol_input {
                engine_core.symbol_preedit.push((keyval as u8) as char);
                engine_core.symbol_input_update()
            } else {
                engine_core.commit_char(keyval)
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




