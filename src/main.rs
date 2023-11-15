mod cli;
mod error;
mod input;
mod filters;

pub(crate) mod replacer;
pub(crate) mod utils;

use std::process;

pub(crate) use self::input::{App, Source};
use ansi_term::{Color, Style};
pub(crate) use error::{Error, Result};
use replacer::Replacer;

use clap::Parser;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}: {}", Style::from(Color::Red).bold().paint("error"), e);
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let options = cli::Options::parse();

    let source = if !options.files.is_empty() {
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
        )?,
    )
    .run(options.preview)?;
    Ok(())
}
