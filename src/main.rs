mod app;
mod error;
mod input;

pub(crate) use {
    crate::error::Error,
    crate::input::{Source, Stream},
};

fn main() -> Result<(), ()> {
    app::App::run().map_err(|e| eprintln!("{}", e))
}
