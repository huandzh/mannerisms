# AGENTS.md — the contract

> **Stop. Read this before touching anything.**

`mannerisms` is a small library of author-habit skills: the working manners one engineer keeps,
packaged so an agent can adopt them. Every skill is a directory under `.agents/skills/`; every tool a
skill needs ships inside its own directory. The repo is meant to be read, copied, and redistributed.

## Done means

`node scripts/check.mjs` exits 0, and the skill you touched still works when its directory is copied
alone into another repo. Anything less is not done.

## The iron laws

1. **The gate is green.** `node scripts/check.mjs`. Nothing ships red. If you could not run it, say
   which part you skipped and why — silence is a lie.
2. **One skill, one directory.** `.agents/skills/<name>/SKILL.md`, `name` lowercase-hyphen and identical
   to the directory. No copies under `.claude/` or `.opencode/`; local installs are gitignored.
3. **Frontmatter is minimal.** `name` and `description` only (optional: `license`, `compatibility`,
   `metadata`). The description is ≤ 1024 chars and carries both triggers and anti-triggers.
4. **Progressive disclosure.** `SKILL.md` ≤ 500 lines; detail goes to `references/`, code to
   `scripts/`, fixtures to `examples/`.
5. **A skill works when copied alone.** No imports from the repo root, no sibling-repo paths, no `../..`
   escapes. The tool it needs lives inside the bundle.
6. **No binaries, no secrets, no private paths.** `bin/` is gitignored. A reviewer must never find a
   personal directory, a token, or a compiled artifact in a bundle.
7. **Provenance first.** A row in `docs/provenance.md` exists before the skill is committed.
8. **One logical change per commit**, and the message names the skill. History is the record: no
   force-push, no rewritten trail.

## Layout

```
.agents/skills/<name>/SKILL.md   the only entry point; frontmatter + instructions
.agents/skills/<name>/references/ documentation loaded on demand, never at discovery
.agents/skills/<name>/scripts/    executables the skill runs
.agents/skills/<name>/tools/      buildable source a script compiles (bundle-local)
.agents/skills/<name>/examples/   fixtures the skill's own smoke test uses
docs/coding-rules.md              the letter: layout, frontmatter, commits, scripts
docs/skill-anatomy.md             the format the validator enforces
docs/provenance.md                origin, license, and changes for every skill
.plans/                           the design ledger; plans are committed
```

## Add a skill

1. Create `.agents/skills/<name>/SKILL.md` — verify it appears in
   `node scripts/validate-skills.mjs --json`.
2. Write `name` + `description` (trigger and anti-trigger) — verify the validator accepts the
   name/directory pair.
3. Add a row to the README catalog and to `docs/provenance.md` — verify the parity check passes.
4. Run `node scripts/check.mjs` — verify exit 0.
5. Commit one logical change naming the skill.

## Port an existing skill

Copy it, then de-privatize: rewrite client-specific paths to bundle-relative ones, drop claims about
artifacts that are not committed, vendor any source a script builds. Do not rewrite the body to taste —
the habit is the content.

## Language

Repo docs are English. A skill body keeps the author's language, because the voice is the manner.
The frontmatter `description` starts in English so the catalog works everywhere. Do not translate a
body without evidence that translation helps; the probe in
`.plans/2026-11-18-init-mannerisms-repo.md` §6 is the precedent.

## No CLAUDE.md

This repo generates no client-specific files. If you want a local `CLAUDE.md`, create it and leave it
untracked; `.gitignore` already ignores it.

## Pointers

- The letter of the rules: [docs/coding-rules.md](docs/coding-rules.md)
- The skill format: [docs/skill-anatomy.md](docs/skill-anatomy.md)
- Where each skill came from: [docs/provenance.md](docs/provenance.md)
- Why decisions were made: [.plans/](.plans/)
