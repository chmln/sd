use crate::{Error, Source, Stream};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    author = "",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(setting = "structopt::clap::AppSettings::NextLineHelp")
)]
pub(crate) struct Options {
    /// The path to file. The file contents will be transformed in-place.
    #[structopt(short = "i", long = "input")]
    file_path: Option<String>,

    /// Enable regular expressions
    #[structopt(short = "r", long = "regex")]
    enable_regex: bool,

    /// The string or regexp (if --regex) to search for.
    find: String,

    /// What to replace each match with. If regex is enabled,
    /// you may use captured values like $1, $2, etc.
    replace_with: String,
}

pub(crate) fn run() -> Result<(), Error> {
    let args = Options::from_args();
    let source = Source::from(args.file_path);
    let mut stream: Stream = (&source).into_stream()?;
    stream.replace(args.enable_regex, &args.find, &args.replace_with)?;

    // replace file in-place, or pipe to stdout
    stream.output(&source)
}
