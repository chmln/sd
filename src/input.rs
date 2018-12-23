use {crate::Error, std::{fs::File, io::prelude::*}};

pub(crate) enum Source {
    Stdin,
    File(String),
}

impl Source {
    pub(crate) fn from(maybe_path: Option<String>) -> Self {
        maybe_path
            .map(Source::File)
            .unwrap_or_else(|| Source::Stdin)
    }

    pub(crate) fn into_stream(&self) -> Result<Stream, Error> {
        match self {
            Source::Stdin => {
                let mut buffer = String::new();
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();

                handle.read_to_string(&mut buffer)?;
                Ok(Stream::new(buffer))
            }
            Source::File(path) => {
                let file = File::open(path)?;
                let mut buf_reader = std::io::BufReader::new(file);
                let mut buffer = String::new();
                buf_reader.read_to_string(&mut buffer)?;
                Ok(Stream::new(buffer))
            }
        }
    }

}

pub(crate) struct Stream {
    data: String,
}

impl Stream {
    pub(crate) fn new(data: String) -> Self {
        Self { data }
    }

    pub(crate) fn replace(
        &mut self,
        is_regex: bool,
        look_for: &str,
        replace_with: &str,
    ) -> Result<(), crate::Error> {
        if is_regex {
            self.data = regex::Regex::new(look_for)?
                .replace_all(&self.data, replace_with)
                .to_string()
        } else {
            self.data = self.data.replace(look_for, replace_with)
        }
        Ok(())
    }

    // Output based on input.
    // When dealing with a file, transform it in-place.
    // Otherwise, pipe to stdout.
    pub(crate) fn output(self, source: &Source) -> Result<(), crate::Error> {
        match source {
            Source::File(path) => {
                Ok(std::fs::write(path, self.data)?)
            },
            Source::Stdin => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                handle.write(self.data.as_bytes())?;
                Ok(())
            }
        }
    }
}
