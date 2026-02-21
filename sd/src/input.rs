use std::{
    fs::File,
    io::{BufRead, BufReader, Read, stdin},
    path::PathBuf,
};

use crate::error::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum Source {
    Stdin,
    File(PathBuf),
}

impl Source {
    pub fn from_paths(paths: Vec<PathBuf>) -> Result<Vec<Self>> {
        paths
            .into_iter()
            .map(|path| {
                if path.exists() {
                    Ok(Source::File(path))
                } else {
                    Err(Error::InvalidPath(path.clone()))
                }
            })
            .collect()
    }

    pub fn from_stdin() -> Vec<Self> {
        vec![Self::Stdin]
    }

    pub fn display(&self) -> String {
        match self {
            Self::Stdin => "STDIN".to_string(),
            Self::File(path) => format!("FILE {}", path.display()),
        }
    }
}

pub fn open_source(source: &Source) -> Result<Box<dyn BufRead + '_>> {
    match source {
        Source::File(path) => {
            let file = File::open(path)?;
            Ok(Box::new(BufReader::new(file)))
        }
        Source::Stdin => {
            let stdin = stdin().lock();
            Ok(Box::new(BufReader::new(stdin)))
        }
    }
}

pub fn read_source(source: &Source) -> Result<Vec<u8>> {
    let mut handle = open_source(source)?;
    let mut buf = Vec::new();
    handle.read_to_end(&mut buf)?;
    Ok(buf)
}
