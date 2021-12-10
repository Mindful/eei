#![allow(non_upper_case_globals)]
mod predict;

use std::ffi::{CString, NulError, CStr};
use std::os::raw::{c_char, c_int};
use log::{LevelFilter};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Root};

use crate::predict::PREDICTOR;
use ibus::{IBusEEIEngine, gboolean, GBOOL_FALSE, ibus_engine_update_lookup_table, IBusEngine, GBOOL_TRUE, ibus_engine_hide_lookup_table, guint, IBusModifierType_IBUS_CONTROL_MASK, IBUS_e, IBUS_w, IBUS_asciitilde, IBUS_space, IBUS_Return, IBUS_BackSpace, IBUS_Escape, IBUS_Page_Down, IBUS_Page_Up, ibus_engine_commit_text, ibus_text_new_from_unichar, ibus_text_new_from_string, gchar, ibus_lookup_table_clear, ibus_lookup_table_append_candidate, IBusText, ibus_engine_update_auxiliary_text, IBUS_Up, IBUS_Down, ibus_lookup_table_get_cursor_pos, IBusLookupTable, ibus_lookup_table_get_label, ibus_lookup_table_cursor_up, ibus_lookup_table_cursor_down, ibus_engine_hide_auxiliary_text, ibus_lookup_table_set_label, ibus_lookup_table_page_down, ibus_lookup_table_page_up, ibus_lookup_table_get_number_of_candidates, ibus_text_new_from_static_string, ibus_lookup_table_get_cursor_in_page, gunichar, IBusModifierType_IBUS_SHIFT_MASK, ibus_lookup_table_get_candidate, ibus_engine_update_preedit_text, ibus_engine_hide_preedit_text, ibus_text_get_length, ibus_text_append_attribute, IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE, IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE, gint};
use std::cmp::min;
use lazy_static::lazy_static;
use InputMode::*;
use std::path::Path;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::filter::threshold::ThresholdFilter;
use log4rs::append::rolling_file::RollingFileAppender;

lazy_static! {
    static ref empty_cstring: CString = CString::new("").unwrap();
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    SymbolTable,
    WordTable
}

pub struct EngineCore {
    table_visible: bool,
    word_buffer: String,
    input_mode: InputMode,
    symbol_preedit: String,
    symbol_label_vec: Vec<CString>,
    symbol_last_page: guint,
    parent_engine: *mut IBusEEIEngine
}

