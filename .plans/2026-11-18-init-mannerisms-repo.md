# Init `mannerisms` — an agent-managed, redistributable skill library

Status: revised after in-file review · Date: 2026-11-18 · Repo: `G:\github\mannerisms` · Branch: `master` (rename in phase 1; currently `main`)

## 1. Goal

Turn an empty, freshly initialized repo into a small, reviewable, redistributable library of
**author-habit skills** that is:

- discovered by DSH with **zero configuration** (`.agents/skills` is a default project root);
- safe to open-source: no private context, no committed binaries, no unlicensed third-party text;
- maintainable by agents under an explicit contract (`AGENTS.md`) with a **deterministic local gate**.

**Non-goal:** another skill manager. The manager is `ah-my-skills`; this repo is a *library* it can
point at. But a library is not only prose — see §2.

## 2. What the name promises

·· tooling should be included for the skills to be effective (closed)

`mannerisms` is the habits first — how the author plans, how the author argues a document, how the
author leaves a session — **and the tooling each habit needs to be effective**. Prose alone is a
suggestion; a habit with a tool is enforceable. So `comment-discuss` ships its `cmt` CLI inside the
bundle, and `local-plan` needs nothing beyond a filesystem. Tooling is admitted by the same rule as
prose: only what the habit actually uses.

Catalog v0.1 — two skills, both real habits, both already in daily use:

| Skill             | The manner it encodes                     | Ships                        |
| ----------------- | ----------------------------------------- | ---------------------------- |
| `local-plan`      | Plan on disk; approve before coding.      | the file convention, nothing |
| `comment-discuss` | Argue in the file itself; close the loop. | `cmt` (Rust, zero deps)      |

A skill is admitted only if it encodes a **manner the author actually practices**, reads in one
sitting, works when copied alone, and carries any tool it needs inside its own directory.

## 3. GitHub identity (input for creating the remote)

·· default branch: master - don't forget history by averting eye sights. (closed)

- **Name:** `mannerisms`
- **Owner:** `<your-github-handle>` (assumed `huandzh`; substitute when creating)
- **Visibility:** public · **Default branch:** `master` · **License:** MIT
- **History is the record.** No force-push, no history rewriting, no squashing a phase into a black
  box. One commit per phase, each carrying its gate output, so a reader can see what changed, why,
  and when it broke. The local branch is renamed before the first push: `git branch -m master`.
- **Settings at this stage:** fork pull requests disabled (the gate runs locally, §6); issues and
  Discussions on; merges never collapse more than one phase.
- **Create the remote empty** — no README, no `.gitignore`, no license; this repo owns all three.

**Description (chosen):**

> Author-habit skills for AI coding agents — e.g. plan on disk, argue in the file. Portable SKILL.md
> bundles, DSH-first, MIT.

The `e.g.` is deliberate: the two skills are named as **examples of a manner**, so the About line
survives the catalog growing from two skills to ten. The README catalog — not the About field — owns
enumeration; an About line that lists every skill goes stale on the next commit.

**Topics:** `agent-skills`, `skill-md`, `agents-md`, `ai-agents`, `coding-agents`, `dsh`,
`deepseek-harness`, `developer-tools`, `workflow`, `prompt-engineering`

**Wire it after the first commits exist:**

```bash
git branch -m master
git remote add origin git@github.com:<your-github-handle>/mannerisms.git
git push -u origin master
```

## 4. Convention: what we inherit, what we change

·· remove CLAUDE.md, AGENTS.md should be suffice (closed)

·· ref coding rules to agents.md, we do need a concise agents.md and coding rules (closed)

