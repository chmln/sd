use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    // hide author from help
    author = "",
    about = "",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(setting = "structopt::clap::AppSettings::NextLineHelp"),
    raw(setting = "structopt::clap::AppSettings::UnifiedHelpMessage"),
)]
pub(crate) struct Options {
    #[structopt(short = "p", long = "preview")]
    pub preview: bool,

    #[structopt(short = "i", long = "in-place")]
    // Deprecated. TODO: remove in next release.
    /// (Deprecated). Modify the files in-place. Otherwise, transformations
    /// will be emitted to STDOUT by default.
    pub in_place: bool,

    #[structopt(short = "s", long = "string-mode")]
    /// Treat expressions as non-regex strings.
    pub literal_mode: bool,

    #[structopt(short = "f", long = "flags")]
    /** Regex flags. May be combined (like `-f mc`).

    c - case-sensitive
    i - case-insensitive
    m - multi-line matching
    w - match full words only

    */
    pub flags: Option<String>,

    /// The regexp or string (if -s) to search for.
    pub find: String,

    /// What to replace each match with. Unless in string mode, you may
    /// use captured values like $1, $2, etc.
    pub replace_with: String,

    /// The path to file(s). This is optional - sd can also read from STDIN.
    pub files: Vec<String>,
}
