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
    /// Transform the file contents in-place. Otherwise, transformation will be
    /// emitted to stdout.
    #[structopt(short = "i", long = "in-place")]
    in_place: bool,

    /// Treat expressions as non-regex strings.
    #[structopt(short = "s", long = "string-mode")]
    literal_mode: bool,

    /// The regexp or string (if -s) to search for.
    find: String,

    /// What to replace each match with. Unless in string mode, you may
    /// use captured values like $1, $2, etc.
    replace_with: String,

    /// The path to file (optional).
    files: Vec<String>,
}

pub(crate) fn run() -> Result<()> {
    let args = Options::from_args();
    let source = Source::from(args.files);
    let replacer =
        Replacer::new(&args.find, &args.replace_with, args.literal_mode)?;
    replacer.run(&source, args.in_place)?;
    Ok(())
}
