mod app;
mod error;
mod input;
pub(crate) mod utils;

pub(crate) use {
    crate::error::Error,
    crate::input::{Replacer, Source},
};

fn main() -> Result<(), Error> {
    app::run()
}
