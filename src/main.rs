mod app;
mod error;
mod input;
pub(crate) mod utils;

pub(crate) use self::input::{Replacer, Source};
pub(crate) use error::{Error, Result};

fn main() -> Result<()> {
    use structopt::StructOpt;
    let args = app::Options::from_args();

    let source = Source::from(args.files);
    let replacer = Replacer::new(
        args.find,
        args.replace_with,
        args.literal_mode,
        args.flags,
    )?;
    replacer.run(&source, !args.preview)?;
    Ok(())
}
