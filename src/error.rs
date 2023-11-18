use std::{fmt, path::PathBuf};

use crate::replacer::InvalidReplaceCapture;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("invalid regex {0}")]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    File(#[from] std::io::Error),
    #[error("failed to move file: {0}")]
    TempfilePersist(#[from] tempfile::PersistError),
    #[error("invalid path: {0}")]
    InvalidPath(PathBuf),
    #[error("{0}")]
    InvalidReplaceCapture(#[from] InvalidReplaceCapture),
    #[error("{0}")]
    FailedJobs(FailedJobs),
}

// pretty-print the error
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct FailedJobs(pub Vec<(PathBuf, Error)>);

impl fmt::Display for FailedJobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Failed processing some inputs\n")?;
        for (source, error) in &self.0 {
            writeln!(f, "    {}: {}", source.display(), error)?;
        }

        Ok(())
    }
}

impl fmt::Debug for FailedJobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
