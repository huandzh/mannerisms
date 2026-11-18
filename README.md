# mannerisms

Author-habit skills for AI coding agents — e.g. plan on disk, argue in the file. Portable SKILL.md
bundles, DSH-first, MIT.

**Status:** v0.1.0 · two skills · the gate runs locally

## Why mannerisms

- **Habits, not frameworks.** Each skill is one working manner, small enough to read in a sitting.
- **Portable by construction.** A skill is one directory; copy it anywhere and it still works.
- **Argued in the file.** Review happens next to the text it is about, and closure is verifiable.

## Catalog

| Skill | What it changes about behavior | Ships |
| --- | --- | --- |
| [`local-plan`](.agents/skills/local-plan/SKILL.md) | Plans land in `.plans/` at the repo root and wait for approval before code is written. | the file convention |
| [`comment-discuss`](.agents/skills/comment-discuss/SKILL.md) | Discussion happens on `·` / `··` lines in the document; `cmt` closes the loop and proves nothing was missed. | `cmt` (Rust, zero dependencies) |

## Install

**DSH** discovers `.agents/skills` in any workspace — clone this repo into a workspace, or into a
subdirectory a workspace already contains, and the skills appear in the catalog with no install step.

**ah-my-skills** can link one skill into any client:

```bash
ah-my-skills link comment-discuss --tool agents --scope repo
```

**Any client that reads the open standard** (a root `AGENTS.md` plus `.agents/skills/`) needs no
translation layer.

**Manual:** copy `.agents/skills/<name>/` into the target repo's `.agents/skills/`. The directory is
self-contained; that is a rule, not an aspiration.

## Verify it loaded

- DSH: both skills appear in the session catalog; invoking `/local-plan` creates `.plans/<date>-<slug>.md`.
- `comment-discuss`: `pwsh .agents/skills/comment-discuss/scripts/cmt.ps1 scan <file>` lists comments
  with context.

## Authoring

Read [AGENTS.md](AGENTS.md) — the contract — and [docs/coding-rules.md](docs/coding-rules.md) — the
letter. The gate is `node scripts/check.mjs`. To add a skill, follow
[AGENTS.md § Add a skill](AGENTS.md#add-a-skill).

## Versioning

Semantic version tags on `master`; per-skill `metadata.version` when a skill changes alone. See
[CHANGELOG.md](CHANGELOG.md).

## License and provenance

MIT — see [LICENSE](LICENSE). Every skill's origin and license is recorded in
[docs/provenance.md](docs/provenance.md).

This repo ships no `CLAUDE.md` and no `.claude/` directory. If a client wants one, create it locally;
`.gitignore` keeps it out of the repo.
