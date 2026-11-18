//! Shared data model for the `cmt` CLI.

use std::path::PathBuf;

/// How a comment is closed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Level 1 (`·`): close by moving the line into the sidecar log.
    Sidecar,
    /// Level 2 (`··`): close by appending `(closed)` to the comment line.
    Inline,
    /// More than two dots: not a valid marker.
    Invalid,
}

impl Mode {
    pub fn of_level(level: usize) -> Mode {
        match level {
            1 => Mode::Sidecar,
            2 => Mode::Inline,
            _ => Mode::Invalid,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mode::Sidecar => "sidecar",
            Mode::Inline => "inline",
            Mode::Invalid => "invalid",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Open,
    Closed,
    Malformed,
}

impl State {
    pub fn label(self) -> &'static str {
        match self {
            State::Open => "open",
            State::Closed => "closed",
            State::Malformed => "malformed",
        }
    }

    pub fn is_open(self) -> bool {
        self == State::Open
    }
}

#[derive(Clone, Debug)]
pub struct Comment {
    /// Sequential id inside the file, starting at 1.
    pub id: usize,
    pub path: PathBuf,
    /// 1-based line number.
    pub line: usize,
    /// Number of leading `·` markers.
    pub level: usize,
    pub mode: Mode,
    pub state: State,
    /// Comment text with the `·` markers and surrounding whitespace removed.
    pub content: String,
    /// Leading whitespace of the original line (preserved when rewriting).
    pub indent: usize,
    /// Nearest preceding ATX heading, if any.
    pub section: Option<String>,
    pub closed_token: Option<String>,
    /// FNV-1a hash of the normalized content, stable across runs.
    pub hash: String,
    pub issues: Vec<&'static str>,
    pub context_before: Vec<(usize, String)>,
    pub context_after: Vec<(usize, String)>,
}

#[derive(Clone, Debug)]
pub struct FileScan {
    pub path: PathBuf,
    pub encoding: &'static str,
    pub eol: &'static str,
    pub lossy: bool,
    pub comments: Vec<Comment>,
}
