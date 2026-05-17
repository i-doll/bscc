use crate::config::LanguageConfig;

#[derive(Debug, Clone, Default)]
pub struct Counts {
    pub lines: u32,
    pub code: u32,
    pub comments: u32,
    pub blanks: u32,
}

enum State {
    Code,
    LineComment,
    BlockComment(Vec<u8>),
    StringLit { end: Vec<u8>, allow_escape: bool },
}

pub fn count(cfg: &LanguageConfig, source: &[u8]) -> Counts {
    let mut counts = Counts::default();
    let mut state = State::Code;
    let mut saw_code = false;
    let mut saw_comment = false;
    let mut i = 0usize;

    while i < source.len() {
        let b = source[i];

        // Newline is the line classifier and resets per-line state regardless
        // of which state we're in.
        if b == b'\n' {
            counts.lines += 1;
            if saw_code {
                counts.code += 1;
            } else if saw_comment {
                counts.comments += 1;
            } else {
                counts.blanks += 1;
            }
            saw_code = false;
            saw_comment = false;
            if matches!(state, State::LineComment) {
                state = State::Code;
            }
            i += 1;
            continue;
        }

        // Take ownership of state so each arm can return a new one without
        // borrowing conflicts.
        let cur = std::mem::replace(&mut state, State::Code);
        state = match cur {
            State::Code => step_code(source, &mut i, b, cfg, &mut saw_code, &mut saw_comment),
            State::LineComment => {
                i += 1;
                State::LineComment
            }
            State::BlockComment(end) => {
                if starts_with_at(source, i, &end) {
                    saw_comment = true;
                    i += end.len();
                    State::Code
                } else {
                    if !b.is_ascii_whitespace() {
                        saw_comment = true;
                    }
                    i += 1;
                    State::BlockComment(end)
                }
            }
            State::StringLit { end, allow_escape } => {
                if allow_escape && b == b'\\' && i + 1 < source.len() {
                    saw_code = true;
                    i += 2;
                    State::StringLit { end, allow_escape }
                } else if starts_with_at(source, i, &end) {
                    saw_code = true;
                    i += end.len();
                    State::Code
                } else {
                    saw_code = true;
                    i += 1;
                    State::StringLit { end, allow_escape }
                }
            }
        };
    }

    // File without a trailing newline still has one final line.
    if !source.is_empty() && source[source.len() - 1] != b'\n' {
        counts.lines += 1;
        if saw_code {
            counts.code += 1;
        } else if saw_comment {
            counts.comments += 1;
        } else {
            counts.blanks += 1;
        }
    }

    counts
}

fn step_code(
    src: &[u8],
    i: &mut usize,
    b: u8,
    cfg: &LanguageConfig,
    saw_code: &mut bool,
    saw_comment: &mut bool,
) -> State {
    // Block comments first — their start often shares a prefix with the
    // line-comment marker (e.g. `/` for `/*` and `//`).
    for pair in &cfg.block_comments {
        let start = pair[0].as_bytes();
        if starts_with_at(src, *i, start) {
            *saw_comment = true;
            *i += start.len();
            return State::BlockComment(pair[1].as_bytes().to_vec());
        }
    }
    for prefix in &cfg.line_comments {
        let p = prefix.as_bytes();
        if starts_with_at(src, *i, p) {
            *saw_comment = true;
            *i += p.len();
            return State::LineComment;
        }
    }
    for pair in &cfg.strings {
        let start = pair[0].as_bytes();
        if starts_with_at(src, *i, start) {
            *saw_code = true;
            *i += start.len();
            return State::StringLit {
                end: pair[1].as_bytes().to_vec(),
                allow_escape: true,
            };
        }
    }
    if !b.is_ascii_whitespace() {
        *saw_code = true;
    }
    *i += 1;
    State::Code
}

fn starts_with_at(haystack: &[u8], pos: usize, needle: &[u8]) -> bool {
    haystack.get(pos..pos + needle.len()) == Some(needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_c_like() -> LanguageConfig {
        LanguageConfig {
            name: "test".into(),
            extensions: vec!["x".into()],
            filenames: vec![],
            line_comments: vec!["//".into()],
            block_comments: vec![["/*".into(), "*/".into()]],
            strings: vec![["\"".into(), "\"".into()]],
        }
    }

    #[test]
    fn empty_file() {
        let c = count(&cfg_c_like(), b"");
        assert_eq!(c.lines, 0);
    }

    #[test]
    fn blanks_only() {
        let c = count(&cfg_c_like(), b"\n\n\n");
        assert_eq!(c.lines, 3);
        assert_eq!(c.blanks, 3);
        assert_eq!(c.code, 0);
        assert_eq!(c.comments, 0);
    }

    #[test]
    fn no_trailing_newline_counts_line() {
        let c = count(&cfg_c_like(), b"int x = 1;");
        assert_eq!(c.lines, 1);
        assert_eq!(c.code, 1);
    }

    #[test]
    fn line_comment_classifies_line_as_comment() {
        let c = count(&cfg_c_like(), b"// hi\n");
        assert_eq!(c.lines, 1);
        assert_eq!(c.comments, 1);
        assert_eq!(c.code, 0);
    }

    #[test]
    fn code_then_line_comment_is_code() {
        let c = count(&cfg_c_like(), b"int x; // trail\n");
        assert_eq!(c.code, 1);
        assert_eq!(c.comments, 0);
    }

    #[test]
    fn block_comment_spans_lines() {
        let c = count(&cfg_c_like(), b"/* a\n b */\n");
        assert_eq!(c.lines, 2);
        assert_eq!(c.comments, 2);
        assert_eq!(c.code, 0);
    }

    #[test]
    fn middle_line_of_block_comment_is_comment_not_blank() {
        // Three-line block comment: opening on line 1, body on line 2,
        // closing on line 3. All three lines should count as comment.
        let c = count(&cfg_c_like(), b"/*\n a\n */\n");
        assert_eq!(c.lines, 3);
        assert_eq!(c.comments, 3);
        assert_eq!(c.blanks, 0);
    }

    #[test]
    fn string_with_comment_marker_inside_is_code() {
        let c = count(&cfg_c_like(), b"\"// not a comment\"\n");
        assert_eq!(c.code, 1);
        assert_eq!(c.comments, 0);
    }

    #[test]
    fn escaped_quote_in_string() {
        let c = count(&cfg_c_like(), b"\"a\\\"b\";\n");
        assert_eq!(c.code, 1);
    }
}
