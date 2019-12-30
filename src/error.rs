use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid regex {0}")]
    Regex(#[from] regex::Error),
    #[error("{0}")]
    File(#[from] std::io::Error),
    #[error("failed to move file: {0}")]
    TempfilePersist(#[from] tempfile::PersistError),
    #[error("file doesn't have parent path: {0}")]
    InvalidPath(std::path::PathBuf),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
