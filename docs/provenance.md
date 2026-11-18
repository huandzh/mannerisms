# Provenance

Where each skill came from, under what license, and what changed. The validator requires a row here
for every directory under `.agents/skills/`.

| Skill | Origin | Author | License | What changed |
| --- | --- | --- | --- | --- |
| `local-plan` | `~/.agents/skills/local-plan`, via `ah-my-skills` | Huan Di | MIT | Copied verbatim; no patch. |
| `comment-discuss` | `cpds_lead/.claude/skills/comment-discuss`, via `ah-my-skills`, plus `cpds_lead/tools/comment_discuss` (the `cmt` crate) | Huan Di | MIT | Client-specific paths rewritten bundle-relative; the prebuilt `cmt.exe` is no longer committed; the crate is vendored under `tools/comment_discuss`; an English digest added under `references/`; the body is otherwise unchanged. |

## Redistribution

Both skills are MIT. A copied bundle should keep this information reachable by its user — this row, or
a short `references/PROVENANCE.md` inside the copied directory.
