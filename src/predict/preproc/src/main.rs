use std::collections::HashMap;
use fst::{Map, Error};
use std::num::ParseIntError;

#[derive(Debug, Clone)]
enum ParseError {
    InvalidJson(String),
    InvalidHex(ParseIntError),
    InvalidCodepoint(u32)
}

//https://stackoverflow.com/questions/69152223/unicode-codepoint-to-rust-string
fn parse_unicode(input: &str) -> Result<char, ParseError> {
    let unicode = u32::from_str_radix(input, 16).map_err(ParseError::InvalidHex)?;
    char::from_u32(unicode).ok_or_else(|| ParseError::InvalidCodepoint(unicode))
}

fn parse_github_emoji_url(url: &String) -> Result<String, ParseError> {
    let bytecode_strings = url.split('/')
        .last().ok_or(ParseError::InvalidJson(url.clone()))?
        .split('.').next().ok_or(ParseError::InvalidJson(url.clone()))?.split('-');

    bytecode_strings.map(|codepoint| parse_unicode(codepoint))
    .collect::<Result<Vec<_>, _>>().map(|char_vec|char_vec.into_iter().collect::<String>())
}

fn github_emoji_shortcodes() -> Vec<(String, String)> {
    let json: HashMap<String, String> = ureq::get("https://api.github.com/emojis").call()
        .unwrap()
        .into_json()
        .unwrap();

    //have to filter out bad URLs like
    // "https://github.githubassets.com/images/icons/emoji/bowtie.png?v8"
    json.iter().filter_map(|(key, url)| {
            parse_github_emoji_url(url).map(|unicode_str| (key.clone(), unicode_str)).ok()
    }).collect::<Vec<(String, String)>>()
}

fn main() {
    let shortcodes = github_emoji_shortcodes();

    println!("debug");
}


