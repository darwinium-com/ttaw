extern crate pest;

use double_metaphone::{Rule, Word};
use error::Error;
use pest::Parser;
use reqwest;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

lazy_static! {
    static ref CMU_DICT: Result<HashMap<String, Vec<Vec<String>>>, Error> =
        from_json_file(&Path::new("res").join("cmudict.json"));
}

fn rhyming_part(phones: &[String]) -> Option<Vec<String>> {
    for (i, s) in phones.iter().rev().enumerate() {
        if let Some(num) = s.chars().collect::<Vec<char>>().last() {
            if *num == '1' || *num == '2' {
                return phones.get(phones.len() - 1 - i..).map(|v| v.to_vec());
            }
        }
    }

    None
}

fn eval_rhyme(phones_a: &[Vec<String>], phones_b: &[Vec<String>]) -> bool {
    for a in phones_a {
        for b in phones_b {
            if rhyming_part(a) == rhyming_part(b) {
                return true;
            }
        }
    }

    false
}

pub fn rhyme(a: &str, b: &str) -> Result<bool, Error> {
    let cmu_dict: &HashMap<String, Vec<Vec<String>>>;

    match &*CMU_DICT {
        Ok(d) => cmu_dict = d,
        Err(e) => return Err(e.clone()),
    }

    if let (Some(phones_a), Some(phones_b)) = (
        cmu_dict.get(a.to_string().to_lowercase().trim()),
        cmu_dict.get(b.to_string().to_lowercase().trim()),
    ) {
        return Ok(eval_rhyme(phones_a, phones_b));
    }

    Ok(false)
}

fn eval_alliteration(phones_a: &[Vec<String>], phones_b: &[Vec<String>]) -> bool {
    for a in phones_a {
        for b in phones_b {
            if let (Some(a), Some(b)) = (a.first(), b.first()) {
                return a == b;
            }
        }
    }

    false
}

pub fn alliteration(a: &str, b: &str) -> Result<bool, Error> {
    if Word::parse(Rule::vowel_first, a.get(..1).unwrap_or_default()).is_ok() {
        return Ok(false);
    }

    if Word::parse(Rule::vowel_first, b.get(..1).unwrap_or_default()).is_ok() {
        return Ok(false);
    }

    let cmu_dict: &HashMap<String, Vec<Vec<String>>>;

    match &*CMU_DICT {
        Ok(d) => cmu_dict = d,
        Err(e) => return Err(e.clone()),
    }

    if let (Some(phones_a), Some(phones_b)) = (
        cmu_dict.get(a.to_string().to_lowercase().trim()),
        cmu_dict.get(b.to_string().to_lowercase().trim()),
    ) {
        return Ok(eval_alliteration(phones_a, phones_b));
    }

    Ok(false)
}

fn from_json_file(path: &Path) -> Result<HashMap<String, Vec<Vec<String>>>, Error> {
    let dict_json: String;

    if !path.exists() {
        // regenerate if the file isn't there
        download_and_serialze(&path)?;
    }

    dict_json = fs::read_to_string(path)?;
    let dict: HashMap<String, Vec<Vec<String>>> = serde_json::from_str(&dict_json)?;
    Ok(dict)
}

fn download_and_serialze(path: &Path) -> Result<(), Error> {
    let dict_string =
        reqwest::get("https://raw.githubusercontent.com/cmusphinx/cmudict/master/cmudict.dict")?
            .text()?;

    let cursor = io::Cursor::new(dict_string);
    let lines = cursor.lines().collect::<Result<Vec<_>, _>>()?;

    let mut dict: HashMap<String, Vec<Vec<String>>> = HashMap::new();

    for line in lines {
        let entry = line
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if let Some((h, t)) = entry.split_first() {
            if let Some(key) = h.split('(').collect::<Vec<&str>>().first() {
                match dict.get_mut(*key) {
                    Some(v) => {
                        v.push(t.to_vec());
                    }
                    None => {
                        dict.insert(key.to_string(), vec![t.to_vec()]);
                    }
                }
            }
        }
    }

    let serialized = serde_json::to_string(&dict).unwrap();
    fs::write(path, serialized)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_and_serialze() {
        let dir = tempfile::tempdir().unwrap();
        let fpath = dir.path().join("serialized");
        let dict = download_and_serialze(&fpath);
        assert!(dict.is_ok());
    }

    #[test]
    fn test_from_json_file() {
        let dir = tempfile::tempdir().unwrap();
        let fpath = dir.path().join("serialized");
        let dict = from_json_file(&fpath);
        assert!(dict.is_ok());
    }
}