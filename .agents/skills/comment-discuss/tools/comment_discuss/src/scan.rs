//! File discovery + single-pass comment scanning.

use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::marker;
use crate::model::{Comment, FileScan, Mode, State};
use crate::text;

pub const SKIP_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    ".venv",
    "venv",
    "node_modules",
    "target",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".temp",
    ".dsh",
    ".frontend-slides",
    "dist",
    "build",
    ".ipynb_checkpoints",
];

/// Log files carry `<!-- comment-discuss log` in their first lines and are
/// skipped by directory walks to avoid re-reading their quoted comments.
pub const LOG_MARKER: &str = "<!-- comment-discuss log";

#[derive(Clone, Debug)]
pub struct ScanOptions {
    pub before: usize,
    pub after: usize,
    pub skip_fences: bool,
    pub include_logs: bool,
    /// Empty means "every extension".
    pub extensions: Vec<String>,
}

impl ScanOptions {
    pub fn from_config(cfg: &Config) -> Self {
        ScanOptions {
            before: cfg.context,
            after: cfg.context,
            skip_fences: cfg.skip_fences,
            include_logs: false,
            extensions: cfg.extensions.clone(),
        }
    }
}

pub fn collect_files(roots: &[PathBuf], opts: &ScanOptions) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    for root in roots {
        if root.is_file() {
            out.push(root.clone());
        } else if root.is_dir() {
            walk(root, opts, &mut out);
        } else {
            return Err(format!("path not found: {}", root.display()));
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

fn walk(dir: &Path, opts: &ScanOptions, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            walk(&path, opts, out);
        } else if file_type.is_file() && extension_allowed(&path, &opts.extensions) {
            out.push(path);
        }
    }
}

pub fn extension_allowed(path: &Path, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return true;
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => extensions
            .iter()
            .any(|want| want.eq_ignore_ascii_case(ext)),
        None => false,
    }
}

/// Is this a sidecar log produced by `cmt close`?
pub fn is_log_file(path: &Path, cfg: &Config) -> bool {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if name.ends_with(&cfg.log_suffix) {
        return true;
    }
    false
}

pub fn scan_file(path: &Path, opts: &ScanOptions, cfg: &Config) -> Result<FileScan, String> {
    let tf = text::read_text(path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let md_like = matches!(ext.as_str(), "md" | "markdown" | "mdx" | "mdown" | "mkd");

    let lines = text::split_lines(&tf.text);
    let mut comments: Vec<Comment> = Vec::new();
    let mut section: Option<String> = None;
    let mut fence: Option<(char, usize)> = None;

    for (idx, raw) in lines.iter().enumerate() {
        let line_no = idx + 1;
        let body = text::strip_eol(raw);

        if md_like && opts.skip_fences {
            if let Some((ch, run)) = marker::fence_open(body) {
                match fence {
                    None => {
                        fence = Some((ch, run));
                        continue;
                    }
                    Some((open_ch, open_run)) if open_ch == ch && run >= open_run => {
                        fence = None;
                        continue;
                    }
                    Some(_) => continue,
                }
            }
            if fence.is_some() {
                continue;
            }
        }

        if let Some(heading) = marker::heading_text(body) {
            section = Some(heading);
            continue;
        }

        let parsed = match marker::parse(body, &cfg.closed_tokens) {
            Some(p) => p,
            None => continue,
        };

        let mut issues: Vec<&'static str> = Vec::new();
        if parsed.level > 2 {
            issues.push("too_many_dots");
        }
        if parsed.content.is_empty() {
            issues.push("empty_comment");
        }
        if parsed.level == 1 && parsed.closed_token.is_some() {
            issues.push("l1_marked_closed");
        }

        let state = if !issues.is_empty() && (parsed.level > 2 || parsed.content.is_empty()) {
            State::Malformed
        } else if parsed.closed_token.is_some() {
            State::Closed
        } else {
            State::Open
        };

        let before_from = line_no.saturating_sub(opts.before).max(1);
        let context_before = (before_from..line_no)
            .map(|ln| (ln, text::strip_eol(lines[ln - 1]).to_string()))
            .collect();
        let after_to = (line_no + opts.after).min(lines.len());
        let context_after = (line_no + 1..=after_to)
            .map(|ln| (ln, text::strip_eol(lines[ln - 1]).to_string()))
            .collect();

        comments.push(Comment {
            id: comments.len() + 1,
            path: path.to_path_buf(),
            line: line_no,
            level: parsed.level,
            mode: Mode::of_level(parsed.level),
            state,
            content: parsed.content.clone(),
            indent: parsed.indent,
            section: section.clone(),
            closed_token: parsed.closed_token.clone(),
            hash: marker::hash64(&parsed.content),
            issues,
            context_before,
            context_after,
        });
    }

    Ok(FileScan {
        path: path.to_path_buf(),
        encoding: tf.encoding.label(),
        eol: tf.eol.label(),
        lossy: tf.lossy,
        comments,
    })
}

/// Predicate shared by the `scan` filters.
pub fn matches(
    comment: &Comment,
    level: Option<usize>,
    open_only: bool,
    closed_only: bool,
    grep: Option<&str>,
) -> bool {
    if let Some(level) = level {
        if comment.level != level {
            return false;
        }
    }
    if open_only && !comment.state.is_open() {
        return false;
    }
    if closed_only && comment.state != State::Closed {
        return false;
    }
    match grep {
        None => true,
        Some(needle) => {
            let needle = needle.to_lowercase();
            comment.content.to_lowercase().contains(&needle)
        }
    }
}
