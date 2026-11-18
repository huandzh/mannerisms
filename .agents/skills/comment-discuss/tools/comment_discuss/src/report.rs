//! Output rendering: human-readable and `--json`.

use std::fmt::Write as _;

use crate::close::CloseOutcome;
use crate::model::{FileScan, State};
use crate::validate::Outcome as ValidateOutcome;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, Default)]
pub struct RenderOpts {
    pub summary_only: bool,
    pub quiet: bool,
}

pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn opt_str(value: Option<&str>) -> String {
    match value {
        Some(s) => esc(s),
        None => "null".to_string(),
    }
}

fn level_dots(level: usize) -> String {
    std::iter::repeat('\u{00B7}').take(level).collect()
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn pad(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - len))
    }
}

fn summary_counts(files: &[FileScan]) -> (usize, usize, usize, usize) {
    let mut comments = 0;
    let mut open = 0;
    let mut closed = 0;
    let mut malformed = 0;
    for file in files {
        for c in &file.comments {
            comments += 1;
            match c.state {
                State::Open => open += 1,
                State::Closed => closed += 1,
                State::Malformed => malformed += 1,
            }
        }
    }
    (comments, open, closed, malformed)
}

// ---------------------------------------------------------------- scan

pub fn scan_human(files: &[FileScan], opts: RenderOpts) -> String {
    let (comments, open, closed, malformed) = summary_counts(files);
    let mut out = String::new();

    let _ = writeln!(
        out,
        "cmt scan · {} file(s) · {} comment(s) (open {} · closed {} · malformed {})",
        files.len(),
        comments,
        open,
        closed,
        malformed
    );

    if opts.quiet {
        return out;
    }
    if comments == 0 {
        out.push_str("no discussion comments found\n");
        return out;
    }

    for file in files {
        if file.comments.is_empty() {
            continue;
        }
        let _ = writeln!(
            out,
            "\n{}  [{} · {}]",
            file.path.display(),
            file.encoding,
            file.eol
        );
        if file.lossy {
            out.push_str("  warning: not valid UTF-8 (decoded lossily); close is disabled\n");
        }
        let _ = writeln!(
            out,
            "  {} {} {} {} {}",
            pad("id", 3),
            pad("line", 5),
            pad("lvl", 3),
            pad("state", 10),
            "comment"
        );
        for c in &file.comments {
            let section = c.section.as_deref().unwrap_or("-");
            let _ = writeln!(
                out,
                "  {} {} {} {} {} · {}",
                pad(&c.id.to_string(), 3),
                pad(&c.line.to_string(), 5),
                pad(&level_dots(c.level), 3),
                pad(c.state.label(), 10),
                truncate_chars(section, 24),
                truncate_chars(&c.content, 70)
            );
        }
    }

    if opts.summary_only {
        return out;
    }

    for file in files {
        for c in &file.comments {
            let _ = writeln!(
                out,
                "\n[{}] {}:{}  L{} {}  {}  hash={}",
                c.id,
                file.path.display(),
                c.line,
                c.level,
                c.mode.label(),
                c.state.label(),
                c.hash
            );
            let _ = writeln!(
                out,
                "    section: {}",
                c.section.as_deref().unwrap_or("-")
            );
            let _ = writeln!(out, "    comment: {}", c.content);
            if !c.issues.is_empty() {
                let _ = writeln!(out, "    issues : {}", c.issues.join(", "));
            }
            out.push('\n');
            for (line_no, text) in &c.context_before {
                let _ = writeln!(out, "    {:>5} | {}", line_no, truncate_chars(text, 200));
            }
            let _ = writeln!(
                out,
                "  >>{:>5} | {} {}",
                c.line,
                level_dots(c.level),
                truncate_chars(&c.content, 200)
            );
            for (line_no, text) in &c.context_after {
                let _ = writeln!(out, "    {:>5} | {}", line_no, truncate_chars(text, 200));
            }
        }
    }
    out
}

