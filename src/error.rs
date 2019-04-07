pub(crate) struct Error {
    // user-facing error output
    pub(crate) message: String,
}

impl<T> From<T> for Error
where
    T: std::error::Error,
{
    fn from(err: T) -> Error {
        Error {
            message: format!("{}", err),
        }
    }
}

impl Error {
    pub(crate) fn log(err: Self) -> Self {
        println!("{}", err);
        err
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
