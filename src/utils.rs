pub(crate) fn unescape(s: &str) -> Option<String> {
    use unescape::unescape;
    unescape(s)
}
