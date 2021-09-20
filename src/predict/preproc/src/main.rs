use std::collections::{HashMap, HashSet};
use fst::{MapBuilder, SetBuilder};
use std::num::ParseIntError;
use std::fs::File;
use std::io;
use std::error;
use std::io::{Write, BufRead};

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


fn write_symbols_and_shortcodes(mut shortcodes_symbols: Vec<(String, String)>) -> Result<(), Box<dyn error::Error>> {
    let writer = io::BufWriter::new(File::create("shortcodes.fst")?);
    let mut map_builder = MapBuilder::new(writer)?;

    shortcodes_symbols.sort();

    let symbols = shortcodes_symbols.iter()
        .map(|(_shortcode, symbol)| { symbol })
        .collect::<HashSet<&String>>();

    let symbol_id_map: HashMap<&String, u64>  = symbols.iter()
        .enumerate()
        .map(|(idx, symbol)| { (*symbol, idx as u64) })
        .collect();

    for (shortcode, symbol) in shortcodes_symbols.iter() {
        map_builder.insert(shortcode, *symbol_id_map.get(symbol).unwrap())?;
    }

    // Finish construction of the map and flush its contents to disk.
    map_builder.finish()?;

    let mut symbol_file = File::create("symbols.bin")?;
    symbol_file.write_all(&bincode::serialize(&symbols)?)?;

    println!("Wrote {shortcodes} shortcodes for {symbols} symbols",
             shortcodes=shortcodes_symbols.len(),
             symbols=symbols.len());

    Ok(())
}

fn process_dictionary() -> Result<(), Box<dyn error::Error>> {
    let writer = io::BufWriter::new(File::create("dictionary.fst")?);
    let mut set_builder = SetBuilder::new(writer)?;

    let mut lines = io::BufReader::new(File::open("hunspell_US.txt")?)
        .lines()
        .collect::<Result<Vec<_>, _>>()?;

    //must be in lexographical order to build the FST
    lines.sort();

    for line in lines.iter() {
        set_builder.insert(line)?;
    }

    set_builder.finish()?;
    println!("Wrote {entries} dictionary entries", entries=lines.len());
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>>{
    println!("Fetching shortcodes from github");
    let shortcodes = github_emoji_shortcodes();

    println!("Writing symbols and shortcodes to files");
    write_symbols_and_shortcodes(shortcodes)?;
    println!("Processing dictionary");
    process_dictionary()?;

    Ok(())
}


