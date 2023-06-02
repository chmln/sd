pub mod error;
pub mod input;
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

pub struct ReplaceConfBuilder {
    inner: ReplaceConf,
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

impl ReplaceConf {
    pub fn builder() -> ReplaceConfBuilder {
        ReplaceConfBuilder {
            inner: Self::default(),
        }
    }
}

impl ReplaceConfBuilder {
    pub fn set_source(mut self, source: Source) -> Self {
        self.inner.source = source;
        self
    }

    pub fn build(self) -> ReplaceConf {
        self.inner
    }
}

/// For example:
///
/// ```no_run
/// use sd::{replace, ReplaceConf, Source};
/// 
/// replace("foo", "bar", ReplaceConf {
///     source: Source::with_files(vec!["./foo.md", "./bar.md", "./foobar.md"]),
///     ..ReplaceConf::default()
/// });
/// ```
pub fn replace<F, R>(
    find: F,
    replace_with: R,
    replace_conf: ReplaceConf,
) -> Result<()>
where
    F: Into<String>,
    R: Into<String>,
{
    App::new(
        replace_conf.source,
        Replacer::new(
            find.into(),
            replace_with.into(),
            replace_conf.literal_mode,
            replace_conf.flags,
            replace_conf.replacements,
            replace_conf.no_swap,
        )?,
    )
    .run(replace_conf.preview)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs::{File, self}, io::{Write, Read}};

    use super::*;

    #[test]
    fn it_works() {
        // Create files for test.
        for p in &vec!["./for-test-foo", "./for-test-bar"] {
            let mut f = File::create(p).unwrap();
            f.write_all(b"foo bar foz").unwrap();
        }

        // Run it.
        let replace_conf = ReplaceConf::builder()
                .set_source(Source::with_files(vec!["./for-test-foo", "./for-test-bar"]))
                .build();
        replace("foo", "bar", replace_conf).unwrap();

        // Assert and cleanup.
        for p in &vec!["./for-test-foo", "./for-test-bar"] {
            let mut f = File::open(p).unwrap();
            let mut buf = vec![];
            f.read_to_end(&mut buf).unwrap();
            assert_eq!(buf, b"bar bar foz");
            fs::remove_file(p).unwrap();
        }
    }
}
