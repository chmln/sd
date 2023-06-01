pub mod input;
pub mod error;
pub mod replacer;
pub mod utils;

pub use self::input::Source;
pub use error::{Error, Result};
use input::App;
pub(crate) use replacer::Replacer;

pub struct ReplaceConf {
    pub source: Source,
    pub preview: bool,
    pub no_swap: bool,
    pub literal_mode: bool,
    pub flags: Option<String>,
    pub replacements: Option<usize>,
}

impl Default for ReplaceConf {
    fn default() -> Self {
        Self {
            source: Source::Stdin,
            preview: false,
            no_swap: false,
            literal_mode: false,
            flags: None,
            replacements: None,
        }
    }
}

/// For example:
/// 
/// ```no_run
/// replace("foo", "bar", ReplaceConf {
///     source: Source::with_files(vec!["./foo.md", "./bar.md", "./foobar.md"]),
///     ..ReplaceConf::default()
/// })
/// ```
pub fn replace(find: String, replace_with: String, replace_conf: ReplaceConf) -> Result<()> {
    App::new(
        replace_conf.source,
        Replacer::new(
            find,
            replace_with,
            replace_conf.literal_mode,
            replace_conf.flags,
            replace_conf.replacements,
            replace_conf.no_swap,
        )?,
    )
    .run(replace_conf.preview)?;
    Ok(())
}