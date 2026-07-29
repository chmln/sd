mod cli;

use clap::Parser;
use std::{io::stdout, process};

use sd::{Replacer, Result, Source, process_sources};

/// Exit status for `--must-match` when the pattern matched no input.
///
/// Distinct from the status used for errors, so that a script can tell "your
/// pattern found nothing" apart from "sd could not do its job".
const NO_MATCH_EXIT_CODE: i32 = 2;

fn main() {
    match try_main() {
        Ok(true) => {}
        Ok(false) => process::exit(NO_MATCH_EXIT_CODE),
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    }
}

/// Returns whether the run satisfied `--must-match`; always true without it.
fn try_main() -> Result<bool> {
    let options = cli::Options::parse();

    let replacer = Replacer::new(
        options.find,
        options.replace_with,
        options.literal_mode,
        options.flags,
        options.replacements,
    )?;

    let sources = if !options.files.is_empty() {
        Source::from_paths(options.files)
    } else {
        Ok(Source::from_stdin())
    };
    let sources = sources?;

    let mut handle = stdout().lock();
    let matched = process_sources(
        &replacer,
        &sources,
        options.preview,
        !options.across,
        &mut handle,
    )?;

    // Any replacements have already been written by this point. --must-match
    // reports that nothing matched, it does not make the run conditional.
    Ok(matched || !options.must_match)
}
