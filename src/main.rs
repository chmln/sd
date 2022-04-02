mod cli;
mod error;
mod input;
pub(crate) mod replacer;
pub(crate) mod utils;

pub(crate) use self::input::{App, Source};
pub(crate) use error::{Error, Result};
use replacer::Replacer;

fn main() -> Result<()> {
    use structopt::StructOpt;
    let options = cli::Options::from_args();

    let source = if options.files.len() > 0 {
        Source::Files(options.files)
    } else if atty::is(atty::Stream::Stdin) {
        Source::recursive()?
    } else {
        Source::Stdin
    };

    App::new(
        source,
        Replacer::new(
            options.find,
            options.replace_with,
            options.literal_mode,
            options.flags,
            options.replacements,
        )?,
    )
    .run(options.preview)?;
    Ok(())
}
