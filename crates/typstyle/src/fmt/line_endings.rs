use typst_syntax::{Source, SyntaxNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LineEnding {
    Lf,
    CrLf,
}

impl LineEnding {
    /// Preserve CRLF only when every ASCII newline in the input uses CRLF.
    /// Inputs without newlines or with mixed line endings fall back to LF.
    pub(super) fn detect_consistent(content: &str) -> Self {
        let bytes = content.as_bytes();
        let mut saw_crlf = false;
        let mut index = 0;

        while index < bytes.len() {
            match bytes[index] {
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => {
                    saw_crlf = true;
                    index += 2;
                }
                b'\r' | b'\n' => return Self::Lf,
                _ => index += 1,
            }
        }

        if saw_crlf { Self::CrLf } else { Self::Lf }
    }

    fn detect_first(content: &str) -> Self {
        let bytes = content.as_bytes();
        let mut index = 0;

        while index < bytes.len() {
            match bytes[index] {
                b'\r' if bytes.get(index + 1) == Some(&b'\n') => return Self::CrLf,
                b'\r' | b'\n' => return Self::Lf,
                _ => index += 1,
            }
        }

        Self::Lf
    }

    pub(super) fn apply(self, content: String) -> String {
        match self {
            Self::Lf => content,
            Self::CrLf => {
                let mut converted = String::with_capacity(content.len());
                push_crlf(&mut converted, &content);
                converted
            }
        }
    }
}

// This tested alternative is intentionally disconnected from the default conservative policy.
#[allow(dead_code)]
pub(super) fn apply_first_line_ending_to_trivia(original: &str, formatted: String) -> String {
    match LineEnding::detect_first(original) {
        LineEnding::Lf => formatted,
        LineEnding::CrLf => {
            let source = Source::detached(formatted);
            let mut converted = String::with_capacity(source.root().len());
            push_node_text(source.root(), &mut converted);
            converted
        }
    }
}

fn push_node_text(node: &SyntaxNode, output: &mut String) {
    let mut children = node.children();
    if let Some(first) = children.next() {
        push_node_text(first, output);
        for child in children {
            push_node_text(child, output);
        }
    } else if node.kind().is_trivia() {
        push_crlf(output, node.leaf_text());
    } else {
        output.push_str(node.leaf_text());
    }
}

fn push_crlf(output: &mut String, content: &str) {
    let mut previous_was_cr = false;
    for ch in content.chars() {
        if ch == '\n' && !previous_was_cr {
            output.push('\r');
        }
        output.push(ch);
        previous_was_cr = ch == '\r';
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_only_consistent_crlf() {
        assert_eq!(
            LineEnding::detect_consistent("a\r\nb\r\n"),
            LineEnding::CrLf
        );
        assert_eq!(LineEnding::detect_consistent("a\nb\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("a\rb\r"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("a\r\nb\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("a\nb\r\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("a\r\nb\r"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_consistent("no newline"), LineEnding::Lf);

        assert_eq!(
            LineEnding::detect_consistent("a\u{000b}\u{000c}\u{0085}\u{2028}\u{2029}\r\n"),
            LineEnding::CrLf
        );
    }

    #[test]
    fn crlf_conversion_only_rewrites_bare_lf() {
        let content = "a\nb\r\nc\rd\u{000b}e\u{000c}f\u{0085}g\u{2028}h\u{2029}i";
        let converted = LineEnding::CrLf.apply(content.to_owned());

        assert_eq!(
            converted,
            "a\r\nb\r\nc\rd\u{000b}e\u{000c}f\u{0085}g\u{2028}h\u{2029}i"
        );
        assert_eq!(LineEnding::Lf.apply(content.to_owned()), content);
    }

    #[test]
    fn detects_first_ascii_physical_line_ending() {
        assert_eq!(LineEnding::detect_first("a\r\nb\n"), LineEnding::CrLf);
        assert_eq!(LineEnding::detect_first("a\nb\r\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_first("a\rb\r\n"), LineEnding::Lf);
        assert_eq!(LineEnding::detect_first("no newline"), LineEnding::Lf);

        assert_eq!(
            LineEnding::detect_first("a\u{000b}\u{000c}\u{0085}\u{2028}\u{2029}\r\nb"),
            LineEnding::CrLf
        );
        assert_eq!(
            LineEnding::detect_first("a\u{000b}\u{000c}\u{0085}\u{2028}\u{2029}b"),
            LineEnding::Lf
        );
    }

    #[test]
    fn first_crlf_strategy_only_rewrites_trivia() {
        let formatted = concat!(
            "/* block\ncomment */\n",
            "#let string = \"a\nb\"\n",
            "#let raw = ```a\nb```\n",
            "#let separator = \"a\u{2028}b\"\n",
        );

        let converted =
            apply_first_line_ending_to_trivia("first\r\nsecond\n", formatted.to_owned());

        assert_eq!(
            converted,
            concat!(
                "/* block\r\ncomment */\r\n",
                "#let string = \"a\nb\"\r\n",
                "#let raw = ```a\nb```\r\n",
                "#let separator = \"a\u{2028}b\"\r\n",
            )
        );
    }

    #[test]
    fn first_lf_strategy_is_identity() {
        let formatted = "#let value = \"a\nb\"\n".to_owned();

        assert_eq!(
            apply_first_line_ending_to_trivia("first\nsecond\r\n", formatted.clone()),
            formatted
        );
    }
}
