use crate::{Replacer, Result, Source};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    // hide author from help
    author = "",
    about = "",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(setting = "structopt::clap::AppSettings::NextLineHelp"),
    raw(setting = "structopt::clap::AppSettings::DisableVersion"),
    raw(setting = "structopt::clap::AppSettings::UnifiedHelpMessage"),
)]
pub(crate) struct Options {
    #[structopt(short = "i", long = "in-place")]
    /// Modify the files in-place. Otherwise, transformations will be
    /// emitted to STDOUT by default.
    in_place: bool,

    #[structopt(short = "s", long = "string-mode")]
    /// Treat expressions as non-regex strings.
    literal_mode: bool,

    #[structopt(short = "f", long = "flags")]
    /** Regex flags. May be combined (like `-f mc`).

    c - case-sensitive
    m - multi-line matching
    i - case-insensitive

    Smart-case is enabled by default.
    */
    flags: Option<String>,

    /// The regexp or string (if -s) to search for.
    find: String,

    /// What to replace each match with. Unless in string mode, you may
    /// use captured values like $1, $2, etc.
    replace_with: String,

    /// The path to file(s). This is optional - sd can also read from STDIN.
    files: Vec<String>,
}

pub(crate) fn run() -> Result<()> {
    let args = Options::from_args();
    let source = Source::from(args.files);
    let replacer = Replacer::new(
        args.find, 
        &args.replace_with, 
        args.literal_mode, 
        args.flags
    )?;
    replacer.run(&source, args.in_place)?;
    Ok(())
}
