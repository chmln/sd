mod app;
mod error;
mod input;
pub(crate) mod utils;

pub(crate) use {
    crate::error::Error,
    crate::input::{Source, Stream},
};

fn main() -> Result<(), Error> {
    app::run()
}
