include!("src/app.rs");

fn main() {
    use std::{env::var, fs, str::FromStr};
    use structopt::clap::Shell;

    let mut app = Options::clap();
    let out_dir = var("SHELL_COMPLETIONS_DIR").or(var("OUT_DIR")).unwrap();
    let bin_name = var("CARGO_PKG_NAME").unwrap();

    fs::create_dir_all(&out_dir).unwrap();

    Shell::variants().into_iter().for_each(|shell| {
        app.gen_completions(
            bin_name.as_ref(),
            Shell::from_str(shell).unwrap(),
            &out_dir,
        );
    });
}
