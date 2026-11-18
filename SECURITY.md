# Security

## The trust model

A skill is executable instruction: prose an agent follows, plus scripts it may run. Installing one is
trusting it. This repo narrows that surface — it cannot remove it.

- No network access from any script.
- The only binary is `cmt`, built locally from the vendored crate; nothing prebuilt is committed.
- That crate has zero dependencies, so the supply chain is the Rust standard library and nothing else.
- The validator refuses committed binaries, oversized files, and links that escape a bundle.

## Reporting

Report a vulnerability privately through GitHub's **Report a vulnerability** form on the Security tab
of this repository. Do not open a public issue.

Include the skill and file, the impact, and a minimal reproduction. Expect an acknowledgement within a
week.

## Scope

**In scope:** the scripts under `.agents/skills/*/scripts/`, the vendored `cmt` crate, and
`scripts/validate-skills.mjs` / `scripts/check.mjs`.

**Out of scope:** the behavior of third-party clients that read `SKILL.md`, and an agent following a
skill's prose (that is the skill working as intended, not a vulnerability).

## Supported versions

`master` only; releases are tagged `vX.Y.Z`.
