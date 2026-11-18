//! `cmt` — in-file discussion comments for agent/human review loops.
//!
//! Marker protocol
//!   `·`  level 1: address it, then move the line into the sidecar log.
//!   `··` level 2: address it, then append `(closed)` to the line itself.

mod close;
mod config;
mod marker;
mod model;
mod report;
mod scan;
mod text;
mod timeutil;
mod validate;

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use config::Config;
use model::{FileScan, State};
use report::RenderOpts;
use scan::ScanOptions;

const COMMANDS: &[&str] = &[
    "scan", "list", "validate", "check", "status", "close", "resolve", "help", "version",
];

fn main() {
    let args = expand(std::env::args().skip(1).collect());
    std::process::exit(run(args));
}

/// Turn `--flag=value` into `--flag value` so the parser needs one shape only.
fn expand(args: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(args.len());
    for arg in args {
        if arg.starts_with("--") {
            if let Some((key, value)) = arg.split_once('=') {
                out.push(key.to_string());
                out.push(value.to_string());
                continue;
            }
        }
        out.push(arg);
    }
    out
}

fn run(args: Vec<String>) -> i32 {
    match run_inner(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("cmt: {}", message);
            eprintln!("try `cmt --help`");
            1
        }
    }
}

fn run_inner(args: &[String]) -> Result<i32, String> {
    if args.is_empty() {
        print!("{}", report::help());
        return Ok(0);
    }

    let command_at = args.iter().position(|a| COMMANDS.contains(&a.as_str()));
    let (command, rest): (String, Vec<String>) = match command_at {
        Some(index) => {
            let mut rest: Vec<String> = args[..index].to_vec();
            rest.extend_from_slice(&args[index + 1..]);
            (args[index].clone(), rest)
        }
        None => {
            if args.iter().any(|a| a == "-V" || a == "--version") {
                println!("cmt {}", report::VERSION);
                return Ok(0);
            }
            let has_path = args.iter().any(|a| !a.starts_with('-'));
            if args.iter().any(|a| a == "-h" || a == "--help") || !has_path {
                print!("{}", report::help());
                return Ok(0);
            }
            // `cmt docs/` is shorthand for `cmt scan docs/`.
            ("scan".to_string(), args.to_vec())
        }
    };

    match command.as_str() {
        "scan" | "list" => cmd_scan(&rest),
        "validate" | "check" | "status" => cmd_validate(&rest),
        "close" | "resolve" => cmd_close(&rest),
        "help" => {
            print!("{}", report::help());
            Ok(0)
        }
        "version" => {
            println!("cmt {}", report::VERSION);
            Ok(0)
        }
        other => Err(format!("unknown command `{}`", other)),
    }
}

// ------------------------------------------------------------------ opts

#[derive(Debug, Default)]
struct Opts {
    json: bool,
    quiet: bool,
    summary: bool,
    help: bool,
    version: bool,
    context: Option<usize>,
    before: Option<usize>,
    after: Option<usize>,
    level: Option<usize>,
    open_only: bool,
    closed_only: bool,
    grep: Option<String>,
    ext: Option<Vec<String>>,
    no_fence_skip: bool,
    include_logs: bool,
    exit_code: bool,
    strict: bool,
    allow_open: bool,
    allow_lossy: bool,
    lines: Vec<usize>,
    ids: Vec<usize>,
    all: bool,
    note: Option<String>,
    log: Option<PathBuf>,
    dry_run: bool,
    force: bool,
    paths: Vec<PathBuf>,
}

