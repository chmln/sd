mod cli;
mod error;
mod input;

pub(crate) mod replacer;
pub(crate) mod utils;

pub(crate) use self::input::{App, Source};
pub(crate) use error::{Error, Result};
use replacer::Replacer;

use clap::Parser;

fn main() -> Result<()> {
    let options = cli::Options::parse();

    let source = if options.recursive {
        Source::recursive()?
    } else if !options.files.is_empty() {
        Source::Files(options.files)
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
            options.extra,
        )?,
    )
    .run(options.preview)?;
    Ok(())
}
