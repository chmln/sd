mod sd {
    include!("../../src/cli.rs");
}
use sd::Options;

use std::{fs, path::Path};

use clap::{CommandFactory, ValueEnum};
use clap_complete::{generate_to, Shell};
use roff::{bold, roman, Roff};

pub fn gen() {
    let gen_dir = Path::new("gen");
    gen_shell(gen_dir);
    gen_man(gen_dir);
}

fn gen_shell(base_dir: &Path) {
    let completions_dir = base_dir.join("completions");
    fs::create_dir_all(&completions_dir).unwrap();

    let mut cmd = Options::command();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "sd", &completions_dir).unwrap();
    }
}

fn gen_man(base_dir: &Path) {
    let man_path = base_dir.join("sd.1");
    let cmd = Options::command();
    let mut buffer: Vec<u8> = Vec::new();

    let man = clap_mangen::Man::new(cmd);
    man.render_title(&mut buffer)
        .expect("failed to render title section");
    man.render_name_section(&mut buffer)
        .expect("failed to render name section");
    man.render_synopsis_section(&mut buffer)
        .expect("failed to render synopsis section");
    man.render_description_section(&mut buffer)
        .expect("failed to render description section");
    man.render_options_section(&mut buffer)
        .expect("failed to render options section");

    let statuses = [
        ("0", "Successful program execution."),
        ("1", "Unsuccessful program execution."),
        ("101", "The program panicked."),
    ];
    let mut sect = Roff::new();
    sect.control("SH", ["EXIT STATUS"]);
    for (code, reason) in statuses {
        sect.control("IP", [code]).text([roman(reason)]);
    }
    sect.to_writer(&mut buffer)
        .expect("failed to render exit status section");

    let examples = [
        // (description, command, result), result can be empty
        (
            "String-literal mode",
            "echo 'lots((([]))) of special chars' | sd -F '((([])))'",
            "lots of special chars",
        ),
        (
            "Regex use. Let's trim some trailing whitespace",
            "echo 'lorem ipsum 23   ' | sd '\\s+$' ''",
            "lorem ipsum 23",
        ),
        (
            "Indexed capture groups",
            r"echo 'cargo +nightly watch' | sd '(\w+)\s+\+(\w+)\s+(\w+)' 'cmd: $1, channel: $2, subcmd: $3'",
            "cmd: cargo, channel: nightly, subcmd: watch",
        ),
        (
            "Find & replace in file",
            r#"sd 'window.fetch' 'fetch' http.js"#,
            "",
        ),
        (
            "Find & replace from STDIN an emit to STDOUT",
            r#"sd 'window.fetch' 'fetch' < http.js"#,
            "",
        ),
    ];
    let mut sect = Roff::new();
    sect.control("SH", ["EXAMPLES"]);
    for (desc, command, result) in examples {
        sect.control("TP", [])
            .text([roman(desc)])
            .text([bold(format!("$ {}", command))])
            .control("br", [])
            .text([roman(result)]);
    }
    sect.to_writer(&mut buffer)
        .expect("failed to render example section");

    std::fs::write(man_path, buffer).expect("failed to write manpage");
}
