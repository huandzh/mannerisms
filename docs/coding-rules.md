# Coding rules

The letter behind [AGENTS.md](../AGENTS.md). AGENTS.md states the contract; this file states the
mechanics the gate enforces.

## Encoding and endings

- UTF-8 without BOM, LF only. `.gitattributes` normalizes; the validator fails a CRLF or a BOM.
- Chinese text is fine everywhere: git, the validator, and the catalog all handle UTF-8. The only
  language rule is discoverability (English description), never bytes.

## Bundle layout

- `<skill>/SKILL.md` is the only entry point. Discovery parses its frontmatter and never reads further.
- `references/` for prose loaded on demand, `scripts/` for executables, `examples/` for fixtures,
  `tools/` for buildable source a script compiles, `bin/` for local build output (gitignored).
- Any other top-level directory in a bundle is a warning: it is probably something that belongs at the
  repo root instead.

## Frontmatter

| Field           | Required | Constraint                                                        |
| --------------- | -------- | ----------------------------------------------------------------- |
| `name`          | yes      | `^[a-z0-9]+(-[a-z0-9]+)*$`, ≤ 64 chars, equal to the directory   |
| `description`   | yes      | 1–1024 chars; what the skill does **and when to use it**          |
| `license`       | no       | SPDX identifier or a path to the bundle's license                 |
| `compatibility` | no       | ≤ 500 chars; environment requirements                             |
| `metadata`      | no       | string map; this repo records `language`, and `version` when a skill changes alone |
| `allowed-tools` | no       | experimental; not used here yet                                   |

Unknown top-level keys warn: other clients ignore them, but a typo in a real key drops the skill, so an
unknown key is worth a look.

## Descriptions

A description is an interface, not a summary:

- start with what it does, in one clause;
- name the triggers ("invoke when the user asks to…");
- name the **anti-triggers** — the nearby request that must not load it;
- stay under 1024 characters. The catalog shows this text for every skill, in every session.

## Progressive disclosure

`SKILL.md` ≤ 500 lines, ideally under 150. Move tables, flag references, and long examples into
`references/`. The body is the decision procedure; the references are the lookup.

## Links

Every relative link in `SKILL.md` must resolve inside the bundle. No `../` escapes, no absolute paths,
no links to the repo root: a bundle is copied alone.

## Scripts

- Cross-platform: `scripts/*.ps1` and `scripts/*.sh` keep the same resolution order —
  `CMT_BIN` → `bin/` → `PATH` → build from `tools/`.
- UTF-8 output: PowerShell 7 defaults to it; a script that prints text must not depend on the console
  code page.
- No network, no install steps, no writes outside the document being edited (plus its sidecar log).
- Exit codes are an interface: `0` ok, `1` usage/IO, `2` work remains, `3` protocol error.

## Commits

- One logical change. Subject ≤ 72 chars, imperative, names the skill when it touches one.
- Body explains the why, wraps at 72, and names the gate that ran.
- Never `git add -A`; stage by path. Never commit a lockfile change you did not cause.
- History is the record: no force-push, no amended published commits, no squash that erases a phase.

## What the gate runs

`node scripts/check.mjs` — validator → `cargo test` (vendored crate) → `cargo build --release` →
`git diff --check`. CI is local by design; there is no GitHub Actions workflow at this stage.
