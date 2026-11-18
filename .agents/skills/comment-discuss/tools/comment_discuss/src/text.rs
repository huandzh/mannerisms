//! Encoding-aware text IO.
//!
//! Read: UTF-8 (with or without BOM), UTF-16 LE/BE with BOM, and a lossy UTF-8
//! fallback for anything else (reported so callers can refuse to rewrite it).
//! Write: the source encoding is preserved line for line.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

pub const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
const UTF16LE_BOM: &[u8] = &[0xFF, 0xFE];
const UTF16BE_BOM: &[u8] = &[0xFE, 0xFF];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Encoding {
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
    /// Input was not valid UTF-8; decoded with replacement characters.
    Lossy,
}

impl Encoding {
    pub fn label(self) -> &'static str {
        match self {
            Encoding::Utf8 => "utf-8",
            Encoding::Utf8Bom => "utf-8-bom",
            Encoding::Utf16Le => "utf-16le",
            Encoding::Utf16Be => "utf-16be",
            Encoding::Lossy => "lossy-utf8",
        }
    }

    /// True when rewriting the text could corrupt it.
    pub fn is_lossy(self) -> bool {
        matches!(self, Encoding::Lossy)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Eol {
    Lf,
    Crlf,
    Mixed,
    /// No line terminator at all (single-line file).
    Bare,
}

impl Eol {
    pub fn label(self) -> &'static str {
        match self {
            Eol::Lf => "lf",
            Eol::Crlf => "crlf",
            Eol::Mixed => "mixed",
            Eol::Bare => "none",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextFile {
    pub encoding: Encoding,
    pub eol: Eol,
    pub text: String,
    pub lossy: bool,
}

pub fn read_text(path: &Path) -> io::Result<TextFile> {
    let mut buf = Vec::new();
    let mut f = File::open(path)?;
    f.read_to_end(&mut buf)?;
    Ok(decode(&buf))
}

pub fn decode(buf: &[u8]) -> TextFile {
    let (encoding, text, lossy) = if let Some(rest) = buf.strip_prefix(UTF8_BOM) {
        match std::str::from_utf8(rest) {
            Ok(s) => (Encoding::Utf8Bom, s.to_string(), false),
            Err(_) => (Encoding::Lossy, String::from_utf8_lossy(rest).into_owned(), true),
        }
    } else if let Some(rest) = buf.strip_prefix(UTF16LE_BOM) {
        (Encoding::Utf16Le, decode_utf16(rest, true), false)
    } else if let Some(rest) = buf.strip_prefix(UTF16BE_BOM) {
        (Encoding::Utf16Be, decode_utf16(rest, false), false)
    } else {
        match std::str::from_utf8(buf) {
            Ok(s) => (Encoding::Utf8, s.to_string(), false),
            Err(_) => (Encoding::Lossy, String::from_utf8_lossy(buf).into_owned(), true),
        }
    };
    let eol = detect_eol(&text);
    TextFile {
        encoding,
        eol,
        text,
        lossy,
    }
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
    let mut units: Vec<u16> = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        units.push(if little_endian {
            u16::from_le_bytes([pair[0], pair[1]])
        } else {
            u16::from_be_bytes([pair[0], pair[1]])
        });
    }
    String::from_utf16_lossy(&units)
}

pub fn detect_eol(text: &str) -> Eol {
    let bytes = text.as_bytes();
    let mut crlf = 0usize;
    let mut lf = 0usize;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            if i > 0 && bytes[i - 1] == b'\r' {
                crlf += 1;
            } else {
                lf += 1;
            }
        }
    }
    match (crlf, lf) {
        (0, 0) => Eol::Bare,
        (0, _) => Eol::Lf,
        (_, 0) => Eol::Crlf,
        _ => Eol::Mixed,
    }
}

