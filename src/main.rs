mod app;
mod error;
mod input;
pub(crate) mod utils;

pub(crate) use self::{
    error::Error,
    input::{Replacer, Source},
    utils::Result,
};

fn main() -> Result<()> {
    use structopt::StructOpt;
    let args = app::Options::from_args();
    if args.in_place {
        eprintln!(
            "Warning: --in-place is now redundant and will be removed in a \
             future release."
        );
    }
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
