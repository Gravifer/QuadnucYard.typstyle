#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineEnding {
    Lf,
    CrLf,
}

impl LineEnding {
    /// Preserve CRLF only when every ASCII newline in the input uses CRLF.
    /// Inputs without newlines or with mixed line endings retain formatter output.
    fn detect_consistent(content: &str) -> Self {
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

    fn apply(self, content: String) -> String {
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

pub(super) fn apply_crlf_preserve(original: &str, formatted: String) -> String {
    LineEnding::detect_consistent(original).apply(formatted)
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
