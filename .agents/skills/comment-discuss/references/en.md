# comment-discuss — English digest (fallback)

Use when the user wants to discuss a file in place, list or close discussion comments, validate that
every comment was addressed, or mentions the `·` / `··` markers.

- Two markers, two close actions. `·` (L1): after the body is fixed, the whole line is removed from the
  document and appended to `<doc>.discuss-log.md`. `··` (L2): after the body is fixed, append
  ` (closed)` to that same line. `···` or an empty comment is malformed — `validate` reports it and only
  `--force` deletes it.
- `cmt` is the tool (Rust, zero dependencies): `scan`, read the section, edit the body, `close`,
  `validate` (exit 0 means the loop is closed).
- Only `cmt close` may close a comment. Never delete or hand-edit a comment line: the hash log and
  idempotency depend on the tool doing it.
- Line numbers move after a close, so prefer one `--all`; closing one by one means largest line first,
  or re-scan after each close.
- Exit codes: 0 closed or nothing to do, 1 usage or IO, 2 open comments remain, 3 protocol error.
- Do not close what you did not address; name it in the summary. Change only what the comment targets.
- A marker inside a fenced code block, or after a `>` quote, is not a comment.

`SKILL.md` (Chinese) is authoritative; this digest is a fallback for readers who do not read Chinese.
It is not a translation and must not be edited into a second source of truth.
