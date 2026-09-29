mod cli;

use clap::Parser;
use std::{io::stdout, process};

use sd::{Replacer, Result, Source, process_sources};

/// On Windows, writes to `io::stdout()` go through `WriteConsoleW` when the
/// handle is a console, which rejects byte sequences that are not UTF-8.
/// Write through `WriteFile` instead by reusing the raw stdout handle as a
/// `File`, matching the byte-oriented behavior of other platforms.
#[cfg(windows)]
fn console_stdout() -> Option<std::mem::ManuallyDrop<std::fs::File>> {
    use std::io::IsTerminal;
    use std::os::windows::io::{AsRawHandle, FromRawHandle};

    if stdout().is_terminal() {
        // SAFETY: `as_raw_handle` returns the process-wide stdout handle,
        // which stays valid for the whole program. `ManuallyDrop` keeps the
        // `File` from closing it when dropped.
        unsafe {
            Some(std::mem::ManuallyDrop::new(std::fs::File::from_raw_handle(
                stdout().as_raw_handle(),
            )))
        }
    } else {
        None
    }
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
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
    #[cfg(windows)]
    let mut console_file = console_stdout();
    #[cfg(windows)]
    let writer: &mut dyn std::io::Write = match &mut console_file {
        Some(file) => &mut **file,
        None => &mut handle,
    };
    #[cfg(not(windows))]
    let writer: &mut dyn std::io::Write = &mut handle;

    process_sources(
        &replacer,
        &sources,
        options.preview,
        !options.across,
        writer,
    )
}
