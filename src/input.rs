use {
    crate::{utils, Error},
    regex::Regex,
    std::{fs::File, io::prelude::*},
};

pub(crate) enum Replacer<'a> {
    Regex(Regex, &'a str),
    Literal(&'a str, &'a str),
}

impl<'a> Replacer<'a> {
    pub(crate) fn new(look_for: &'a str, replace_with: &'a str, is_literal: bool) -> Result<Self, Error> {
        if is_literal {
            return Ok(Replacer::Literal(look_for, replace_with));
        }
        return Ok(Replacer::Regex(regex::Regex::new(look_for)?, replace_with));
    }

    pub(crate) fn replace(&self, source: &Source, in_place: bool) -> Result<(), Error> {
        let content = source.to_string()?;
        let replaced = match self {
            Replacer::Regex(regex, replace_with) => {
                let replaced = regex.replace_all(&content, *replace_with).to_string();
                utils::unescape(&replaced).unwrap_or_else(|| replaced)
            }
            Replacer::Literal(search, replace_with) => content.replace(search, replace_with),
        };

        Replacer::output(source, replaced, in_place)
    }

    fn output(source: &Source, data: String, in_place: bool) -> Result<(), Error> {
        use atomic_write::atomic_write;
        match (source, in_place) {
            (Source::Stdin, _) | (Source::File(_), false) => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                handle.write(data.as_bytes())?;
                Ok(())
            }
            (Source::File(path), true) => Ok(atomic_write(path, data)?),
        }
    }
}

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

    pub(crate) fn to_string(&self) -> Result<String, Error> {
        match self {
            Source::Stdin => {
                let mut buffer = String::new();
                let stdin = std::io::stdin();
                let mut handle = stdin.lock();
                handle.read_to_string(&mut buffer)?;
                Ok(buffer)
            }
            Source::File(path) => {
                let file = File::open(path)?;
                let mut buf_reader = std::io::BufReader::new(file);
                let mut buffer = String::new();
                buf_reader.read_to_string(&mut buffer)?;
                Ok(buffer)
            }
        }
    }
}
