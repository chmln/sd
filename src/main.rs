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
    app::run()
}
