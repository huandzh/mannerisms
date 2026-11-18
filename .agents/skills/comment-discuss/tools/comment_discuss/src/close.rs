//! Closing comments: level 1 moves the line to the sidecar log, level 2 marks
//! the line `(closed)` in place.

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::marker;
use crate::model::{Comment, Mode, State};
use crate::scan::{self, ScanOptions};
use crate::text;
use crate::timeutil;

/// Header written at the top of every sidecar log, also used to recognise logs.
pub const LOG_HEADER_PREFIX: &str = "<!-- comment-discuss log v1";
const HASH_PREFIX: &str = "<!-- cmt:";

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub line: usize,
    pub section: Option<String>,
    pub hash: String,
    pub content: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LogRecord {
    pub hash: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct CloseOptions {
    pub lines: Vec<usize>,
    pub ids: Vec<usize>,
    pub all: bool,
    pub levels: Vec<usize>,
    pub note: Option<String>,
    pub log: Option<PathBuf>,
    pub dry_run: bool,
    pub allow_lossy: bool,
    pub force: bool,
    pub token: String,
}

impl Default for CloseOptions {
    fn default() -> Self {
        CloseOptions {
            lines: Vec::new(),
            ids: Vec::new(),
            all: false,
            levels: Vec::new(),
            note: None,
            log: None,
            dry_run: false,
            allow_lossy: false,
            force: false,
            token: "closed".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Action {
    pub path: PathBuf,
    pub line: usize,
    pub level: usize,
    pub mode: &'static str,
    pub action: &'static str,
    pub log: Option<PathBuf>,
    pub hash: String,
    pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct CloseOutcome {
    pub actions: Vec<Action>,
    pub logs: Vec<PathBuf>,
    pub files: usize,
}

/// Default sidecar path: `report.md` -> `report.discuss-log.md`.
pub fn default_log_path(doc: &Path, cfg: &Config) -> PathBuf {
    let stem = doc
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "document".to_string());
    doc.with_file_name(format!("{}{}", stem, cfg.log_suffix))
}

pub fn close_file(path: &Path, cfg: &Config, opts: &CloseOptions) -> Result<CloseOutcome, String> {
    let tf = text::read_text(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    if tf.encoding.is_lossy() && !opts.allow_lossy && !opts.force {
        return Err(format!(
            "{}: not valid UTF-8, refusing to rewrite (pass --allow-lossy to force)",
            path.display()
        ));
    }

    let mut scan_opts = ScanOptions::from_config(cfg);
    scan_opts.before = 0;
    scan_opts.after = 0;
    let file_scan = scan::scan_file(path, &scan_opts, cfg)?;
    let targets = select(&file_scan.comments, opts)?;
    if targets.is_empty() {
        return Ok(CloseOutcome::default());
    }

    let log_path = opts
        .log
        .clone()
        .unwrap_or_else(|| default_log_path(path, cfg));
    let existing = read_log(&log_path);
    let existing_hashes: Vec<&str> = existing.iter().map(|r| r.hash.as_str()).collect();

    // Byte-exact line vector: terminators travel with their line.
    let mut lines: Vec<String> = text::split_lines(&tf.text)
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    let now = timeutil::now_utc();
    let mut actions: Vec<Action> = Vec::new();
    let mut entries: Vec<LogEntry> = Vec::new();
    let mut indices: Vec<usize> = targets.iter().map(|c| c.line).collect();
    indices.sort_unstable();
    indices.dedup();

    // Rewrite from the bottom up so earlier line numbers stay valid.
    for index in indices.into_iter().rev() {
        let comment = match targets.iter().find(|c| c.line == index) {
            Some(c) => *c,
            None => continue,
        };
        let slot = index - 1;

        if comment.level == 1 {
            if slot >= lines.len() {
                return Err(format!("{}: line {} is out of range", path.display(), index));
            }
            lines.remove(slot);
            tidy_gap(&mut lines, slot);
            let already = existing_hashes.contains(&comment.hash.as_str());
            if !already {
                entries.push(LogEntry {
                    timestamp: now.clone(),
                    line: comment.line,
                    section: comment.section.clone(),
                    hash: comment.hash.clone(),
                    content: comment.content.clone(),
                    note: opts.note.clone(),
                });
            }
            actions.push(Action {
                path: comment.path.clone(),
                line: comment.line,
                level: 1,
                mode: Mode::Sidecar.label(),
                action: if already { "migrated-dedup" } else { "migrated" },
                log: Some(log_path.clone()),
                hash: comment.hash.clone(),
                content: comment.content.clone(),
            });
        } else if comment.level == 2 {
            if comment.state == State::Closed {
                actions.push(Action {
                    path: path.to_path_buf(),
                    line: comment.line,
                    level: 2,
                    mode: Mode::Inline.label(),
                    action: "already-closed",
                    log: None,
                    hash: comment.hash.clone(),
                    content: comment.content.clone(),
                });
                continue;
            }
            if slot >= lines.len() {
                return Err(format!("{}: line {} is out of range", path.display(), index));
            }
            lines[slot] = marker::append_closed(&lines[slot], &opts.token);
            actions.push(Action {
                path: path.to_path_buf(),
                line: comment.line,
                level: 2,
                mode: Mode::Inline.label(),
                action: "marked-closed",
                log: None,
                hash: comment.hash.clone(),
                content: comment.content.clone(),
            });
        } else if opts.force {
            if slot < lines.len() {
                lines.remove(slot);
            }
            actions.push(Action {
                path: path.to_path_buf(),
                line: comment.line,
                level: comment.level,
                mode: Mode::Invalid.label(),
                action: "removed",
                log: None,
                hash: comment.hash.clone(),
                content: comment.content.clone(),
            });
        } else {
            return Err(format!(
                "{}:{}: {} dots is not a valid marker (use `·` or `··`, or --force to delete)",
                path.display(),
                comment.line,
                comment.level
            ));
        }
    }

    entries.sort_by_key(|e| e.line);
    actions.sort_by_key(|a| a.line);

    if !opts.dry_run {
        if !entries.is_empty() {
            append_log(&log_path, path, &entries)?;
        }
        let new_text: String = lines.concat();
        if new_text != tf.text {
            text::write_atomic(path, &tf, &new_text)
                .map_err(|e| format!("{}: {}", path.display(), e))?;
        }
    }

    Ok(CloseOutcome {
        actions,
        logs: if entries.is_empty() {
            Vec::new()
        } else {
            vec![log_path]
        },
        files: 1,
    })
}

fn select<'a>(
    comments: &'a [Comment],
    opts: &CloseOptions,
) -> Result<Vec<&'a Comment>, String> {
    let mut chosen: Vec<&Comment> = Vec::new();

    if opts.all {
        chosen.extend(
            comments
                .iter()
                .filter(|c| c.state.is_open() || (opts.force && c.state == State::Malformed)),
        );
    }
    for line in &opts.lines {
        match comments.iter().find(|c| c.line == *line) {
            Some(c) => chosen.push(c),
            None => return Err(format!("no discussion comment on line {}", line)),
        }
    }
    for id in &opts.ids {
        match comments.iter().find(|c| c.id == *id) {
            Some(c) => chosen.push(c),
            None => return Err(format!("no discussion comment with id {}", id)),
        }
    }

    if !opts.all && opts.lines.is_empty() && opts.ids.is_empty() {
        return Err("nothing selected: use --all, --line N or --id N".to_string());
    }
    if !opts.levels.is_empty() {
        chosen.retain(|c| opts.levels.contains(&c.level));
    }
    chosen.sort_by_key(|c| c.line);
    chosen.dedup_by_key(|c| c.line);
    Ok(chosen)
}

/// Removing a comment line between two blank lines would leave a double gap;
/// collapse it so the discuss file stays readable. Everything else is untouched.
fn tidy_gap(lines: &mut Vec<String>, removed_at: usize) {
    if removed_at == 0 || removed_at >= lines.len() {
        return;
    }
    let is_blank =
        |line: &str| text::strip_eol(line).trim().is_empty();
    if is_blank(&lines[removed_at - 1]) && is_blank(&lines[removed_at]) {
        lines.remove(removed_at);
    }
}

pub fn format_entry(entry: &LogEntry) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\n## {} · line {}",
        entry.timestamp, entry.line
    ));
    if let Some(section) = entry.section.as_deref() {
        if !section.is_empty() {
            out.push_str(&format!(" · {}", section));
        }
    }
    out.push_str("\n\n");
    out.push_str(&format!("{}{} -->\n", HASH_PREFIX, entry.hash));
    out.push_str(&format!("> · {}\n", entry.content));
    if let Some(note) = entry.note.as_deref() {
        let note = note.trim();
        if !note.is_empty() {
            out.push_str(&format!("> resolution: {}\n", note));
        }
    }
    out
}