Sources read: `I:\repos\otus` (AGENTS.md + `.agents/skills` + `.plans` + lefthook + gitattributes),
`G:\github\ah-my-skill` (AGENTS/CLAUDE contract, `.claude/rules`, provenance), and the current open
standards: [agents.md](https://agents.md) (stewarded by the Agentic AI Foundation, 60k+ repos),
[Agent Skills spec](https://agentskills.io/specification), and DSH's own skill-root table
(`packages/skill/skill-filesystem`: `project-agents` = `<projectRoot>/.agents/skills`, rank 200).

| Practice        | otus                            | ah-my-skill                                   | mannerisms decision                            | Why                                                              |
| --------------- | ------------------------------- | --------------------------------------------- | ---------------------------------------------- | ---------------------------------------------------------------- |
| Agent contract  | `AGENTS.md` (+ `AgentsReminder`) | `AGENTS.md` + `CLAUDE.md` = `@AGENTS.md`     | **`AGENTS.md` only**                           | One contract; the open standard is what clients read             |
| Coding rules    | `coding-guide` skill            | `.claude/rules/*.md`                          | **`docs/coding-rules.md`, linked from AGENTS.md** | AGENTS.md stays short; the letter lives in one place, read on demand |
| Skills home     | `.agents/skills` (one tree)     | `skills/` **+** `.claude/skills` **+** `.opencode/skills` | **`.agents/skills` only**         | DSH discovers it natively; one tree cannot drift                  |
| Provenance      | implicit                        | `skills/SKILL_SOURCES.md`                     | **`docs/provenance.md`, required by the gate** | Redistribution is the point; every skill needs a license trail    |
| Gate            | `moon run :check`               | `cargo fmt/clippy/test`                       | **local `scripts/check.mjs` + lefthook**       | No GitHub compute on a repo that takes no fork PRs               |
| Committed binaries | none                         | `skills/comment-discuss/bin/cmt.exe`          | **vendor source, build on demand**             | 470 KB of exe is permanent history for no review value            |
| Plans           | `.plans/` ledger                | `.plans/` + `refs/`                           | **`.plans/<date>-<slug>.md`**                  | Sortable, self-documenting, same habit                            |
| License         | —                               | MIT                                           | **MIT, Copyright Huan Di**                     | Matches the seed skills and the manager                           |

**No `.claude/` directory at all.** A contributor who prefers `CLAUDE.md` creates one locally;
`.gitignore` keeps it untracked. Nothing is generated, so nothing drifts, and any client that reads
the open standard needs no translation layer from us.

**The drift we are avoiding.** `ah-my-skill` pins the same skills in three trees; they have already
diverged (`archive-plan/skill.md` vs `SKILL.md`, per-tool link state in `.ah-my-skills.json`). A single
source tree plus one linker command removes that entire failure class.

## 5. Target tree

·· revise accordingly per above comments (closed)

```
mannerisms/
├── .agents/skills/                      # THE library — one source, no per-client copies
│   ├── local-plan/
│   │   └── SKILL.md                     # copied as-is from the author's skill
│   └── comment-discuss/
│       ├── SKILL.md                     # protocol + workflow (copied, patched, see §7)
│       ├── references/cli.md            # full flag table, bundle-relative paths
│       ├── references/en.md             # 20-line English digest, fallback not authoritative
│       ├── scripts/cmt.ps1 | cmt.sh     # resolve: CMT_BIN → bin/ → PATH → cargo build
│       ├── tools/comment_discuss/       # vendored Rust crate (MIT), standalone [workspace]
│       ├── examples/discussion-demo.md  # smoke fixture for scan → close → validate
│       └── bin/.gitkeep                 # build output, gitignored
├── .plans/
│   └── 2026-11-18-init-mannerisms-repo.md
├── .github/
│   ├── ISSUE_TEMPLATE/{bug,feature,skill-proposal}.md
│   └── PULL_REQUEST_TEMPLATE.md         # no workflows/ — the gate runs locally
├── docs/
│   ├── skill-anatomy.md                 # spec digest the gate enforces
│   ├── coding-rules.md                  # the letter: layout, frontmatter, commits, scripts
│   └── provenance.md                    # origin + license + changes, one row per skill
├── scripts/
│   ├── validate-skills.mjs              # zero-dep skill gate
│   └── check.mjs                        # the whole gate: validator + cargo test + diff check
├── AGENTS.md                            # the contract (≤ 120 lines)
├── README.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── CHANGELOG.md
├── LICENSE                              # MIT
├── lefthook.yml
├── .gitattributes
├── .editorconfig
└── .gitignore                           # also ignores CLAUDE.md, .claude/, target/, bin/*
```

**Why the source sits *inside* the skill bundle.** Skills are redistributed by copying one directory.
A `repo-root/tools/…` dependency breaks the moment the skill lands in someone else's repo. The bundle
must carry its own buildable source — the existing `cmt.ps1` already looks for
`<skill>/tools/comment_discuss/Cargo.toml`, so the convention is preserved rather than invented.

## 6. File specifications

·· test bilingual useful or not, if not contributing we use chinese(more info density) ( english - without encoding issues), delegate subagents to test effectiveness (closed)

·· else at least provide an english fallback for agents to pick up and tweak locally (closed)

### Language policy

UTF-8 everywhere — Chinese is not an encoding problem in git, CI, or a skill catalog. The open
question is *comprehension*, not bytes.

- repo docs (README, AGENTS, CONTRIBUTING, PR template): **English** — GitHub discovery and outside
  contributors read those;
- skill bodies: **the author's Chinese** — more information per line, and the voice is the manner;
- frontmatter `description`: **English trigger sentence first**, then the Chinese detail — the catalog
  is what every client shows;
- **tested, not assumed.** A two-arm probe (Chinese-only vs Chinese + English digest) ran on
  2026-11-18: same document, same questions, scored on comment lines, marker levels, close commands,
  and exit codes. Both arms scored 4/4 (§13), so a **full English translation is rejected** — it buys
  no comprehension and rots against the Chinese body;
- the **thin fallback ships anyway**: `references/en.md` is a 20-line digest, marked *fallback, not
  authoritative*, so an English reader can orient and an agent can localize it locally without
  forking the skill. It stays a pointer, never a second source of truth.

### `AGENTS.md` (~120 lines, the only contract)

Written *at* the agent, not about the repo:

1. **Stop. Read this first.** — what this repo is, what "done" means.
2. **Identity** — a habit library, DSH-first, redistribution-ready; not a manager, not a framework.
3. **Iron laws** (each with a one-line test): the gate is green; one skill = one directory, name ==
   directory, no per-client copies; frontmatter is `name` + `description` only; progressive
   disclosure; a skill works when copied alone; no binaries or secrets; provenance row before commit;
   one logical change per commit.
4. **Layout** — the tree above, one line each.
5. **Add a skill** — numbered, every step paired with its verify command.
6. **Pointers** — `docs/coding-rules.md` for the letter of the rules, `docs/skill-anatomy.md` for the
   format, `docs/provenance.md`, `.plans/`.

Coding rules are **linked, not inlined**: `AGENTS.md` stays a contract a person can read in two
minutes; `docs/coding-rules.md` holds file layout, frontmatter fields, validator expectations, commit
style, and script conventions.

·· drop CLAUDE.md support directly, let agent fix its local habits (closed)

### No `CLAUDE.md`

The repo ships no `CLAUDE.md`. Clients that read the open standard pick up `AGENTS.md`; a contributor
who wants a Claude-specific local file creates it and `.gitignore` keeps it untracked. Nothing is
generated, so nothing drifts.

### `README.md`

Title + the About description verbatim as the subtitle + status line (no CI badge — there is no
workflow) → **Why mannerisms** (habits, portable bundles, argued in the file) → **Catalog** table
(skill · what it changes about behavior · what it ships) → **Install** (DSH: clone or drop into a
workspace, `.agents/skills` is native; `ah-my-skills link <name> --tool agents --scope repo`; manual:
copy the directory) → **Verify it loaded** → **Authoring** pointer → **Versioning / License /
Provenance**.

### `docs/coding-rules.md`

The letter behind the contract: bundle layout, frontmatter fields and limits, description-writing
rules (what + when + anti-trigger), progressive-disclosure budget, relative-link rule, script
resolution order, commit convention (one phase per commit, gate output in the message), and the
Windows/UTF-8 constraints scripts must survive.

### `docs/skill-anatomy.md`

The spec digest the gate enforces: directory shape, frontmatter table, description-writing rules,
progressive disclosure budget, link rules, and a pre-commit checklist.

### `docs/provenance.md`

One row per skill: `skill | origin (path/URL) | author | license | what changed | commit`. Seeded with
`local-plan` (from `~/.agents/skills/local-plan` via `ah-my-skill`) and `comment-discuss` (from
`cpds_lead/.claude/skills/comment-discuss` via `ah-my-skill`, plus the `cpds_lead/tools/comment_discuss`
crate, MIT, zero dependencies).

### `scripts/validate-skills.mjs`

Zero-dependency Node ESM, `--json`, deterministic exit codes. Checks:

1. every `.agents/skills/*/SKILL.md` exists, UTF-8 without BOM, LF only;
2. frontmatter parses; `name` matches `^[a-z0-9]+(-[a-z0-9]+)*$`, ≤ 64 chars, equals the directory;
3. `description` present, 1–1024 chars;
4. body ≤ 500 lines;
5. unknown top-level frontmatter keys warn (portability);
6. every relative link in `SKILL.md` resolves inside the bundle;
7. every skill appears in the README catalog **and** `docs/provenance.md`;
8. no file > 5 MB and no `.exe/.dll/.pdb` in a bundle;
9. only conventional bundle subdirectories (`references/`, `scripts/`, `assets/`, `tools/`, `bin/`,
   `examples/`) — anything else warns.

·· save github's compute power, ci on ourself, this repo won't allow PR from other's fork for this stage (closed)

### Local gate, not GitHub Actions

`scripts/check.mjs` runs the whole gate in one command: validator → `cargo test` (vendored cmt) →
`cargo build --release` → `git diff --check`. `lefthook` wires it to `pre-commit` (staged-file
portion) and `pre-push` (full). There is no workflow file: this stage takes no fork PRs, so GitHub
compute buys nothing. Revisit the day the repo opens to outside pull requests.

### Templates

- PR: problem → linked plan → what changed → gate output pasted → **authoring environment** (hand-written
  vs model/harness/version/plugins) → existing PRs searched.
- Issues: `bug`, `feature`, and a `skill-proposal` template that asks for the *habit* the skill encodes.

### Hygiene files

·· copyright Huan Di (closed)

- `.gitattributes`: `* text=auto eol=lf`; `*.bat/*.cmd` CRLF; images/binary rules; `Cargo.lock` stays LF.
- `.editorconfig`: UTF-8, LF, final newline, trim trailing whitespace (off for `.md`), 2-space JS/MD,
  4-space Rust.
- `.gitignore`: `target/`, `node_modules/`, `.temp/`, `.venv/`, `__pycache__/`, `.dsh/`, `.sessions/`,
  `.worktrees/`, `*.log`, OS noise, `CLAUDE.md`, `.claude/`, and
  `.agents/skills/comment-discuss/bin/*` (keep `.gitkeep`).
- `LICENSE`: MIT, `Copyright (c) 2026 Huan Di`.

## 7. Skill specifications

·· is dsh specifics required? guess not (closed)

·· revise what are unfit for opensource, else copy as it is, no need to rewrite all (closed)

**Port by copy, then de-privatize.** Both skills already work; rewriting them costs fidelity and
invites new bugs. Copy the bundle, apply the smallest patch that removes anything unfit for a public
repo, and let the validator prove the result. Nothing DSH-specific goes into a skill body — the skills
stay client-agnostic; only the repo's own docs mention DSH.

### `local-plan`

Copy `.agents/skills/local-plan/SKILL.md` verbatim (6 lines). **Patch list: none.** Nothing in it is
private, and the naming/template rules an earlier draft proposed are not added: the habit is "plan on
disk, write first", and the skill already says exactly that. If it needs more later, that is a new
review, not an opening rewrite.

### `comment-discuss`

Copy `SKILL.md`, `references/cli.md`, `scripts/cmt.ps1`, `scripts/cmt.sh`, then apply this patch list
and nothing else:

1. **Paths** — `.claude/skills/comment-discuss/...` → bundle-relative `<skill>/scripts/...`.
2. **Claims** — stop asserting that a prebuilt `cmt.exe` ships in the repo; state the resolution
   order (CMT_BIN → `bin/` → PATH → local `cargo build --release`).
3. **Source** — vendor `tools/comment_discuss/` (`Cargo.toml`, `src/`, `tests/`) so the wrapper's
   build path exists and the crate stays standalone (`[workspace]`, no dependencies).
4. **Reference** — update `references/cli.md` build/test paths to the bundle layout.
5. **English fallback** — add `references/en.md` as the 20-line digest from §6 (fallback, not a
   translation; the probe rejected the full one).
6. **Untouched** — protocol, exit codes, log format, config, boundaries, and the Chinese body.
7. **Known gap** — `cmt close -n` writes a resolution note only when an L1 line moves; on an L2-only
   file it reports `0 log(s)` and the note is dropped. State that in `references/cli.md` (one line) or
   have the wrapper echo it; the reviewer's reasoning must not vanish silently.

## 8. Gates

```powershell
node scripts/check.mjs          # validator + cargo test + release build + git diff --check
node scripts/validate-skills.mjs
cargo test  --manifest-path .agents/skills/comment-discuss/tools/comment_discuss/Cargo.toml
git diff --check
```

**DSH smoke:** start DSH with this repo as the workspace; both skills must appear in the catalog;
`/local-plan` must create a `.plans/<date>-<slug>.md`; `cmt scan/close/validate` must pass on
`examples/discussion-demo.md` with the documented exit codes (0/2/3). The smoke stays manual — there
is no headless harness for it yet.

## 9. Execution order (one commit per phase, staged by path)

1. `chore: repo hygiene` — `.gitattributes`, `.editorconfig`, `.gitignore`, `LICENSE`; `git branch -m master`.
2. `docs: contract and rules` — `AGENTS.md`, `docs/coding-rules.md`, `README.md`, this plan.
3. `feat(local-plan): copy the habit`.
4. `feat(comment-discuss): port, de-privatize, vendor cmt`.
5. `chore: local gate` — `scripts/validate-skills.mjs`, `scripts/check.mjs`, `lefthook.yml`.
6. `docs: provenance, contributing, conduct, security, changelog, templates`.
7. `chore: v0.1.0` — full gate green, tag after human review.

Every phase ends with the gate green and `git status` clean. No `git add -A`, and history is never
rewritten.

## 10. Verification checklist

- [ ] validator exits 0 with both skills; a deliberately broken frontmatter exits 1 (the gate is real).
- [ ] `cargo test` passes; release build produces `cmt`; the wrapper finds it without `CMT_BIN`.
- [ ] DSH catalog lists `local-plan` and `comment-discuss`; both bodies load.
- [ ] `/local-plan` creates a plan; `cmt scan` → close → validate reaches exit 0.
- [ ] README catalog == provenance == on-disk skills (validator enforces).
- [ ] `git ls-files` contains no `.exe`, no secret, no file > 5 MB.
- [ ] copy `comment-discuss/` alone into a scratch repo; the skill still runs (portability).
- [ ] fresh `git clone` + `node scripts/check.mjs` on a clean machine.
- [ ] default branch is `master`; history intact (no force-push, one commit per phase).
- [ ] the language probe result is recorded and matches what shipped.

## 11. Assumptions and non-blocking decisions

·· remove Claude Code compatibility, it Claude Code compatible with industry standard it can pick up (closed)

- **Assumed:** owner handle `huandzh`; MIT in Huan Di's name; Windows authoring with cross-platform
  scripts; **DSH is the first client, and the open standard is the only compatibility contract** — a
  client that reads `AGENTS.md` and `.agents/skills` needs nothing from us, and no client-specific file
  is produced.
- **Resolved:** the language probe (§6) ran on 2026-11-18 — Chinese body, English description, thin
  English digest; no full translation. Changing a default is a one-line edit here, not a rewrite.

## 12. Out of scope

·· ok, delay these (closed)

Confirmed deferred on 2026-11-18: marketplace/registry publishing, GUI, telemetry, machine translation
of skill bodies, porting skills beyond the two named, plugin/bundle packaging (`.claude-plugin/`, DSH
bundle), and a generated `catalog.json` (revisit at ~5 skills). The repo stays small on purpose: two
habits, executed well, prove the shape.

## 13. Review log

14 in-file comments, reviewed and closed on 2026-11-18. All were L2, so their history stays inline in
this file and **no sidecar log was written** — `cmt close -n` records a note only when an L1 line moves
to the log (confirmed on a one-comment fixture: `0 log(s)`, note dropped; see §7, patch item 7). The
resolutions, in comment order:

| Line | Resolution                                                                     |
| ---- | ------------------------------------------------------------------------------ |
| 19   | §2 admits tooling: `cmt` ships inside the `comment-discuss` bundle.            |
| 39   | Branch is `master`; history is immutable — no force-push, one commit per phase. |
| 73   | `CLAUDE.md` dropped; `AGENTS.md` is the only contract.                         |
| 75   | `docs/coding-rules.md` added; `AGENTS.md` links it instead of inlining rules.  |
| 104  | Tree revised: no `.claude/`, no workflows, plus `coding-rules.md`, gate scripts, `references/en.md`. |
| 151  | Language probe run (2 arms, below); full translation rejected, Chinese body kept. |
| 153  | Fallback ships as `references/en.md`, a 20-line digest, marked non-authoritative. |
| 192  | No `CLAUDE.md` generated; a local one stays gitignored.                        |
| 243  | No GitHub Actions: local `scripts/check.mjs` + lefthook; fork PRs disabled.    |
| 260  | LICENSE copyright is Huan Di.                                                  |
| 272  | DSH-specific step removed; `local-plan` copied verbatim.                       |
| 274  | Port by copy + de-privatization patch list; no rewrite.                        |
| 345  | Claude Code compatibility removed; the open standard is the only contract.     |
| 356  | Deferred items confirmed and dated.                                            |

**Language probe (n=2, same model, 2026-11-18).** Two arms read `comment-discuss` in Chinese-only vs
Chinese + a 20-line English digest, then answered the same four questions on a probe document holding
one L1 comment, one L2 comment, a fenced `·`, and a quoted `> ·`.

| Arm           | Comment lines / levels | Close order   | Exit codes | Boundary traps | Score |
| ------------- | ---------------------- | ------------- | ---------- | -------------- | ----- |
| Chinese       | 2/2 correct            | largest-first | 2 → 0      | both correct   | 4/4   |
| + English     | 2/2 correct            | largest-first | 2 → 0      | both correct   | 4/4   |

Finding: the English digest bought **no measurable comprehension**; the Chinese-only skill was already
correct on every trap (fence/quote exclusion, close-only, descending order). Full translation rejected;
the thin digest kept as a human-facing fallback. Re-test only if a non-Chinese-reading model or harness
becomes a target, or a user reports confusion.
