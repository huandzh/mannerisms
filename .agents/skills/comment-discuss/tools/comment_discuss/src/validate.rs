//! Validation: every comment must reach its protocol-defined closed state.

use std::path::{Path, PathBuf};

use crate::close;
use crate::config::Config;
use crate::model::State;
use crate::scan::{self, ScanOptions};

#[derive(Clone, Debug)]
pub struct Violation {
    pub path: PathBuf,
    pub line: usize,
    pub comment_id: Option<usize>,
    /// Machine-stable code, e.g. `open-comment-l1`.
    pub code: &'static str,
    /// 1 = warning, 2 = open work, 3 = protocol error.
    pub severity: u8,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub files: usize,
    pub comments: usize,
    pub open: usize,
    pub closed: usize,
    pub malformed: usize,
    pub violations: Vec<Violation>,
}

impl Outcome {
    pub fn max_severity(&self) -> u8 {
        self.violations.iter().map(|v| v.severity).max().unwrap_or(0)
    }

    /// 0 = clean, 2 = open comments remain, 3 = protocol error.
    pub fn exit_code(&self) -> i32 {
        match self.max_severity() {
            0 | 1 => 0,
            2 => 2,
            _ => 3,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ValidateOptions {
    /// Cross-check the sidecar log and flag duplicate comments.
    pub strict: bool,
    /// Treat still-open comments as warnings instead of failures.
    pub allow_open: bool,
    pub allow_lossy: bool,
}

pub fn validate_file(
    path: &Path,
    cfg: &Config,
    opts: &ValidateOptions,
) -> Result<Outcome, String> {
    let mut scan_opts = ScanOptions::from_config(cfg);
    scan_opts.before = 0;
    scan_opts.after = 0;
    let file_scan = scan::scan_file(path, &scan_opts, cfg)?;

    let mut out = Outcome {
        files: 1,
        comments: file_scan.comments.len(),
        ..Default::default()
    };

    if file_scan.lossy && !opts.allow_lossy {
        out.violations.push(Violation {
            path: path.to_path_buf(),
            line: 0,
            comment_id: None,
            code: "lossy-encoding",
            severity: 3,
            message: "file is not valid UTF-8; close/rewrite is unsafe (convert it first or pass --allow-lossy)"
                .to_string(),
        });
    }

    let log_path = close::default_log_path(path, cfg);
    let log_records = if opts.strict && log_path.is_file() {
        close::read_log(&log_path)
    } else {
        Vec::new()
    };
    let open_severity = if opts.allow_open { 1 } else { 2 };

    for comment in &file_scan.comments {
        match comment.state {
            State::Open => out.open += 1,
            State::Closed => out.closed += 1,
            State::Malformed => out.malformed += 1,
        }

        let mut push = |code: &'static str, severity: u8, message: String| {
            out.violations.push(Violation {
                path: path.to_path_buf(),
                line: comment.line,
                comment_id: Some(comment.id),
                code,
                severity,
                message,
            });
        };

        match (comment.state, comment.level) {
            (State::Malformed, _) => {
                let detail = comment.issues.join(", ");
                push(
                    "malformed-marker",
                    3,
                    format!("marker is malformed ({}): use `·` or `··` followed by text", detail),
                );
            }
            (State::Open, 1) => push(
                "open-comment-l1",
                open_severity,
                format!(
                    "level-1 comment is still open: address it, then `cmt close {} --line {}`",
                    display(path),
                    comment.line
                ),
            ),
            (State::Open, 2) => push(
                "open-comment-l2",
                open_severity,
                format!(
                    "level-2 comment is still open: address it, then `cmt close {} --line {}` to mark it (closed)",
                    display(path),
                    comment.line
                ),
            ),
            (State::Open, _) => push(
                "open-comment-unknown-level",
                open_severity,
                "comment level is unknown; use `·` or `··`".to_string(),
            ),
            (State::Closed, 1) => push(
                "l1-inline-closed",
                2,
                "level-1 comment was marked closed inline; level 1 belongs in the sidecar log (`cmt close`), use `··` when the discussion must stay in the file"
                    .to_string(),
            ),
            _ => {}
        }

        if opts.strict {
            if let Some(record) = log_records.iter().find(|r| r.hash == comment.hash) {
                push(
                    "already-logged",
                    3,
                    format!(
                        "sidecar log `{}` already records this comment (hash {}); delete the line instead of re-closing it",
                        display(&log_path),
                        record.hash
                    ),
                );
            }
        }
    }

    if opts.strict {
        // Identical normalized text twice in one file is almost always a copy-paste slip.
        for comment in &file_scan.comments {
            let twins = file_scan
                .comments
                .iter()
                .filter(|other| other.hash == comment.hash && other.line != comment.line)
                .count();
            if twins > 0 {
                out.violations.push(Violation {
                    path: path.to_path_buf(),
                    line: comment.line,
                    comment_id: Some(comment.id),
                    code: "duplicate-comment",
                    severity: 3,
                    message: format!(
                        "duplicate of another comment on the same file ({} identical), hash {}",
                        twins, comment.hash
                    ),
                });
            }
        }
    }

    Ok(out)
}

fn display(path: &Path) -> String {
    path.display().to_string()
}
