mod cli;
mod error;
mod input;

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

    let sources = if !options.files.is_empty() {
        Source::from_paths(options.files)
    } else {
        Source::from_stdin()
    };

    App::new(
        sources,
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
