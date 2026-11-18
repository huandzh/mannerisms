//! Minimal per-directory config: `.comment-discuss.conf`.
//!
//! Discovery walks up from the target file's directory (max 6 levels) and uses
//! the first config found. Only simple `key = value` lines are supported, so the
//! tool stays dependency-free.

use std::path::Path;

pub const CONF_NAME: &str = ".comment-discuss.conf";

#[derive(Clone, Debug)]
pub struct Config {
    /// Sidecar log takes the place of the document extension: `report.discuss-log.md`.
    pub log_suffix: String,
    pub context: usize,
    pub closed_tokens: Vec<String>,
    pub skip_fences: bool,
    pub extensions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            log_suffix: ".discuss-log.md".to_string(),
            context: 4,
            closed_tokens: vec!["closed".to_string()],
            skip_fences: true,
            extensions: vec![
                "md".to_string(),
                "markdown".to_string(),
                "mdx".to_string(),
                "txt".to_string(),
            ],
        }
    }
}

impl Config {
    pub fn load_for(path: &Path) -> Config {
        let mut dir = path.parent();
        let mut depth = 0;
        while let Some(d) = dir {
            if depth > 6 {
                break;
            }
            let candidate = d.join(CONF_NAME);
            if candidate.is_file() {
                if let Ok(text) = std::fs::read_to_string(&candidate) {
                    return Config::parse(&text);
                }
                break;
            }
            dir = d.parent();
            depth += 1;
        }
        Config::default()
    }

    pub fn parse(text: &str) -> Config {
        let mut cfg = Config::default();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }
            let (key, value) = match line.split_once('=') {
                Some(kv) => kv,
                None => continue,
            };
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim();
            match key.as_str() {
                "log_suffix" if !value.is_empty() => cfg.log_suffix = value.to_string(),
                "context" => {
                    if let Ok(n) = value.parse::<usize>() {
                        cfg.context = n.min(50);
                    }
                }
                "closed_tokens" => {
                    let list = split_list(value);
                    if !list.is_empty() {
                        cfg.closed_tokens = list;
                    }
                }
                "skip_fences" => {
                    cfg.skip_fences = !matches!(value.to_ascii_lowercase().as_str(), "0" | "false" | "no");
                }
                "extensions" => {
                    let mut list = split_list(value);
                    for ext in list.iter_mut() {
                        *ext = ext.trim_start_matches('.').to_ascii_lowercase();
                    }
                    list.retain(|e| !e.is_empty());
                    cfg.extensions = list; // empty list means "every extension"
                }
                _ => {}
            }
        }
        cfg
    }
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let cfg = Config::default();
        assert_eq!(cfg.context, 4);
        assert_eq!(cfg.log_suffix, ".discuss-log.md");
        assert!(cfg.skip_fences);
    }

    #[test]
    fn parses_overrides() {
        let cfg = Config::parse(
            "# comment\ncontext = 3\nlog_suffix = .comments.md\nclosed_tokens = closed, done\nextensions = .md, rst\nskip_fences = false\nbogus = 1\n",
        );
        assert_eq!(cfg.context, 3);
        assert_eq!(cfg.log_suffix, ".comments.md");
        assert_eq!(cfg.closed_tokens, vec!["closed", "done"]);
        assert_eq!(cfg.extensions, vec!["md", "rst"]);
        assert!(!cfg.skip_fences);
    }
}
