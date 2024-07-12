use std::{io::BufRead, path::PathBuf};

use crate::language_type::LanguageType;

#[derive(Debug)]
pub struct Counter {
    pub comment: usize,
    pub blank: usize,
    pub code: usize,
}

impl std::ops::Add for Counter {
    type Output = Counter;

    fn add(self, rhs: Counter) -> Counter {
        Counter {
            comment: self.comment + rhs.comment,
            blank: self.blank + rhs.blank,
            code: self.code + rhs.code,
        }
    }
}

impl Counter {
    pub fn new() -> Counter {
        Counter {
            comment: 0,
            blank: 0,
            code: 0,
        }
    }

    pub fn lines(&self) -> usize {
        self.comment + self.blank + self.code
    }
}

pub fn count_lines(file_path: &PathBuf, language_type: LanguageType) -> std::io::Result<Counter> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);

    let mut counts = Counter::new();
    let mut in_multiline_comment = false;
    let mut in_verbatim_quote = false;
    let mut in_quote = false;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            counts.blank += 1;
        } else if in_multiline_comment {
            counts.comment += 1;
            for (_, end) in language_type.multi_line_comments() {
                if line.ends_with(end) {
                    in_multiline_comment = false;
                    break;
                }
            }
        } else if in_verbatim_quote {
            counts.code += 1;
            for (_, end) in language_type.verbatim_quotes() {
                if line.contains(end) {
                    in_verbatim_quote = false;
                    break;
                }
            }
        } else if in_quote {
            in_quote = false;
        } else {
            let mut is_comment = false;

            for comment in language_type.line_comment() {
                if line.starts_with(comment) {
                    counts.comment += 1;
                    is_comment = true;
                    break;
                }
            }

            if !is_comment {
                for (start, end) in language_type.multi_line_comments() {
                    if line.starts_with(start) {
                        counts.comment += 1;
                        in_multiline_comment = !line.ends_with(end);
                        is_comment = true;
                        break;
                    }
                }
            }

            if !is_comment {
                for (start, _) in language_type.verbatim_quotes() {
                    if line.contains(start) {
                        in_verbatim_quote = true;
                        break;
                    }
                }
            }

            if !is_comment {
                counts.code += 1;
            }
        }
    }

    Ok(counts)
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};

    use crate::counter::count_lines;

    #[test]
    fn test_count1() {
        let counts = count_lines(
            &PathBuf::from_str(".test/example.cpp").unwrap(),
            crate::language_type::LanguageType::Cpp,
        )
        .unwrap();
        assert_eq!(counts.code, 5);
        assert_eq!(counts.comment, 9);
        assert_eq!(counts.blank, 3);
    }
}
