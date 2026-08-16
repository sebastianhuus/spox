status: completed

# Glow output rendering

## Intent
When the `glow` CLI (charmbracelet) is available on the user's PATH and stdout is a TTY,
spox pipes markdown output through it for pretty terminal rendering. Commands that emit
spec content (`view`) benefit most; plain structured output (status lines, error messages)
is not piped. If `glow` is absent or stdout is not a TTY, output falls back to the
current plain-text behaviour with no error. Users can opt out via a `SPOX_NO_GLOW`
environment variable.

## Acceptance criteria
- [x] `spox view <spec>` pipes markdown content through `glow` when `glow` is in PATH and stdout is a TTY
- [x] The glow invocation does not spawn a pager (use `glow --pager=false -` or equivalent)
- [x] If `glow` is not in PATH, `spox view` output is unchanged from today
- [x] If stdout is not a TTY (e.g. piped to `grep`), glow is skipped and raw markdown is printed
- [x] Setting `SPOX_NO_GLOW=1` disables glow rendering even when it would otherwise apply
- [x] Non-markdown output (status change lines, error messages, `spox list` table) is not affected

## Notes
The `terminal_size` crate is already imported in `src/main.rs` — the same pattern can guard
the glow check. Spawn glow as a child process with stdin piped, write the markdown, then
wait. The `which`/`std::process::Command` approach avoids a new crate dependency.