pub fn encode(tf: &TextFile, text: &str) -> Vec<u8> {
    match tf.encoding {
        Encoding::Utf8 | Encoding::Lossy => text.as_bytes().to_vec(),
        Encoding::Utf8Bom => {
            let mut out = Vec::with_capacity(text.len() + 3);
            out.extend_from_slice(UTF8_BOM);
            out.extend_from_slice(text.as_bytes());
            out
        }
        Encoding::Utf16Le | Encoding::Utf16Be => {
            let mut out = Vec::with_capacity(text.len() * 2 + 2);
            out.extend_from_slice(if tf.encoding == Encoding::Utf16Le {
                UTF16LE_BOM
            } else {
                UTF16BE_BOM
            });
            for unit in text.encode_utf16() {
                let b = if tf.encoding == Encoding::Utf16Le {
                    unit.to_le_bytes()
                } else {
                    unit.to_be_bytes()
                };
                out.extend_from_slice(&b);
            }
            out
        }
    }
}

/// Write via a sibling temp file + rename, so a crash never truncates the document.
pub fn write_atomic(path: &Path, tf: &TextFile, text: &str) -> io::Result<()> {
    let bytes = encode(tf, text);
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "cmt".to_string());
    let tmp = dir.join(format!(".{}.cmt-{}.tmp", name, std::process::id()));
    {
        let mut f = File::create(&tmp)?;
        f.write_all(&bytes)?;
        f.flush()?;
        let _ = f.sync_all();
    }
    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}

/// Split into lines, keeping each line's terminator so a rewrite is byte-exact.
pub fn split_lines(text: &str) -> Vec<&str> {
    text.split_inclusive('\n').collect()
}

/// Drop the trailing `\n` / `\r\n` from one line.
pub fn strip_eol(line: &str) -> &str {
    let s = line.strip_suffix('\n').unwrap_or(line);
    s.strip_suffix('\r').unwrap_or(s)
}

/// Split one line into (body, terminator).
pub fn split_eol(line: &str) -> (&str, &str) {
    if let Some(body) = line.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = line.strip_suffix('\n') {
        (body, "\n")
    } else {
        (line, "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_utf8_and_detects_eol() {
        let tf = decode("a\nb\n".as_bytes());
        assert_eq!(tf.encoding, Encoding::Utf8);
        assert_eq!(tf.eol, Eol::Lf);
        assert!(!tf.lossy);
    }

    #[test]
    fn keeps_utf8_bom_on_roundtrip() {
        let mut raw = UTF8_BOM.to_vec();
        raw.extend_from_slice("标题\n".as_bytes());
        let tf = decode(&raw);
        assert_eq!(tf.encoding, Encoding::Utf8Bom);
        assert_eq!(encode(&tf, &tf.text), raw);
    }

    #[test]
    fn roundtrips_utf16le() {
        let mut raw = UTF16LE_BOM.to_vec();
        for unit in "中文\n".encode_utf16() {
            raw.extend_from_slice(&unit.to_le_bytes());
        }
        let tf = decode(&raw);
        assert_eq!(tf.encoding, Encoding::Utf16Le);
        assert_eq!(tf.text, "中文\n");
        assert_eq!(encode(&tf, &tf.text), raw);
    }

    #[test]
    fn detects_mixed_and_crlf() {
        assert_eq!(detect_eol("a\r\nb\r\n"), Eol::Crlf);
        assert_eq!(detect_eol("a\r\nb\n"), Eol::Mixed);
        assert_eq!(detect_eol("single"), Eol::Bare);
    }

    #[test]
    fn invalid_utf8_is_flagged_lossy() {
        let tf = decode(&[0xFF, 0x41, 0x42]);
        assert!(tf.lossy);
        assert_eq!(tf.encoding, Encoding::Lossy);
    }

    #[test]
    fn split_lines_keeps_terminators() {
        assert_eq!(split_lines("a\r\nb"), vec!["a\r\n", "b"]);
        assert_eq!(strip_eol("a\r\n"), "a");
        assert_eq!(split_eol("a\r\n"), ("a", "\r\n"));
        assert_eq!(split_eol("a"), ("a", ""));
    }
}
