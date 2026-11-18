# Changelog

All notable changes to this project are documented here. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning: [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-11-18

### Added

- `local-plan` — plans land in `.plans/` at the repo root and wait for approval before code is written.
- `comment-discuss` — in-file discussion on `·` / `··` lines, closed by `cmt` and proven by
  `validate`; the zero-dependency Rust crate is vendored so the skill builds itself.
- Local gate: `scripts/validate-skills.mjs`, `scripts/check.mjs`, `lefthook.yml`.
- Contract and docs: `AGENTS.md`, `docs/coding-rules.md`, `docs/skill-anatomy.md`,
  `docs/provenance.md`.
- An English digest for `comment-discuss` at `references/en.md` — fallback, not authoritative.
