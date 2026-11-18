# Skill anatomy

The format the validator enforces. It follows the [Agent Skills specification](https://agentskills.io/specification)
with the fields DSH understands natively.

## A bundle is a directory

```
<skill>/
├── SKILL.md          required; frontmatter + the decision procedure
├── references/       optional prose, loaded on demand
├── scripts/          optional executables
├── examples/         optional fixtures the smoke test uses
├── tools/            optional buildable source a script compiles
└── bin/              optional local build output; gitignored
```

## Frontmatter

Only `name` and `description` are required. The full table and limits live in
[coding-rules.md](coding-rules.md#frontmatter). The short version: the name is the directory, and the
description is the interface — what it does, when to invoke it, and when not to.

## The body

- opens with the decision: when to use this skill, and when not to;
- then the procedure, numbered, small enough to follow without re-reading;
- then boundaries and failure modes — what the tool cannot do, and what to do instead;
- links to `references/` instead of inlining long tables.

## Discovery budget

Discovery reads the frontmatter only: one summary per skill reaches the model, every session. Loading
reads the body once. Neither ever reads `references/` on its own, so a large reference file costs
nothing until an agent opens it.

## Checklist before commit

- [ ] `name` equals the directory, lowercase-hyphen, ≤ 64 chars
- [ ] `description` ≤ 1024 chars, with a trigger and an anti-trigger
- [ ] body ≤ 500 lines (ideally under 150)
- [ ] every relative link resolves inside the bundle
- [ ] no binary, no private path, no secret
- [ ] a README catalog row and a `docs/provenance.md` row exist
- [ ] `node scripts/check.mjs` is green
