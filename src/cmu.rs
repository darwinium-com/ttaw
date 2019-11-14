use error::Error;
use reqwest;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

type Pronunciations = Vec<Vec<String>>;

fn download_and_serialze(path: &Path) -> Result<(), Error> {
    let dict_string =
        reqwest::get("https://raw.githubusercontent.com/cmusphinx/cmudict/master/cmudict.dict")?
            .text()?;

    let cursor = io::Cursor::new(dict_string);
    let lines = cursor.lines().collect::<Result<Vec<_>, _>>()?;

    let mut dict: HashMap<String, Pronunciations> = HashMap::new();

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

fn from_json_file(path: &Path) -> Result<HashMap<String, Pronunciations>, Error> {
    let dict_json: String;

    if !path.exists() {
        // regenerate if the file isn't there
        download_and_serialze(&path)?;
    }

    dict_json = fs::read_to_string(path)?;
    let dict: HashMap<String, Pronunciations> = serde_json::from_str(&dict_json)?;
    Ok(dict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;

    #[test]
    fn _serialize_dict() {
        let dir = tempfile::tempdir().unwrap();
        let fpath = dir.path().join("serialized");
        let dict = download_and_serialze(&fpath);
        assert!(dict.is_ok());
    }

    #[test]
    fn from_json() {
        let dir = tempfile::tempdir().unwrap();
        let fpath = dir.path().join("serialized");
        let dict = from_json_file(&fpath);
        assert!(dict.is_ok());
    }
}
