status: completed

# skill-init-validator

## Intent
Skills that probe for spox presence (e.g. `new-spec`, `spox`) run a shell check like
`!spox 2>/dev/null || echo "(no spox project found)"` during init. The command validator
rejects this because it sees `spox` as a command requiring approval, even though it's a
read-only probe. This blocks skill execution entirely. We should ensure standalone `spox`
invocations (with no mutating subcommands) are auto-allowed by the validator, or adjust
skill init probes so they don't trigger the permission check.

## Acceptance criteria
- [x] Running the `new-spec` skill no longer fails with a permission check error on the `spox` probe command
- [x] Other skills that call `spox` during init (e.g. `spox list`, `spox check`) are similarly unblocked
- [x] The fix does not bypass approval for genuinely dangerous commands — only read-only spox invocations are whitelisted

## Notes
The failing pattern was: `!spox 2>/dev/null || echo "(no spox project found)"` — the validator flagged the `spox` part as requiring approval. Root cause is likely the check-chain hook or the command allowlist not recognising bare `spox` calls as safe. Two candidate fixes: (1) add `spox` (with no subcommand or with read-only subcommands) to the auto-allow list in settings, or (2) rewrite skill probes to use `command -v spox` instead of running spox directly.
