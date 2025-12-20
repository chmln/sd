use std::{
    env,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};

mod generate;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate static assets
    Gen,
}

fn main() {
    let Cli { command } = Cli::parse();

    env::set_current_dir(project_root()).unwrap();

    match command {
        Commands::Gen => generate::generate(),
    }
}

fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}
