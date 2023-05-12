use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    #[clap(short, long)]
    /// Output result into stdout and do not modify files.
    pub preview: bool,

    #[clap(short = 's', long = "string-mode")]
    /// Treat expressions as non-regex strings.
    pub literal_mode: bool,

    #[clap(short)]
    /// Recursively replace files
    pub recursive: bool,

    #[clap(short = 'n')]
    /// Limit the number of replacements
    pub replacements: Option<usize>,

    #[clap(short, long, verbatim_doc_comment)]
    #[rustfmt::skip]
    /** Regex flags. May be combined (like `-f mc`).

c - case-sensitive
e - disable multi-line matching
i - case-insensitive
m - multi-line matching
s - make `.` match newlines
w - match full words only
{n}{n}
    */
    pub flags: Option<String>,

    /// The regexp or string (if -s) to search for.
    pub find: String,

    /// What to replace each match with. Unless in string mode, you may
    /// use captured values like $1, $2, etc.
    pub replace_with: String,

    /// The path to file(s). This is optional - sd can also read from STDIN.
    ///{n}{n}Note: sd modifies files in-place by default. See documentation for
    /// examples.
    pub files: Vec<std::path::PathBuf>,
}
