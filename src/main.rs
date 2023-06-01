mod cli;

use clap::Parser;

use sd::{Result, Source, replace, ReplaceConf};

fn main() -> Result<()> {
    let options = cli::Options::parse();

    let source = if options.recursive {
        Source::recursive()?
    } else if !options.files.is_empty() {
        Source::Files(options.files)
    } else {
        Source::Stdin
    };

    replace(options.find, options.replace_with, ReplaceConf {
        source,
        preview: options.preview,
        no_swap: options.no_swap,
        literal_mode: options.literal_mode,
        flags: options.flags,
        replacements: options.replacements,
    })?;
    Ok(())
}