pub fn scan_json(files: &[FileScan]) -> String {
    let (comments, open, closed, malformed) = summary_counts(files);
    let mut out = String::new();
    out.push_str("{\n  \"command\": \"scan\",\n");
    let _ = writeln!(out, "  \"version\": {},", esc(VERSION));
    out.push_str("  \"files\": [\n");
    for (fi, file) in files.iter().enumerate() {
        out.push_str("    {\n");
        let _ = writeln!(out, "      \"path\": {},", esc(&file.path.display().to_string()));
        let _ = writeln!(out, "      \"encoding\": {},", esc(file.encoding));
        let _ = writeln!(out, "      \"eol\": {},", esc(file.eol));
        let _ = writeln!(out, "      \"lossy\": {},", file.lossy);
        out.push_str("      \"comments\": [\n");
        for (ci, c) in file.comments.iter().enumerate() {
            out.push_str("        {");
            let _ = write!(out, "\"id\": {}, ", c.id);
            let _ = write!(out, "\"line\": {}, ", c.line);
            let _ = write!(out, "\"level\": {}, ", c.level);
            let _ = write!(out, "\"mode\": {}, ", esc(c.mode.label()));
            let _ = write!(out, "\"state\": {}, ", esc(c.state.label()));
            let _ = write!(out, "\"hash\": {}, ", esc(&c.hash));
            let _ = write!(out, "\"section\": {}, ", opt_str(c.section.as_deref()));
            let _ = write!(out, "\"closed_token\": {}, ", opt_str(c.closed_token.as_deref()));
            let _ = write!(out, "\"indent\": {}, ", c.indent);
            let issues: Vec<String> = c.issues.iter().map(|i| esc(i)).collect();
            let _ = write!(out, "\"issues\": [{}], ", issues.join(","));
            let _ = write!(out, "\"content\": {}, ", esc(&c.content));
            let _ = write!(out, "\"context_before\": {}", context_json(&c.context_before));
            let _ = write!(out, ", \"context_after\": {}", context_json(&c.context_after));
            out.push('}');
            if ci + 1 < file.comments.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("      ]\n    }");
        if fi + 1 < files.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");
    let _ = writeln!(
        out,
        "  \"summary\": {{\"files\": {}, \"comments\": {}, \"open\": {}, \"closed\": {}, \"malformed\": {}}}",
        files.len(),
        comments,
        open,
        closed,
        malformed
    );
    out.push_str("}\n");
    out
}

fn context_json(lines: &[(usize, String)]) -> String {
    let body: Vec<String> = lines
        .iter()
        .map(|(n, t)| format!("{{\"line\": {}, \"text\": {}}}", n, esc(t)))
        .collect();
    format!("[{}]", body.join(", "))
}

// ------------------------------------------------------------ validate

pub fn validate_human(outcome: &ValidateOutcome, opts: RenderOpts) -> String {
    let mut out = String::new();
    let verdict = if outcome.exit_code() == 0 { "PASS" } else { "FAIL" };
    let _ = writeln!(
        out,
        "cmt validate · {} file(s) · {} comment(s) (open {} · closed {} · malformed {}) · {}",
        outcome.files,
        outcome.comments,
        outcome.open,
        outcome.closed,
        outcome.malformed,
        verdict
    );

    if outcome.violations.is_empty() {
        if !opts.quiet {
            out.push_str("all comments are closed or moved to the sidecar log\n");
        }
        return out;
    }
    if opts.quiet {
        let _ = writeln!(out, "{} violation(s)", outcome.violations.len());
        return out;
    }

    let mut violations = outcome.violations.clone();
    violations.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.line.cmp(&b.line))
            .then(a.code.cmp(b.code))
    });
    for v in &violations {
        let tag = match v.severity {
            1 => "WARN",
            2 => "OPEN",
            _ => "FAIL",
        };
        let _ = writeln!(
            out,
            "  {}  {}:{}  [{}]  {}",
            tag,
            v.path.display(),
            v.line,
            v.code,
            v.message
        );
    }
    out
}

