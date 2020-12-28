use std::path::PathBuf;
use std::fs::File;
use csv::{Reader};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Result as IoResult};
use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
struct Record {
    uid: String,
    content: String,
}

pub fn parse_csv(path: &PathBuf) -> Result<HashMap<String, String>> {
    let rdr = read_csv(path)
        .with_context(|| "could not read file")?;
    let records = process_records(rdr)
        .with_context(|| "could not parse csv")?;
    Ok(records)
}

fn read_csv(path: &PathBuf) -> IoResult<Reader<File>> {
    if !path.exists() {
        return Err(Error::new(ErrorKind::InvalidData,
                              format!("the file `{}` does not exist", &path.to_str().unwrap())));
    }
    Ok(csv::Reader::from_path(path).unwrap())
}

fn process_records(mut rdr: Reader<File>) -> IoResult<HashMap<String, String>> {
    let mut lines = 0;
    let mut res: HashMap<String, String> = HashMap::new();
    for result in rdr.deserialize() {
        let record: Record = result?;
        res.insert(record.uid, record.content);
        lines += 1;
    }
    if lines == 0 {
        return Err(Error::new(ErrorKind::InvalidData, "the file is empty"));
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use std::result;

    #[test]
    fn should_load_csv() {
        let path = PathBuf::from("data/test.csv");
        let rdr = read_csv(&path);

        assert_eq!(3, rdr.unwrap().records().count());
    }

    #[test]
    // #[should_panic(expected = "could not read file")]
    fn should_throw_error_if_no_file()  -> result::Result<(), String> {
        let path = PathBuf::from("data/test1.csv");
        match read_csv(&path) {
            Ok(_) => Err(String::from("expected to fail because there is no such file")),
            Err(e) => {
                if e.to_string().contains("the file `data/test1.csv` does not exist") {
                    Ok(())
                } else {
                    Err(String::from("expected to contain [the file `data/test1.csv` does not exist] message, but did not"))
                }
            }
        }
    }

    #[test]
    fn should_throw_error_if_file_empty() -> result::Result<(), String> {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "").unwrap();
        let path = file.path();
        let pb = PathBuf::from(Box::new(path).as_ref());
        let rdr = read_csv(&pb).unwrap();

        match process_records(rdr) {
            Ok(_) => Err(String::from("expected to fail because file is empty")),
            Err(e) => {
                if e.to_string().contains("the file is empty") {
                    Ok(())
                } else {
                    Err(String::from("expected to contain [the file is empty] message, but did not"))
                }
            }
        }
    }
}