use memmap2::{Mmap, MmapOptions};
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

// TODO: memmap2 docs state that users should implement proper
// procedures to avoid problems the `unsafe` keyword indicate.
// This would be in a later PR.
pub fn make_mmap(path: &PathBuf) -> Result<Mmap> {
    Ok(unsafe { Mmap::map(&File::open(path)?)? })
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

pub fn make_mmap_stdin() -> Result<Mmap> {
    let mut handle = stdin().lock();
    let mut buf = Vec::new();
    handle.read_to_end(&mut buf)?;
    let mut mmap = MmapOptions::new().len(buf.len()).map_anon()?;
    mmap.copy_from_slice(&buf);
    let mmap = mmap.make_read_only()?;
    Ok(mmap)
}
