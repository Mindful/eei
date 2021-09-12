use std::collections::HashMap;
use fst::{Map, Error};

#[derive(Debug, Clone)]
struct ParseError;

fn parse_github_emoji_url(url: String) -> Result<String, ParseError> {
    let bytecode_strings: Vec<&str> = url.split('/')
        .last().ok_or(ParseError)?
        .split('.').next().ok_or(ParseError)?
        .split('-').collect();

    Ok(String::from("Dog"))
}

fn main() {
    let test_str = String::from("https://github.githubassets.com/images/icons/emoji/unicode/1f44d.png?v8");
    let test_other_Str = String::from("https://github.githubassets.com/images/icons/emoji/unicode/1f1e6-1f1fd.png?v8");
    let l = test_str.split('/').last().unwrap().split('.').next().unwrap();
    let ll: Vec<&str> = l.split("-").collect();


    let json: HashMap<String, String> = ureq::get("https://api.github.com/emojis").call()
        .unwrap()
        .into_json()
        .unwrap();

    let emoji_data: HashMap<_, _> = json.iter()
        .map(|(key, url)| {(key.clone(), url)})
        .collect();
    println!("requested");
}