fn header(doc: &Path) -> String {
    let name = doc
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    format!(
        "{} · sidecar for {} -->\n\n# Discussion log · {}\n\nLevel-1 comments (`·`) are moved here once they are addressed, so the discuss\nfile itself stays clean. Appended by `cmt close` (newest last); the\n`<!-- cmt:… -->` hashes are what `cmt validate --strict` reads back.\n",
        LOG_HEADER_PREFIX, name, name
    )
}

fn append_log(log: &Path, doc: &Path, entries: &[LogEntry]) -> Result<(), String> {
    let existed = log.exists();
    let mut body = String::new();
    if !existed {
        body.push_str(&header(doc));
    }
    for entry in entries {
        body.push_str(&format_entry(entry));
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .map_err(|e| format!("{}: {}", log.display(), e))?;
    file.write_all(body.as_bytes())
        .map_err(|e| format!("{}: {}", log.display(), e))?;
    Ok(())
}

/// Parse a sidecar log back into records (best effort, never fails).
pub fn read_log(log: &Path) -> Vec<LogRecord> {
    let text = match std::fs::read_to_string(log) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut records: Vec<LogRecord> = Vec::new();
    let mut pending: Option<LogRecord> = None;

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(HASH_PREFIX) {
            if let Some(record) = pending.take() {
                records.push(record);
            }
            let hash = rest
                .trim_start()
                .split(|c: char| c.is_whitespace() || c == '-')
                .next()
                .unwrap_or("")
                .to_string();
            pending = Some(LogRecord {
                hash,
                content: String::new(),
            });
        } else if let Some(record) = pending.as_mut() {
            if let Some(quoted) = trimmed.strip_prefix("> · ") {
                if !record.content.is_empty() {
                    record.content.push(' ');
                }
                record.content.push_str(quoted.trim());
            } else if trimmed.is_empty() {
                if !record.content.is_empty() {
                    records.push(pending.take().expect("pending record"));
                }
            }
        }
    }
    if let Some(record) = pending {
        if !record.content.is_empty() {
            records.push(record);
        }
    }
    records
}

