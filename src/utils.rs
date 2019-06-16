pub(crate) type Result<T> = std::result::Result<T, crate::Error>;

pub(crate) fn unescape(s: &str) -> Option<String> {
    unescape::unescape(s)
}
