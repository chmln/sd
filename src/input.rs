use memmap2::{Mmap, MmapOptions};
use std::{
    fs::File,
    io::{stdin, Read},
    path::PathBuf,
};

use crate::error::Result;

#[derive(Debug, PartialEq)]
pub(crate) enum Source {
    Stdin,
    File(PathBuf),
}

impl Source {
    pub(crate) fn from_paths(paths: Vec<PathBuf>) -> Vec<Self> {
        paths.into_iter().map(Self::File).collect()
    }

    pub(crate) fn from_stdin() -> Vec<Self> {
        vec![Self::Stdin]
    }

    pub(crate) fn display(&self) -> String {
        match self {
            Self::Stdin => "STDIN".to_string(),
            Self::File(path) => format!("FILE {}", path.display()),
        }
    }
}

// TODO: memmap2 docs state that users should implement proper
// procedures to avoid problems the `unsafe` keyword indicate.
// This would be in a later PR.
pub(crate) unsafe fn make_mmap(path: &PathBuf) -> Result<Mmap> {
    Ok(Mmap::map(&File::open(path)?)?)
}

pub(crate) fn make_mmap_stdin() -> Result<Mmap> {
    let mut handle = stdin().lock();
    let mut buf = Vec::new();
    handle.read_to_end(&mut buf)?;
    let mut mmap = MmapOptions::new().len(buf.len()).map_anon()?;
    mmap.copy_from_slice(&buf);
    let mmap = mmap.make_read_only()?;
    Ok(mmap)
}
