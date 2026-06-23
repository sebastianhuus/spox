status: completed

# Explicit subcommands

## Intent
Replace positional arguments with explicit subcommands so each mode can be allowlisted precisely in Claude Code (e.g. `spox list` or `spox view` instead of `spox *`). Mirrors the `gh` CLI pattern (`gh issue view`, `gh pr list`) where noun+verb subcommands are safe to grant individually.

## Acceptance criteria
- [x] `spox list` lists all specs with their status (replaces bare `spox`)
- [x] `spox list -c` also prints open criteria under each spec (replaces `spox -c`)
- [x] `spox view <spec>` prints the full raw spec file content (replaces `spox <spec>`)
- [x] `spox view -c <spec>` prints only open criteria in the dashboard format (replaces `spox -c <spec>`)
- [x] `spox <spec>` (positional form) still works but prints a deprecation warning directing users to `spox view <spec>`
- [x] Bare `spox` with no arguments still works but prints a deprecation warning directing users to `spox list`
- [x] All internal references (SKILL.md, skill files, README, help text) are updated to use the new subcommands
- [x] `spox help` lists `list` and `view` subcommands

## Notes
`-c` means the same thing in both contexts: show open criteria. `list` vs `view` cleanly separates all-specs from one-spec scope.

`spox view <spec>` prints the full raw markdown file; `spox view -c <spec>` shows only the open criteria dashboard — the two modes are intentionally distinct.

Deprecation warnings give a migration window before the positional forms are removed.
