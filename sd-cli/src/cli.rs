use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "sd",
    author,
    version,
    about,
    max_term_width = 100,
    help_template = "\
{before-help}{name} v{version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}"
)]
pub struct Options {
    #[arg(short, long)]
    /// Display changes in a human reviewable format (the specifics of the
    /// format are likely to change in the future).
    pub preview: bool,

    #[arg(
        short = 'F',
        long = "fixed-strings",
        short_alias = 's',
        alias = "string-mode"
    )]
    /// Treat FIND and REPLACE_WITH args as literal strings
    pub literal_mode: bool,

    #[arg(
        short = 'n',
        long = "max-replacements",
        value_name = "LIMIT",
        default_value_t
    )]
    /// Limit the number of replacements that can occur per file. 0 indicates
    /// unlimited replacements.
    pub replacements: usize,

    #[arg(short, long, verbatim_doc_comment)]
    #[rustfmt::skip]
    /** Regex flags. May be combined (like `-f mc`).

c - case-sensitive

e - disable multi-line matching

i - case-insensitive

m - multi-line matching

s - make `.` match newlines

w - match full words only
    */
    pub flags: Option<String>,

    #[arg(short = 'A', long = "across")]
    /// Process each input as a whole rather than line by line. This allows
    /// patterns to match across line boundaries but uses more memory and
    /// prevents streaming.
    pub across: bool,

    /// The regexp or string (if using `-F`) to search for.
    pub find: String,

    /// What to replace each match with. Unless in string mode, you may
    /// use captured values like $1, $2, etc.
    pub replace_with: String,

    /// The path to file(s). This is optional - sd can also read from STDIN.
    ///
    /// Note: sd modifies files in-place by default. See documentation for
    /// examples.
    pub files: Vec<std::path::PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use clap::CommandFactory;

    #[test]
    fn debug_assert() {
        let cmd = Options::command();
        cmd.debug_assert();
    }
}
