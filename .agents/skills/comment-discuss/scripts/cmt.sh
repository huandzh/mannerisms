#!/usr/bin/env bash
# Thin launcher for the Rust `cmt` CLI (comment-discuss skill).
# Resolution: $CMT_BIN -> <skill>/bin/cmt -> PATH -> cargo build from <skill>/tools/comment_discuss.
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
skill_dir="$(cd "$script_dir/.." && pwd)"
bin="${CMT_BIN:-$skill_dir/bin/cmt}"

if [ ! -x "$bin" ]; then
  if command -v cmt >/dev/null 2>&1; then
    exec cmt "$@"
  fi
  manifest="$skill_dir/tools/comment_discuss/Cargo.toml"
  if [ ! -f "$manifest" ]; then
    echo "cmt: no CMT_BIN, no bin/cmt, no cmt on PATH, and no source at $manifest" >&2
    exit 1
  fi
  if ! command -v cargo >/dev/null 2>&1; then
    echo "cmt: cargo not found; set CMT_BIN or put cmt on PATH" >&2
    exit 1
  fi
  echo "[cmt] building release binary (first run)..." >&2
  cargo build --release --offline --manifest-path "$manifest" >/dev/null
  built="$skill_dir/tools/comment_discuss/target/release/cmt"
  mkdir -p "$skill_dir/bin" 2>/dev/null || true
  cp "$built" "$skill_dir/bin/cmt" 2>/dev/null || true
  bin="$built"
fi

exec "$bin" "$@"
