use clap::Parser;

#[derive(Parser, Debug)]
#[command(
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
    /// Output result into stdout and do not modify files.
    pub preview: bool,

    #[arg(
        short = 'F',
        long = "fixed-strings",
        short_alias = 's',
        alias = "string-mode"
    )]
    /// Treat FIND and REPLACE_WITH args as literal strings
    pub literal_mode: bool,

    #[arg(short)]
    /// Recursively replace files
    pub recursive: bool,

    #[arg(short = 'n')]
    /// Limit the number of replacements
    pub replacements: Option<usize>,

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

    /// The regexp or string (if -s) to search for.
    pub find: String,

    /// What to replace each match with. Unless in string mode, you may
    /// use captured values like $1, $2, etc.
    pub replace_with: String,

    #[arg(long)]
    /// Overwrite file instead of creating tmp file and swaping atomically
    pub no_swap: bool,

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