#[no_mangle]
pub unsafe extern "C" fn new_engine_core(parent_engine: *mut IBusEEIEngine) -> *mut EngineCore {
    Box::into_raw(Box::new(EngineCore {
        table_visible: false,
        word_buffer: String::new(),
        input_mode: InputMode::Normal,
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

    /*
    ** General Methods **
    */

    fn parent_engine_as_ibus_engine(&self) -> *mut IBusEngine {
        self.parent_engine as *mut IBusEngine
    }

    unsafe fn get_table(&self) -> *mut IBusLookupTable {
        (*self.parent_engine).table
    }

    unsafe fn get(engine: *mut IBusEngine) -> Option<&'static mut EngineCore> {
        ((*(engine as *mut IBusEEIEngine)).engine_core as *mut EngineCore).as_mut()
    }

    unsafe fn commit_char(&mut self, keyval: guint) {
        self.word_buffer.push((keyval as u8) as char);
        ibus_engine_commit_text(self.parent_engine_as_ibus_engine(), ibus_text_new_from_unichar(keyval as gunichar));
    }

    unsafe fn commit_text(&mut self, text: *mut IBusText) {
        log::info!("commit text {}", CStr::from_ptr((*text).text as *mut c_char).to_str().unwrap());
        ibus_engine_commit_text(self.parent_engine_as_ibus_engine(), text);
    }

    unsafe fn update_lookup_table(&mut self) {
        if self.input_mode == SymbolTable {
            let page_size = (*self.get_table()).page_size;
            let idx = ibus_lookup_table_get_cursor_pos(self.get_table());
            let page_num = idx / page_size;
            if self.symbol_last_page != page_num {
                self.symbol_last_page = page_num;
                for (idx, table_idx) in (page_num * page_size..min(page_size * (page_size+1), self.symbol_label_vec.len() as u32)).enumerate() {
                    ibus_lookup_table_set_label(self.get_table(), idx as guint, ibus_text_new_from_static_string(self.symbol_label_vec.get_unchecked(table_idx as usize).as_ptr()))
                }
            }
        }
        ibus_engine_update_lookup_table(self.parent_engine_as_ibus_engine(), self.get_table(), GBOOL_TRUE);
        self.update_preedit();
    }

    unsafe fn update_preedit(&mut self) {
        match self.input_mode {
            SymbolTable => {
                let idx = ibus_lookup_table_get_cursor_pos(self.get_table());
                let symbol = ibus_lookup_table_get_label(self.get_table(), idx);
                let len = ibus_text_get_length(symbol);
                ibus_text_append_attribute(symbol, IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
                                           IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE, 0, len as gint);
                ibus_engine_update_preedit_text(self.parent_engine_as_ibus_engine(), symbol,
                                                len,GBOOL_TRUE);
            }
            WordTable => {
                let idx = ibus_lookup_table_get_cursor_pos(self.get_table());
                let candidate = ibus_lookup_table_get_candidate(self.get_table(), idx);
                self.get_word_remainder(candidate).map(|remainder| {
                    let len = ibus_text_get_length(remainder);
                    ibus_text_append_attribute(remainder, IBusAttrType_IBUS_ATTR_TYPE_UNDERLINE,
                                               IBusAttrUnderline_IBUS_ATTR_UNDERLINE_SINGLE, 0, len as gint);
                    ibus_engine_update_preedit_text(self.parent_engine_as_ibus_engine(), remainder,
                                                    len, GBOOL_TRUE);
                });
            }
            Normal => {}
        }
    }

    /*
    ** Word input methods
     */

    unsafe fn word_table_enable(&mut self) {
        if self.table_visible || self.word_buffer.is_empty() {
            //not an error if this is called while word buffer is empty, so don't log
            return;
        }

        self.input_mode = WordTable;
        self.table_visible = true;
        self.word_table_update();
    }

    unsafe fn word_table_disable(&mut self) {
        if !self.table_visible {
            log::error!("Call to disable word table while no table is visible");
            return;
        }

        self.input_mode = Normal;
        self.table_visible = false;
        ibus_engine_hide_preedit_text(self.parent_engine_as_ibus_engine());
        ibus_engine_hide_lookup_table(self.parent_engine_as_ibus_engine());
    }

    unsafe fn word_table_update(&mut self) {
        if !self.table_visible || self.input_mode != WordTable {
            log::error!("Word table update called while table invisible or input mode is not word");
            return;
        }
        else if self.word_buffer.is_empty() {
            self.word_table_disable();
            return;
        }

        let search_result  = PREDICTOR.word(self.word_buffer.as_str());
        match search_result {
            Ok(candidates) => {
                log::info!("Word search for {} and got {:?}", self.word_buffer, candidates);
                let table = self.get_table();
                ibus_lookup_table_clear(table);
                for word in candidates {
                    match into_ibus_string(word) {
                        Ok(ibus_text) => {
                            ibus_lookup_table_append_candidate(table, ibus_text)
                        }
                        Err(err) => {
                            log::error!("Failed string conversion for word lookup: {}", err);
                        }
                    }
                }
                ibus_engine_update_lookup_table(self.parent_engine_as_ibus_engine(), table, GBOOL_TRUE);
                self.update_preedit();
            }
            Err(err) => {
                log::error!("{}", err);
            }
        }
    }

    unsafe fn word_commit(&mut self) {
        if !self.table_visible || self.input_mode != WordTable {
            log::error!("Word commit called outside word input mode");
            return;
        }

        let idx = ibus_lookup_table_get_cursor_pos(self.get_table());
        let candidate = ibus_lookup_table_get_candidate(self.get_table(), idx);
        self.get_word_remainder(candidate)
            .map(|remainder| {
                self.commit_text(remainder)
            });

        self.word_buffer.clear();
        self.word_table_disable();
    }

    unsafe fn get_word_remainder(&self, candidate: *mut IBusText) -> Option<*mut IBusText> {
        match CStr::from_ptr((*candidate).text as *const c_char).to_str() {
            Ok(word) => {
                match into_ibus_string(String::from(&word[self.word_buffer.len()..])) {
                    Ok(ibus_word) => {
                        Some(ibus_word)
                    }
                    Err(err) => {
                        log::error!("Failed to convert slice back into ibus string: {}", err);
                        None
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to convert word to string: {}", err);
                None
            }
        }
    }


    /*
    ** Symbol input methods **
     */

    unsafe fn symbol_table_enable(&mut self) {
        if self.table_visible {
            log::error!("Call to enable symbol table while a table is already visible");
            return;
        }

        self.input_mode = SymbolTable;
        self.table_visible = true;
        ibus_lookup_table_clear(self.get_table());
        ibus_engine_update_lookup_table(self.parent_engine_as_ibus_engine(), self.get_table(), GBOOL_TRUE);
    }

    unsafe fn symbol_table_disable(&mut self) {
        if self.input_mode != SymbolTable {
            log::error!("Call to disable symbol table outside of symbol input mode");
            return;
        }

        self.input_mode = Normal;
        self.table_visible = false;
        self.symbol_preedit.clear();
        ibus_engine_hide_preedit_text(self.parent_engine_as_ibus_engine());
        ibus_engine_hide_lookup_table(self.parent_engine_as_ibus_engine());
        ibus_engine_hide_auxiliary_text(self.parent_engine_as_ibus_engine());
        for i in 0..(*self.get_table()).page_size {
            ibus_lookup_table_set_label(self.get_table(), i, ibus_text_new_from_static_string(empty_cstring.as_ptr()));
        }
    }

    unsafe fn symbol_input_update(&mut self) {
        if !self.table_visible || self.input_mode != SymbolTable {
            log::error!("Word table update called while table invisible or input mode is not symbol");
            return;
        }
        match into_ibus_string(self.symbol_preedit.clone()) {
            Ok(ibus_string) => {
                ibus_engine_update_auxiliary_text(self.parent_engine_as_ibus_engine(), ibus_string, GBOOL_TRUE);
            }
            Err(err) => {
                log::error!("Failed string conversion for symbol aux text update: {}", err);
            }
        }

        if self.symbol_preedit.is_empty() {
            return;
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
            }
        }
        self.update_preedit();
    }

    unsafe fn symbol_commit(&mut self) {
        if self.input_mode != SymbolTable {
            log::error!("Symbol input commit called outside symbol input mode");
            return;
        }

        let idx = ibus_lookup_table_get_cursor_in_page(self.get_table());
        let symbol = ibus_lookup_table_get_label(self.get_table(), idx);
        self.commit_text(symbol);

        self.symbol_table_disable();
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

#[allow(unused_variables)]
#[no_mangle]
pub unsafe extern "C" fn ibus_eei_engine_process_key_event(engine: *mut IBusEngine, keyval: guint,
    keycode: guint, modifiers: guint) -> gboolean {

    let engine_core = match EngineCore::get(engine) {
        Some(engine_ref) => engine_ref,
        None => {
            log::error!("Could not retrieve engine core");
            return GBOOL_FALSE
        }
    };


    if modifiers == IBusModifierType_IBUS_CONTROL_MASK {
        //control key (and only control key) is held down
        return match keyval {
            IBUS_e => {
                match engine_core.input_mode {
                    SymbolTable => {
                        engine_core.symbol_table_disable();
                    }
                    WordTable => {}
                    Normal => {
                        engine_core.symbol_table_enable();
                    }
                }
                GBOOL_TRUE
            }
            IBUS_w => {
                match engine_core.input_mode {
                    SymbolTable => {}
                    WordTable => {
                        engine_core.word_table_disable();
                    }
                    Normal => {
                        engine_core.word_table_enable();
                    }
                }
                GBOOL_TRUE
            }
            _ => {
                GBOOL_FALSE
            }
        }
    } else if (modifiers & !IBusModifierType_IBUS_SHIFT_MASK) != 0 {
        return GBOOL_FALSE; //This also covers released keys with IBUS_RELEASE_MASK
    }

    match keyval {
        IBUS_space => {
            match engine_core.input_mode {
                SymbolTable => {
                    engine_core.symbol_table_disable();
                },
                WordTable => {
                    engine_core.word_table_disable();
                }
                Normal => {}
            }
            engine_core.commit_char(keyval);
            engine_core.word_buffer.clear();
            GBOOL_TRUE
        }
        IBUS_Return => {
            let ret = match engine_core.input_mode {
                SymbolTable => {
                    engine_core.symbol_commit();
                    GBOOL_TRUE
                }
                WordTable => {
                    engine_core.word_commit();
                    GBOOL_TRUE
                }
                Normal => {
                    GBOOL_FALSE
                }
            };
            engine_core.word_buffer.clear();
            ret
        }
        IBUS_Up => {
            if engine_core.table_visible {
                let ret = ibus_lookup_table_cursor_up(engine_core.get_table());
                engine_core.update_lookup_table();
                ret
            } else {
                GBOOL_FALSE
            }
        }
        IBUS_Down => {
            if engine_core.table_visible {
                let ret = ibus_lookup_table_cursor_down(engine_core.get_table());
                engine_core.update_lookup_table();
                ret
            } else {
                GBOOL_FALSE
            }
        }
        IBUS_BackSpace => {
            match engine_core.input_mode {
                SymbolTable => {
                    engine_core.symbol_preedit.pop();
                    engine_core.symbol_input_update();
                    GBOOL_TRUE
                }
                WordTable => {
                    engine_core.word_buffer.pop();
                    engine_core.word_table_update();
                    GBOOL_FALSE //let the character pass through so deletion still happens
                }
                Normal => {
                    GBOOL_FALSE
                }
            }
        }
        IBUS_Page_Down => {
            if engine_core.table_visible {
                let res = ibus_lookup_table_page_down(engine_core.get_table());
                engine_core.update_lookup_table();
                return res
            } else {
                GBOOL_FALSE
            }
        }
        IBUS_Page_Up => {
            if engine_core.table_visible {
                let res = ibus_lookup_table_page_up(engine_core.get_table());
                engine_core.update_lookup_table();
                return res
            } else {
                GBOOL_FALSE
            }
        }
        IBUS_Escape => {
            match engine_core.input_mode {
                SymbolTable => {
                    engine_core.symbol_table_disable();
                    GBOOL_TRUE
                }
                WordTable => {
                    engine_core.word_table_disable();
                    GBOOL_TRUE
                }
                Normal => {
                    GBOOL_FALSE
                }
            }
        }
        IBUS_space..=IBUS_asciitilde => {
            match engine_core.input_mode {
                SymbolTable => {
                    engine_core.symbol_preedit.push((keyval as u8) as char);
                    engine_core.symbol_input_update();
                    GBOOL_TRUE
                }
                WordTable => {
                    engine_core.commit_char(keyval);
                    engine_core.word_table_update();
                    GBOOL_TRUE
                }
                Normal => {
                    engine_core.commit_char(keyval);
                    GBOOL_TRUE
                }
            }
        }
        _ => GBOOL_FALSE
    }
}


static DATA_DIRNAME: &str = "eei";

#[no_mangle]
pub unsafe extern "C" fn configure_logging() {
    //https://stackoverflow.com/questions/56345288/how-do-i-use-log4rs-rollingfileappender-to-incorporate-rolling-logging

    let log_location = std::env::var("XDG_DATA_HOME").map(|dir| Path::new(dir.as_str()).join(DATA_DIRNAME))
        .or(std::env::var("HOME").map(|home| Path::new(home.as_str()).join(".local").join("share").join(DATA_DIRNAME)));

    match log_location {
        Ok(location) => {
            // https://stackoverflow.com/questions/56345288/how-do-i-use-log4rs-rollingfileappender-to-incorporate-rolling-logging
            let window_size = 3; // log0, log1, log2
            let fixed_window_roller = FixedWindowRoller::builder().build(location.join("log_archive_{}.txt").to_str().unwrap(), window_size).unwrap();
            let size_limit = 1024 * 5; // 5KB as max size before roll
            let size_trigger = SizeTrigger::new(size_limit);
            let compound_policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));

            let log_level = if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Warn
            };

            let config = Config::builder()
                .appender(
                    Appender::builder()
                        .filter(Box::new(ThresholdFilter::new(log_level)))
                        .build(
                            "logfile",
                            Box::new(
                                RollingFileAppender::builder()
                                    .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {l}::{m}{n}")))
                                    .build(location.join("log.txt"), Box::new(compound_policy)).unwrap(),
                            ),
                        ),
                )
                .build(
                    Root::builder()
                        .appender("logfile")
                        .build(LevelFilter::Debug),
                ).unwrap();

            log4rs::init_config(config).unwrap();

            log::info!("Logging initialized");
        }
        Err(err) => {
            println!("ERROR: COULD NOT INITIALIZE LOGGING: {}", err)
        }
    }
}