pub fn validate_json(outcome: &ValidateOutcome) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"command\": \"validate\",\n");
    let _ = writeln!(out, "  \"version\": {},", esc(VERSION));
    let _ = writeln!(out, "  \"ok\": {},", outcome.exit_code() == 0);
    let _ = writeln!(out, "  \"exit_code\": {},", outcome.exit_code());
    let _ = writeln!(
        out,
        "  \"summary\": {{\"files\": {}, \"comments\": {}, \"open\": {}, \"closed\": {}, \"malformed\": {}}},",
        outcome.files, outcome.comments, outcome.open, outcome.closed, outcome.malformed
    );
    out.push_str("  \"violations\": [\n");
    for (i, v) in outcome.violations.iter().enumerate() {
        out.push_str("    {");
        let _ = write!(out, "\"path\": {}, ", esc(&v.path.display().to_string()));
        let _ = write!(out, "\"line\": {}, ", v.line);
        let id = match v.comment_id {
            Some(id) => id.to_string(),
            None => "null".to_string(),
        };
        let _ = write!(out, "\"id\": {}, ", id);
        let _ = write!(out, "\"code\": {}, ", esc(v.code));
        let _ = write!(out, "\"severity\": {}, ", v.severity);
        let _ = write!(out, "\"message\": {}", esc(&v.message));
        out.push('}');
        if i + 1 < outcome.violations.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

// --------------------------------------------------------------- close

pub fn close_human(outcome: &CloseOutcome, dry_run: bool) -> String {
    let mut out = String::new();
    let mode = if dry_run { " (dry run)" } else { "" };
    let _ = writeln!(
        out,
        "cmt close{} · {} action(s) · {} file(s) · {} log(s)",
        mode,
        outcome.actions.len(),
        outcome.files,
        outcome.logs.len()
    );
    for action in &outcome.actions {
        let _ = write!(
            out,
            "  {}  {}:{}  ",
            pad(action.action, 15),
            action.path.display(),
            action.line
        );
        if let Some(log) = &action.log {
            let _ = write!(out, "→ {}  ", log.display());
        }
        let _ = writeln!(out, "· {}", truncate_chars(&action.content, 60));
    }
    out
}

pub fn close_json(outcome: &CloseOutcome, dry_run: bool) -> String {
    let mut out = String::new();
    out.push_str("{\n  \"command\": \"close\",\n");
    let _ = writeln!(out, "  \"version\": {},", esc(VERSION));
    let _ = writeln!(out, "  \"dry_run\": {},", dry_run);
    out.push_str("  \"actions\": [\n");
    for (i, action) in outcome.actions.iter().enumerate() {
        out.push_str("    {");
        let _ = write!(out, "\"path\": {}, ", esc(&action.path.display().to_string()));
        let _ = write!(out, "\"line\": {}, ", action.line);
        let _ = write!(out, "\"level\": {}, ", action.level);
        let _ = write!(out, "\"mode\": {}, ", esc(action.mode));
        let _ = write!(out, "\"action\": {}, ", esc(action.action));
        let log = match &action.log {
            Some(p) => esc(&p.display().to_string()),
            None => "null".to_string(),
        };
        let _ = write!(out, "\"log\": {}, ", log);
        let _ = write!(out, "\"hash\": {}, ", esc(&action.hash));
        let _ = write!(out, "\"content\": {}", esc(&action.content));
        out.push('}');
        if i + 1 < outcome.actions.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ],\n");
    let logs: Vec<String> = outcome
        .logs
        .iter()
        .map(|p| esc(&p.display().to_string()))
        .collect();
    let _ = writeln!(
        out,
        "  \"summary\": {{\"actions\": {}, \"files\": {}, \"logs\": [{}]}}",
        outcome.actions.len(),
        outcome.files,
        logs.join(", ")
    );
    out.push_str("}\n");
    out
}

const HELP_BODY: &str = r#"
USAGE
  cmt <command> [options] <path>...

COMMANDS
  scan      list discussion comments with surrounding context
  validate  check the closure protocol (open / malformed / strict log checks)
  close     close comments: `·` -> sidecar log, `··` -> append (closed)
  help      show this text

MARKERS
  ·   level 1  address it, then move the line into <doc>.discuss-log.md
  ··  level 2  address it, then append (closed) to the line itself

OPTIONS (all commands)
  --json               machine-readable output
  -q, --quiet          one summary line only
  -h, --help           this text
  -V, --version        version

OPTIONS (scan)
  -C, --context <n>    context lines before and after (default 4, max 50)
  -B, --before <n>     context lines before only
  -A, --after <n>      context lines after only
      --level <1|2>    only this level
      --open-only      only open comments
      --closed-only    only closed comments
  -g, --grep <text>    only comments containing text (case-insensitive)
  -s, --summary        table only, no context blocks
      --ext <list>     file extensions for directory walks (md,txt...)
      --no-fence-skip  also read comments inside ``` fences
      --include-logs   do not skip <doc>.discuss-log.md sidecars
      --exit-code      2 if open comments remain, 3 if malformed

OPTIONS (validate)
      --strict         also read the sidecar log and flag duplicates
      --allow-open     treat open comments as warnings (exit 0)
      --allow-lossy    accept non-UTF-8 files instead of failing

OPTIONS (close)
  -a, --all            every open comment (with --force: malformed markers too)
  -l, --line <n>       one comment by line (repeatable)
      --id <n>         one comment by scan id (repeatable)
      --level <1|2>    restrict to one level
  -n, --note <text>    resolution note written to the sidecar log
      --log <path>     sidecar log path (default <doc>.discuss-log.md)
      --dry-run        report actions, write nothing
      --force          delete malformed markers instead of refusing
      --allow-lossy    rewrite non-UTF-8 files anyway

EXIT CODES
  0 ok · 1 usage/IO error · 2 open comments remain · 3 protocol error
"#;

pub fn help() -> String {
    let mut out = String::new();
    let _ = writeln!(out, "cmt {} · in-file discussion comments (· / ··)", VERSION);
    out.push_str(HELP_BODY);
    out
}
