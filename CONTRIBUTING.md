# Contributing

Read [AGENTS.md](AGENTS.md) first — it is the contract, for humans and agents alike.

## Setup

- Node.js 22+ — the gate and the validator are zero-dependency ESM.
- Rust stable — the `comment-discuss` skill vendors a Rust crate whose tests run in the gate.
- Optional: `lefthook install` to run the gate on pre-commit and pre-push.

```bash
git clone https://github.com/<owner>/mannerisms.git
cd mannerisms
node scripts/check.mjs
```

## The loop

1. Open an issue. Bugs and features use their templates; a new skill uses **skill proposal** and must
   name the habit it encodes, not the tool it wraps.
2. Branch from `master`.
3. Make one logical change; keep `SKILL.md` under 500 lines.
4. Run `node scripts/check.mjs` and paste its tail into the PR.
5. Fill every section of the PR template, including how the change was authored.

## What gets rejected

- A skill with no habit behind it ("it might be useful").
- A rewrite of a body that was only being ported.
- A committed binary, a generated file, or a private path.
- A PR that bundles unrelated changes.
- A skill that breaks when its directory is copied alone.

## Commit style

Subject ≤ 72 characters, imperative, naming the skill when it touches one. The body explains why and
names the gate that ran. See [docs/coding-rules.md](docs/coding-rules.md#commits).

## License

MIT. Contributions are accepted under the same license.