pub fn close_paths(
    paths: &[PathBuf],
    cfg: &Config,
    opts: &CloseOptions,
) -> Result<CloseOutcome, String> {
    let mut total = CloseOutcome::default();
    for path in paths {
        if path.is_dir() {
            return Err(format!(
                "{}: close takes files, not directories (pipe `cmt scan <dir> --json` first)",
                path.display()
            ));
        }
        let outcome = close_file(path, cfg, opts)?;
        if outcome.files > 0 {
            total.files += outcome.files;
        }
        total.actions.extend(outcome.actions);
        for log in outcome.logs {
            if !total.logs.contains(&log) {
                total.logs.push(log);
            }
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join(".temp")
            .join(format!("cmt-test-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn closes_level2_in_place() {
        let dir = tmp_dir("l2");
        let doc = dir.join("doc.md");
        std::fs::write(&doc, "# A\n\n·· 补充上下文\n\n正文\n").unwrap();
        let outcome = close_file(&doc, &Config::default(), &CloseOptions {
            all: true,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(outcome.actions.len(), 1);
        assert_eq!(outcome.actions[0].action, "marked-closed");
        let after = text::read_text(&doc).unwrap().text;
        assert!(after.contains("·· 补充上下文 (closed)"));
    }

    #[test]
    fn closes_level1_into_sidecar_log() {
        let dir = tmp_dir("l1");
        let doc = dir.join("doc.md");
        std::fs::write(&doc, "# 洞察\n\n· 主题要调整\n\n正文\n").unwrap();
        let outcome = close_file(&doc, &Config::default(), &CloseOptions {
            all: true,
            note: Some("改为拒绝过度复杂".to_string()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(outcome.actions[0].action, "migrated");
        let after = text::read_text(&doc).unwrap().text;
        assert!(!after.contains('·'), "comment line must be gone: {after:?}");
        let log = dir.join("doc.discuss-log.md");
        assert!(log.exists());
        let log_text = std::fs::read_to_string(&log).unwrap();
        assert!(log_text.starts_with(LOG_HEADER_PREFIX));
        assert!(log_text.contains("主题要调整"));
        assert!(log_text.contains("resolution: 改为拒绝过度复杂"));
        let records = read_log(&log);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].content, "主题要调整");
    }

    #[test]
    fn collapses_blank_gap_left_by_removed_comment() {
        let dir = tmp_dir("gap");
        let doc = dir.join("doc.md");
        std::fs::write(&doc, "a\n\n· 改一下\n\nb\n").unwrap();
        close_file(&doc, &Config::default(), &CloseOptions {
            all: true,
            ..Default::default()
        })
        .unwrap();
        let after = text::read_text(&doc).unwrap().text;
        assert_eq!(after, "a\n\nb\n");
    }

    #[test]
    fn dry_run_writes_nothing() {
        let dir = tmp_dir("dry");
        let doc = dir.join("doc.md");
        let original = "# A\n\n· 改一下\n";
        std::fs::write(&doc, original).unwrap();
        let outcome = close_file(&doc, &Config::default(), &CloseOptions {
            all: true,
            dry_run: true,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(outcome.actions.len(), 1);
        assert_eq!(std::fs::read_to_string(&doc).unwrap(), original);
        assert!(!dir.join("doc.discuss-log.md").exists());
    }

    #[test]
    fn closing_twice_does_not_duplicate_log() {
        let dir = tmp_dir("twice");
        let doc = dir.join("doc.md");
        std::fs::write(&doc, "# A\n\n· 改一下\n").unwrap();
        let cfg = Config::default();
        close_file(&doc, &cfg, &CloseOptions { all: true, ..Default::default() }).unwrap();
        // Re-add the same comment; the log already records the hash.
        let mut content = text::read_text(&doc).unwrap().text;
        content.push_str("\n· 改一下\n");
        std::fs::write(&doc, content).unwrap();
        let outcome = close_file(&doc, &cfg, &CloseOptions { all: true, ..Default::default() }).unwrap();
        assert_eq!(outcome.actions[0].action, "migrated-dedup");
        let log = read_log(&dir.join("doc.discuss-log.md"));
        assert_eq!(log.len(), 1, "hash must be logged once");
    }
}
