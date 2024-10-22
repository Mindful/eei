use fst::MapBuilder;
use std::collections::{HashMap, HashSet};
use std::error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::num::ParseIntError;

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum InvalidParseError {
    Json(String),
    Hex(ParseIntError),
    Codepoint(u32),
    WordFreq(String),
}

//code point;class;char;entity name;entity set;note/description;CHARACTER NAME

impl Display for InvalidParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for InvalidParseError {}

//https://stackoverflow.com/questions/69152223/unicode-codepoint-to-rust-string
fn parse_unicode(input: &str) -> Result<char, InvalidParseError> {
    let unicode = u32::from_str_radix(input, 16).map_err(InvalidParseError::Hex)?;
    char::from_u32(unicode).ok_or(InvalidParseError::Codepoint(unicode))
}

fn parse_github_emoji_url(url: &str) -> Result<String, InvalidParseError> {
    let bytecode_strings = url
        .split('/')
        .last()
        .ok_or(InvalidParseError::Json(url.to_owned()))?
        .split('.')
        .next()
        .ok_or(InvalidParseError::Json(url.to_owned()))?
        .split('-');

    bytecode_strings
        .map(parse_unicode)
        .collect::<Result<Vec<_>, _>>()
        .map(|char_vec| char_vec.into_iter().collect::<String>())
}

fn math_symbol_shortcodes() -> Vec<(String, String)> {
    let whitelist = io::BufReader::new(File::open("math_whitelist.txt").unwrap())
        .lines()
        .collect::<Result<HashSet<String>, _>>()
        .unwrap();

    let reader = ureq::get("https://www.unicode.org/Public/math/revision-15/MathClassEx-15.txt")
        .call()
        .unwrap()
        .into_reader();

    let rdr = csv::ReaderBuilder::new()
        .comment(Some(b'#'))
        .delimiter(b';')
        .has_headers(false)
        .from_reader(reader);

    rdr.into_records()
        .filter_map(|result| {
            result.ok().map(|record| {
                let symbol = &record[2];
                (String::from(&record[3]), String::from(symbol)) //shortcode, symbol
            })
        })
        .filter(|(_shortcode, symbol)| whitelist.contains(symbol))
        .collect()
}

fn github_emoji_shortcodes() -> Vec<(String, String)> {
    let json: HashMap<String, String> = ureq::get("https://api.github.com/emojis")
        .call()
        .unwrap()
        .into_json()
        .unwrap();

    //have to filter out bad URLs like
    // "https://github.githubassets.com/images/icons/emoji/bowtie.png?v8"
    json.iter()
        .filter_map(|(key, url)| {
            let key_chars: Vec<char> = key.chars().collect();
            if key_chars.first().map(|c| c == &'u').unwrap_or(false)
                && key_chars
                    .get(1)
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                None //filter out "u5272" etc. for Japanese emoji
            } else {
                parse_github_emoji_url(url)
                    .map(|unicode_str| (key.clone(), unicode_str))
                    .ok()
            }
        })
        .collect::<Vec<(String, String)>>()
}

fn write_symbols_and_shortcodes(
    mut shortcodes_symbols: Vec<(String, String)>,
) -> Result<(), Box<dyn error::Error>> {
    let writer = io::BufWriter::new(File::create("shortcodes.fst")?);
    let mut map_builder = MapBuilder::new(writer)?;

    shortcodes_symbols.sort();

    let symbols = shortcodes_symbols
        .iter()
        .map(|(_shortcode, symbol)| symbol)
        .collect::<HashSet<&String>>();

    let symbol_id_map: HashMap<&String, u64> = symbols
        .iter()
        .enumerate()
        .map(|(idx, symbol)| (*symbol, idx as u64))
        .collect();

    for (shortcode, symbol) in shortcodes_symbols.iter() {
        map_builder.insert(shortcode, *symbol_id_map.get(symbol).unwrap())?;
    }

    // Finish construction of the map and flush its contents to disk.
    map_builder.finish()?;

    let mut symbol_file = File::create("symbols.bin")?;
    symbol_file.write_all(&bincode::serialize(&symbols)?)?;

    println!(
        "Wrote {shortcodes} shortcodes for {symbols} symbols",
        shortcodes = shortcodes_symbols.len(),
        symbols = symbols.len()
    );

    Ok(())
}

fn load_word_freq_data() -> Result<HashMap<String, u64>, Box<dyn error::Error>> {
    let lines = io::BufReader::new(File::open("count_1w.txt")?)
        .lines()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(lines
        .into_iter()
        .filter_map(|line| {
            if line.is_empty() {
                None
            } else {
                let mut split_line = line.split("\t");
                Some(
                    split_line
                        .next()
                        .ok_or(InvalidParseError::WordFreq(line.clone()))
                        .and_then(|word| {
                            split_line
                                .last()
                                .ok_or(InvalidParseError::WordFreq(line.clone()))
                                .and_then(|x| {
                                    x.parse::<u64>()
                                        .map_err(|_| InvalidParseError::WordFreq(line.clone()))
                                })
                                .map(|count| (word.to_lowercase(), count))
                        }),
                )
            }
        })
        .collect::<Result<HashMap<String, u64>, InvalidParseError>>()?)
}

fn process_dictionary() -> Result<(), Box<dyn error::Error>> {
    let writer = io::BufWriter::new(File::create("dictionary.fst")?);
    let mut map_builder = MapBuilder::new(writer)?;

    let mut lines = io::BufReader::new(File::open("hunspell_US.txt")?)
        .lines()
        .map(|line_res| line_res.map(|line| line.to_lowercase()))
        .collect::<Result<Vec<_>, _>>()?;

    //must be in lexographical order to build the FST
    lines.sort();
    lines.dedup();
    let word_freq = load_word_freq_data()?;

    let mut words_without_freq = 0;

    for line in lines.iter() {
        map_builder.insert(
            line,
            *word_freq.get(line.as_str()).unwrap_or_else(|| {
                words_without_freq += 1;
                &0
            }),
        )?;
    }

    let words_with_freq = lines.len() - words_without_freq;

    map_builder.finish()?;
    println!(
        "Wrote {entries} dictionary entries, of which {with_freq} had frequency ({perc:.2}%)",
        entries = lines.len(),
        with_freq = words_with_freq,
        perc = (words_with_freq as f64 / lines.len() as f64)
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    println!("Fetching math symbols");
    let math_symbols = math_symbol_shortcodes();

    println!("Fetching shortcodes from github");
    let shortcodes = github_emoji_shortcodes();

    let all_symbols = [math_symbols, shortcodes].concat();

    println!("Writing symbols and shortcodes to files");
    write_symbols_and_shortcodes(all_symbols)?;
    println!("Processing dictionary");
    process_dictionary()?;

    Ok(())
}
