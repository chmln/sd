use std::{
    fmt::{self, Write},
    path::PathBuf,
};

#[derive(thiserror::Error)]
pub enum Error {
    #[error("invalid regex {0}")]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    File(#[from] std::io::Error),
    #[error("failed to move file: {0}")]
    TempfilePersist(#[from] tempfile::PersistError),
    #[error("file doesn't have parent path: {0}")]
    InvalidPath(PathBuf),
    #[error("failed processing files:\n{0}")]
    FailedProcessing(FailedJobs),
}

pub struct FailedJobs(Vec<(PathBuf, Error)>);

impl From<Vec<(PathBuf, Error)>> for FailedJobs {
    fn from(vec: Vec<(PathBuf, Error)>) -> Self {
        Self(vec)
    }
}

impl fmt::Display for FailedJobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\tFailedJobs(\n")?;
        for (path, err) in &self.0 {
            f.write_str(&format!("\t{:?}: {}\n", path, err))?;
        }
        f.write_char(')')
    }
}

// pretty-print the error
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
