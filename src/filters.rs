use regex::Regex;

/// This is the most naive type of filter: One required full copy of the test
/// per filter and it requires valid utf8
pub trait Filter {
    fn process(&self, input: &str) -> String;
}

struct LinePicker {
    start: usize,
    end: usize,
}

impl Filter for LinePicker {
    fn process(&self, input: &str) -> String {
        input
            .split_inclusive('\n')
            .skip(self.start)
            .take(self.end)
            .collect()
    }
}

struct Leftpad;
impl Filter for Leftpad {
    fn process(&self, input: &str) -> String {
        input
            .split_inclusive('\n')
            .map(|x| x.trim_start())
            .collect()
    }
}

struct RemoveNewlines;
impl Filter for RemoveNewlines {
    fn process(&self, input: &str) -> String {
        input.lines().collect()
    }
}

struct ReplacerLiteral {
    pattern: String,
    replace_with: String,
    max: usize,
}

impl Filter for ReplacerLiteral {
    fn process(&self, input: &str) -> String {
        let regex = Regex::new(&regex::escape(&self.pattern)).unwrap();
        regex
            .replacen(input, self.max, &self.replace_with)
            .into_owned()
    }
}

struct ReplacerRegex1 {
    pattern: String,
    replace_with: String,
    max: usize,
}

impl Filter for ReplacerRegex1 {
    fn process(&self, input: &str) -> String {
        Regex::new(&self.pattern)
            .unwrap() // Just panic: The user entered an invalid regex
            .replacen(&input, self.max, &self.replace_with)
            .into_owned()
    }
}

struct ReplacerRegex2 {
    pattern: Regex, //Pass it already a validated pattern
    replace_with: String,
    max: usize,
}

impl Filter for ReplacerRegex2 {
    fn process(&self, input: &str) -> String {
        self.pattern
            .replacen(&input, self.max, &self.replace_with)
            .into_owned()
    }
}

pub fn process_filters1(
    mut input: String,
    filters: Vec<Box<dyn Filter>>,
) -> String {
    for f in filters {
        input = f.process(&input);
    }
    input
}

//======= Line by line mode

trait LineFilter {
    fn process(&self, input: &mut Vec<String>);
}

struct LinePicker2 {
    start: usize,
    end: usize,
}

impl LineFilter for LinePicker2 {
    fn process(&self, input: &mut Vec<String>){
        *input = input.drain(self.start..self.end).collect();
    }
}

struct Leftpad2;
impl LineFilter for Leftpad2 {
    fn process(&self, input: &mut Vec<String>) {
        for x in input {
            *x = x.trim_start().to_string();
        }
    }
}


enum LineFilterWrapper {
    Line(Box<dyn LineFilter>),
    Filter(Box<dyn Filter>),
}


fn process_filters_3(
    mut input: String,
    filters: Vec<LineFilterWrapper>,
) -> String {
    todo!()
}

//======= I'm not in love with this but it might be neccessary for some filters?

trait FailableFilter {
    fn process(
        &self,
        input: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

struct ReplacerRegex3 {
    pattern: String,
    replace_with: String,
    max: usize,
}

impl FailableFilter for ReplacerRegex3 {
    fn process(
        &self,
        input: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Regex::new(&self.pattern)?
            .replacen(&input, self.max, &self.replace_with)
            .into_owned())
    }
}

enum FilterWrapper {
    Unfailable(Box<dyn Filter>),
    Failable(Box<dyn FailableFilter>),
}

fn process_filters2(
    mut input: String,
    filters: Vec<FilterWrapper>,
) -> Result<String, Box<dyn std::error::Error>> {
    for f in filters {
        input = match f {
            FilterWrapper::Unfailable(f) => f.process(&input),
            FilterWrapper::Failable(f) => f.process(&input)?,
        }
    }
    Ok(input)
}

#[cfg(test)]
mod test {
    use super::*;

    static TEST_STR: &str = r#"skip
    foo
    bar
    baz
    skip
    skip"#;

    #[test]
    fn linepicker() {
        let out = LinePicker { start: 1, end: 3 }.process(TEST_STR);
        assert_eq!(out, "    foo\n    bar\n    baz\n");
    }

    #[test]
    fn leftpad() {
        let out = Leftpad.process(TEST_STR);
        assert_eq!(out, "skip\nfoo\nbar\nbaz\nskip\nskip");
    }

    #[test]
    fn chain() {
        let out = Leftpad.process(
            LinePicker { start: 1, end: 3 }.process(TEST_STR).as_str(),
        );
        assert_eq!(out, "foo\nbar\nbaz\n");
    }

    #[test]
    fn thing() {
        let filters: Vec<Box<dyn Filter>> = vec![
            Box::new(Leftpad),
            Box::new(LinePicker { start: 1, end: 3 }),
            Box::new(ReplacerLiteral {
                pattern: "foo".to_string(),
                replace_with: "bar".to_string(),
                max: 0,
            }),
            Box::new(ReplacerRegex1 {
                pattern: "baz".to_string(),
                replace_with: "bar".to_string(),
                max: 0,
            }),
            Box::new(RemoveNewlines),
        ];

        let out = process_filters1(TEST_STR.to_string(), filters);
        assert_eq!(out, "barbarbar");
    }
}
