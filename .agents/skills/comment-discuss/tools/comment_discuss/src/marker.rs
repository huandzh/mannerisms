//! Marker grammar: `·` (level 1, sidecar) and `··` (level 2, inline close).

use crate::text;

pub const DOT: char = '\u{00B7}';
const DOT_LEN: usize = 2; // UTF-8 length of U+00B7

#[derive(Clone, Debug)]
pub struct Parsed {
    pub level: usize,
    pub indent: usize,
    /// Text after the dots, trimmed.
    pub content: String,
    /// The closure token found inside the content, normalized to lowercase.
    pub closed_token: Option<String>,
}

/// A discussion comment is a line whose first non-blank character is `·`.
pub fn parse(line: &str, closed_tokens: &[String]) -> Option<Parsed> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    if !trimmed.starts_with(DOT) {
        return None;
    }
    let indent = line.len() - trimmed.len();
    let level = trimmed.chars().take_while(|c| *c == DOT).count();
    let rest = &trimmed[level * DOT_LEN..];
    let content = rest.trim().to_string();
    let closed_token = find_closed(&content, closed_tokens);
    Some(Parsed {
        level,
        indent,
        content,
        closed_token,
    })
}

/// Find a closure marker such as `(closed)` / `（closed）`, case-insensitively.
pub fn find_closed(text: &str, tokens: &[String]) -> Option<String> {
    if tokens.is_empty() || text.is_empty() {
        return None;
    }
    let lower = text.to_lowercase();
    for token in tokens {
        let t = token.to_lowercase();
        if lower.contains(&format!("({})", t)) || lower.contains(&format!("（{}）", t)) {
            return Some(t);
        }
    }
    None
}

/// Append `(token)` to a comment line, before its line terminator.
pub fn append_closed(line: &str, token: &str) -> String {
    let (body, eol) = text::split_eol(line);
    let body = body.trim_end_matches([' ', '\t']);
    format!("{} ({}){}", body, token, eol)
}

/// True for the characters dropped when comparing comment texts.
fn ignorable(c: char) -> bool {
    c.is_whitespace()
        || c.is_ascii_punctuation()
        || matches!(
            c,
            '\u{00B7}' // ·
                | '，' | '。' | '、' | '；' | '：' | '？' | '！'
                | '（' | '）' | '“' | '”' | '‘' | '’' | '—' | '…' | '《' | '》'
                | '「' | '」' | '【' | '】'
        )
}

/// Normalized form used for hashing and sidecar-log matching.
pub fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| !ignorable(*c))
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Stable 64-bit FNV-1a of the normalized text, hex encoded.
pub fn hash64(text: &str) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in normalize(text).as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    format!("{:016x}", h)
}

/// `#`-prefixed heading text, if the line is an ATX heading.
pub fn heading_text(line: &str) -> Option<String> {
    let s = line.trim_start_matches([' ', '\t']);
    let level = s.bytes().take_while(|b| *b == b'#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &s[level..];
    if rest.is_empty() {
        return None;
    }
    if !(rest.starts_with(' ') || rest.starts_with('\t')) {
        return None;
    }
    let t = rest.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Opening fence of a fenced code block: (fence char, run length).
pub fn fence_open(line: &str) -> Option<(char, usize)> {
    let s = line.trim_start_matches([' ', '\t']);
    let ch = s.chars().next()?;
    if ch != '`' && ch != '~' {
        return None;
    }
    let run = s.chars().take_while(|c| *c == ch).count();
    if run < 3 {
        return None;
    }
    Some((ch, run))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> Vec<String> {
        vec!["closed".to_string()]
    }

    #[test]
    fn parses_levels() {
        let p = parse("· 主题要调整\r\n", &tokens()).unwrap();
        assert_eq!(p.level, 1);
        assert_eq!(p.content, "主题要调整");
        assert_eq!(p.closed_token, None);

        let p = parse("  ·· 需要补充上下文", &tokens()).unwrap();
        assert_eq!(p.level, 2);
        assert_eq!(p.indent, 2);
        assert_eq!(p.content, "需要补充上下文");
    }

    #[test]
    fn ignores_other_lines() {
        for line in ["·abc?" /* no: still a marker */, "- · item", "> · quoted", "text"] {
            if line.starts_with('·') {
                assert!(parse(line, &tokens()).is_some());
            } else {
                assert!(parse(line, &tokens()).is_none(), "{}", line);
            }
        }
    }

    #[test]
    fn detects_closure_tokens() {
        let p = parse("·· 已重写 (closed)", &tokens()).unwrap();
        assert_eq!(p.closed_token.as_deref(), Some("closed"));
        let p = parse("·· 已重写（CLOSED）", &tokens()).unwrap();
        assert_eq!(p.closed_token.as_deref(), Some("closed"));
        assert!(parse("·· 还没好", &tokens()).unwrap().closed_token.is_none());
    }

    #[test]
    fn appends_before_eol() {
        assert_eq!(append_closed("·· 好 (x)  \r\n", "closed"), "·· 好 (x) (closed)\r\n");
        assert_eq!(append_closed("·· 好", "closed"), "·· 好 (closed)");
    }

    #[test]
    fn hash_ignores_whitespace_and_punctuation() {
        assert_eq!(hash64("主题，要调整"), hash64("主题  要调整"));
        assert_ne!(hash64("主题要调整"), hash64("主题要补充"));
    }

    #[test]
    fn headings_and_fences() {
        assert_eq!(heading_text("### 洞察2").as_deref(), Some("洞察2"));
        assert_eq!(heading_text("#nope"), None);
        assert_eq!(heading_text("####### 7").as_deref(), None);
        assert_eq!(fence_open("```rust").map(|(c, n)| (c, n)), Some(('`', 3)));
        assert_eq!(fence_open("~~~~").map(|(c, n)| (c, n)), Some(('~', 4)));
        assert_eq!(fence_open("``"), None);
    }
}
