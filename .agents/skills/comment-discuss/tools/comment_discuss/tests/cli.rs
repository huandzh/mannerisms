//! Binary-level tests: run the real `cmt` executable against fixture files.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_cmt")
}

fn workdir(tag: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(".temp")
        .join(format!("cmt-it-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create workdir");
    dir
}

fn run(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(bin()).args(args).output().expect("spawn cmt");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn write_bytes(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes).expect("write fixture");
}

#[test]
fn close_level1_keeps_crlf_and_writes_log() {
    let dir = workdir("crlf");
    let doc = dir.join("doc.md");
    write_bytes(&doc, "a\r\n\r\n· 改标题\r\n\r\nb\r\n".as_bytes());

    let (code, stdout, stderr) = run(&["close", doc.to_str().unwrap(), "--all"]);
    assert_eq!(code, 0, "stdout={stdout} stderr={stderr}");
    assert!(stdout.contains("migrated"));

    let after = std::fs::read(&doc).unwrap();
    let after = String::from_utf8(after).unwrap();
    assert!(!after.contains('\u{00B7}'), "comment line must be gone: {after:?}");
    assert_eq!(after, "a\r\n\r\nb\r\n");
    assert!(
        !after.replace("\r\n", "").contains('\n'),
        "CRLF must be preserved: {after:?}"
    );

    let log = std::fs::read_to_string(dir.join("doc.discuss-log.md")).unwrap();
    assert!(log.starts_with("<!-- comment-discuss log v1"));
    assert!(log.contains("改标题"));
}

#[test]
fn validate_strict_flags_already_logged_comment() {
    let dir = workdir("strict");
    let doc = dir.join("doc.md");
    write_bytes(&doc, "· 重复的意见\n".as_bytes());
    assert_eq!(run(&["close", doc.to_str().unwrap(), "--all"]).0, 0);

    // The same comment is written back into the document by mistake.
    let mut content = std::fs::read_to_string(&doc).unwrap();
    content.push_str("· 重复的意见\n");
    std::fs::write(&doc, content).unwrap();

    let (code, stdout, _) = run(&["validate", doc.to_str().unwrap(), "--strict", "--json"]);
    assert_eq!(code, 3);
    assert!(stdout.contains("already-logged"), "{stdout}");
}

#[test]
fn scan_reports_levels_and_skips_fenced_blocks() {
    let dir = workdir("scan");
    let doc = dir.join("doc.md");
    write_bytes(
        &doc,
        "```\n· 代码块里的示例\n```\n  · 缩进评论\n·· 行内评论 (closed)\n".as_bytes(),
    );

    let (code, stdout, _) = run(&["scan", doc.to_str().unwrap(), "--json"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("\"comments\": 2"), "{stdout}");
    // The fenced example may appear as context, but it must never be a comment.
    assert!(
        !stdout.contains("\"content\": \"代码块里的示例\""),
        "{stdout}"
    );
    assert!(stdout.contains("\"indent\": 2"), "{stdout}");
    assert!(stdout.contains("\"state\": \"closed\""), "{stdout}");

    let (code, _, _) = run(&["scan", doc.to_str().unwrap(), "--exit-code"]);
    assert_eq!(code, 2);
}

#[test]
fn scan_directory_skips_sidecar_logs() {
    let dir = workdir("walk");
    write_bytes(&dir.join("a.md"), "· 待办\n".as_bytes());
    write_bytes(
        &dir.join("a.discuss-log.md"),
        "<!-- comment-discuss log v1 · sidecar for a.md -->\n\n> · 待办\n".as_bytes(),
    );

    let (code, stdout, _) = run(&["scan", dir.to_str().unwrap(), "--json", "--quiet"]);
    assert_eq!(code, 0);
    assert_eq!(stdout.matches("\"path\"").count(), 1, "{stdout}");
    assert!(stdout.contains("a.md"), "{stdout}");
    assert!(!stdout.contains("a.discuss-log.md"), "{stdout}");
}

#[test]
fn close_by_id_and_refuses_malformed_without_force() {
    let dir = workdir("byid");
    let doc = dir.join("doc.md");
    write_bytes(&doc, "· 第一条\n\n··· 三条杠\n".as_bytes());

    let (code, stdout, _) = run(&["close", doc.to_str().unwrap(), "--id", "1"]);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("migrated"));

    // A malformed marker is never deleted silently when targeted directly.
    let (code, _, stderr) = run(&["close", doc.to_str().unwrap(), "--line", "2"]);
    assert_eq!(code, 1);
    assert!(stderr.contains("not a valid marker"), "{stderr}");

    let (code, stdout, _) = run(&["close", doc.to_str().unwrap(), "--all", "--force"]);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("removed"));
}

#[test]
fn utf16_file_is_read_and_rewritten_in_its_own_encoding() {
    let dir = workdir("utf16");
    let doc = dir.join("doc.md");
    let mut bytes = vec![0xFF, 0xFE];
    for unit in "· 改一下\r\n正文\r\n".encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    write_bytes(&doc, &bytes);

    let (code, stdout, stderr) = run(&["scan", doc.to_str().unwrap(), "--json"]);
    assert_eq!(code, 0, "{stderr}");
    assert!(stdout.contains("\"encoding\": \"utf-16le\""), "{stdout}");

    let (code, stdout, stderr) = run(&["close", doc.to_str().unwrap(), "--all"]);
    assert_eq!(code, 0, "{stderr}{stdout}");

    let after = std::fs::read(&doc).unwrap();
    assert_eq!(&after[..2], &[0xFF, 0xFE], "BOM must survive");
    let units: Vec<u16> = after[2..]
        .chunks_exact(2)
        .map(|p| u16::from_le_bytes([p[0], p[1]]))
        .collect();
    assert_eq!(String::from_utf16(&units).unwrap(), "正文\r\n");
}
