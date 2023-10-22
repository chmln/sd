#[derive(Debug)]
pub struct Pair {
    regex: regex::Regex,
    rep: String,
}

#[derive(Debug)]
pub enum Opt {
    ReplaceLineIfMatch,
    AnotherOption,
}

pub fn replace_line<P: AsRef<[Pair]>, S: AsRef<str>>(
    pairs: P,
    opts: Vec<Opt>,
    content: S,
) -> Vec<String> {
    let pairs = pairs.as_ref();
    let content = content.as_ref();

    let mut lines = content.lines().map(|x| x.to_owned()).collect::<Vec<_>>();

    for opt in opts {
        match opt {
            Opt::ReplaceLineIfMatch => replace_line_if_match(pairs, &mut lines),
            Opt::AnotherOption => other(pairs, &mut lines),
        }
    }
    lines
}

fn replace_line_if_match(pairs: &[Pair], lines: &mut Vec<String>) {
    for line in lines.iter_mut() {
        for Pair { regex, rep } in pairs {
            if regex.is_match(&line) {
                *line = rep.clone();
            }
        }
    }
}

fn other(pairs: &[Pair], lines: &mut Vec<String>) {
    unimplemented!()
}