fn parse_opts(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--" {
            o.paths.extend(args[i + 1..].iter().map(PathBuf::from));
            break;
        }
        if !arg.starts_with('-') || arg == "-" {
            o.paths.push(PathBuf::from(arg));
            i += 1;
            continue;
        }
        match arg {
            "--json" => o.json = true,
            "-q" | "--quiet" => o.quiet = true,
            "-s" | "--summary" => o.summary = true,
            "-h" | "--help" => o.help = true,
            "-V" | "--version" => o.version = true,
            "-C" | "--context" => {
                let v = take_value(args, &mut i, arg)?;
                o.context = Some(number(&v, arg)?);
            }
            "-B" | "--before" => {
                let v = take_value(args, &mut i, arg)?;
                o.before = Some(number(&v, arg)?);
            }
            "-A" | "--after" => {
                let v = take_value(args, &mut i, arg)?;
                o.after = Some(number(&v, arg)?);
            }
            "--level" => {
                let v = take_value(args, &mut i, arg)?;
                let level = number(&v, arg)?;
                if level == 0 || level > 2 {
                    return Err("--level takes 1 (·) or 2 (··)".to_string());
                }
                o.level = Some(level);
            }
            "--open-only" => o.open_only = true,
            "--closed-only" => o.closed_only = true,
            "-g" | "--grep" => o.grep = Some(take_value(args, &mut i, arg)?),
            "--ext" => {
                let v = take_value(args, &mut i, arg)?;
                let list: Vec<String> = v
                    .split(',')
                    .map(|s| s.trim().trim_start_matches('.').to_ascii_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                o.ext = Some(list);
            }
            "--no-fence-skip" => o.no_fence_skip = true,
            "--include-logs" => o.include_logs = true,
            "--exit-code" => o.exit_code = true,
            "--strict" => o.strict = true,
            "--allow-open" => o.allow_open = true,
            "--allow-lossy" => o.allow_lossy = true,
            "-l" | "--line" => {
                let v = take_value(args, &mut i, arg)?;
                o.lines.push(number(&v, arg)?);
            }
            "--id" => {
                let v = take_value(args, &mut i, arg)?;
                o.ids.push(number(&v, arg)?);
            }
            "-a" | "--all" => o.all = true,
            "-n" | "--note" => o.note = Some(take_value(args, &mut i, arg)?),
            "--log" => o.log = Some(PathBuf::from(take_value(args, &mut i, arg)?)),
            "--dry-run" => o.dry_run = true,
            "--force" => o.force = true,
            other => return Err(format!("unknown option `{}`", other)),
        }
        i += 1;
    }
    Ok(o)
}

fn take_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| format!("{} requires a value", flag))
}

fn number(value: &str, flag: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|_| format!("{} expects a number, got `{}`", flag, value))
}

// ---------------------------------------------------------------- scan

fn cmd_scan(rest: &[String]) -> Result<i32, String> {
    let o = parse_opts(rest)?;
    if o.help {
        print!("{}", report::help());
        return Ok(0);
    }
    if o.version {
        println!("cmt {}", report::VERSION);
        return Ok(0);
    }
    if o.paths.is_empty() {
        return Err("scan: pass at least one file or directory".to_string());
    }

    let base_cfg = Config::load_for(&o.paths[0]);
    let mut scan_opts = ScanOptions::from_config(&base_cfg);
    if let Some(ext) = &o.ext {
        scan_opts.extensions = ext.clone();
    }
    let ctx = o.context.unwrap_or(base_cfg.context).min(50);
    scan_opts.before = o.before.unwrap_or(ctx).min(50);
    scan_opts.after = o.after.unwrap_or(ctx).min(50);
    scan_opts.skip_fences = base_cfg.skip_fences && !o.no_fence_skip;
    scan_opts.include_logs = o.include_logs;

    let paths = scan::collect_files(&o.paths, &scan_opts)?;
    let mut cache: HashMap<PathBuf, Config> = HashMap::new();
    let mut views: Vec<FileScan> = Vec::new();

    for path in paths {
        let cfg = cfg_for(&path, &mut cache);
        if !o.include_logs && looks_like_log(&path, &cfg) {
            continue;
        }
        let mut file_scan = scan::scan_file(&path, &scan_opts, &cfg)?;
        file_scan.comments.retain(|c| {
            scan::matches(
                c,
                o.level,
                o.open_only,
                o.closed_only,
                o.grep.as_deref(),
            )
        });
        views.push(file_scan);
    }

    if o.json {
        print!("{}", report::scan_json(&views));
    } else {
        print!(
            "{}",
            report::scan_human(
                &views,
                RenderOpts {
                    summary_only: o.summary,
                    quiet: o.quiet,
                }
            )
        );
    }

    if !o.exit_code {
        return Ok(0);
    }
    let mut malformed = 0;
    let mut open = 0;
    for file in &views {
        for c in &file.comments {
            match c.state {
                State::Malformed => malformed += 1,
                State::Open => open += 1,
                State::Closed => {}
            }
        }
    }
    Ok(if malformed > 0 {
        3
    } else if open > 0 {
        2
    } else {
        0
    })
}

