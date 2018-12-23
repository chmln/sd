pub(crate) struct Error {
    // low-level cause, used only for debugging
    cause: String,
    // user-facing error output
    description: String,
}

impl<T> From<T> for Error
where
    T: std::error::Error,
{
    fn from(err: T) -> Error {
        Error {
            cause: err
                .source()
                .map_or_else(|| "N/A".to_string(), |e| e.to_string()),
            description: format!("{}", err),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}\nDetails: {}", self.description, self.cause)
    }
}
