use {crate::{Error,Stream,Source}};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub(crate) struct App;

impl App {
    pub(crate) fn run() -> Result<(), Error> {
        use clap;
        let app = clap::App::new("re")
            .version(VERSION)
            .setting(clap::AppSettings::ColoredHelp)
            .setting(clap::AppSettings::NextLineHelp)
            .arg(
                clap::Arg::with_name("enable_regex")
                    .short("r")
                    .long("regex")
                    .required(false)
                    .takes_value(false)
                    .help("Enable regex"),
            )
            .arg(
                clap::Arg::with_name("input")
                    .short("i")
                    .required(false)
                    .takes_value(true)
                    .help("The path to file")
            )
            .arg(
                clap::Arg::with_name("find")
                    .help("The string or regexp (if --regex) to search for.")
                    .required(true)
                    .index(1),
            )
            .arg(
                clap::Arg::with_name("replace_with")
                    .help("What to replace each match with. If regex is enabled, you may use captured values like $1, $2, etc.")
                    .required(true)
                    .index(2),
            );

        let matches = app.get_matches();

        let file_path = matches.value_of("input").map(|p| p.to_string());
        let find = matches.value_of("find").unwrap();
        let replace_with = matches.value_of("replace_with").unwrap();
        let is_regex = matches.occurrences_of("enable_regex") == 1;

        let source = Source::from(file_path);
        let mut stream: Stream = (&source).into_stream()?;
        stream.replace(is_regex, find, replace_with)?;

        // replace file in-place, or pipe to stdout
        stream.output(&source)
    }
}