// ------------------------------------------------------------ validate

fn cmd_validate(rest: &[String]) -> Result<i32, String> {
    let o = parse_opts(rest)?;
    if o.help {
        print!("{}", report::help());
        return Ok(0);
    }
    if o.paths.is_empty() {
        return Err("validate: pass at least one file or directory".to_string());
    }

    let base_cfg = Config::load_for(&o.paths[0]);
    let mut scan_opts = ScanOptions::from_config(&base_cfg);
    if let Some(ext) = &o.ext {
        scan_opts.extensions = ext.clone();
    }
    let files = scan::collect_files(&o.paths, &scan_opts)?;

    let vopts = validate::ValidateOptions {
        strict: o.strict,
        allow_open: o.allow_open,
        allow_lossy: o.allow_lossy,
    };
    let mut cache: HashMap<PathBuf, Config> = HashMap::new();
    let mut outcome = validate::Outcome::default();
    for path in files {
        let cfg = cfg_for(&path, &mut cache);
        if !o.include_logs && looks_like_log(&path, &cfg) {
            continue;
        }
        let one = validate::validate_file(&path, &cfg, &vopts)?;
        outcome.files += one.files;
        outcome.comments += one.comments;
        outcome.open += one.open;
        outcome.closed += one.closed;
        outcome.malformed += one.malformed;
        outcome.violations.extend(one.violations);
    }

    if o.json {
        print!("{}", report::validate_json(&outcome));
    } else {
        print!(
            "{}",
            report::validate_human(
                &outcome,
                RenderOpts {
                    summary_only: false,
                    quiet: o.quiet,
                }
            )
        );
    }
    Ok(outcome.exit_code())
}

// --------------------------------------------------------------- close

fn cmd_close(rest: &[String]) -> Result<i32, String> {
    let o = parse_opts(rest)?;
    if o.help {
        print!("{}", report::help());
        return Ok(0);
    }
    if o.paths.is_empty() {
        return Err("close: pass at least one file".to_string());
    }
    if o.log.is_some() && o.paths.len() > 1 {
        return Err("--log takes a single document; run close once per file".to_string());
    }

    let cfg = Config::load_for(&o.paths[0]);
    let opts = close::CloseOptions {
        lines: o.lines.clone(),
        ids: o.ids.clone(),
        all: o.all,
        levels: o.level.map(|l| vec![l]).unwrap_or_default(),
        note: o.note.clone(),
        log: o.log.clone(),
        dry_run: o.dry_run,
        allow_lossy: o.allow_lossy,
        force: o.force,
        token: cfg
            .closed_tokens
            .first()
            .cloned()
            .unwrap_or_else(|| "closed".to_string()),
    };

    let outcome = close::close_paths(&o.paths, &cfg, &opts)?;
    if o.json {
        print!("{}", report::close_json(&outcome, o.dry_run));
    } else if outcome.actions.is_empty() {
        println!("cmt close · nothing to close");
    } else {
        print!("{}", report::close_human(&outcome, o.dry_run));
    }
    Ok(0)
}

// ------------------------------------------------------------- helpers

fn cfg_for(path: &Path, cache: &mut HashMap<PathBuf, Config>) -> Config {
    let dir = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    cache
        .entry(dir)
        .or_insert_with(|| Config::load_for(path))
        .clone()
}

fn looks_like_log(path: &Path, cfg: &Config) -> bool {
    if scan::is_log_file(path, cfg) {
        return true;
    }
    let mut buf = [0u8; 512];
    if let Ok(mut file) = std::fs::File::open(path) {
        if let Ok(n) = file.read(&mut buf) {
            let head = String::from_utf8_lossy(&buf[..n]);
            let head = head.trim_start_matches('\u{feff}');
            return head.starts_with(scan::LOG_MARKER);
        }
    }
    false
}
