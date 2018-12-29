use crate::{Error, Source, Stream};
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
    file_path: Option<String>,
}

pub(crate) fn run() -> Result<(), Error> {
    let args = Options::from_args();
    let source = Source::from(args.file_path);
    let mut stream: Stream = (&source).into_stream()?;
    stream.replace(!args.literal_mode, &args.find, &args.replace_with)?;

    // replace file in-place, or pipe to stdout
    stream.output(&source, args.in_place)
}
