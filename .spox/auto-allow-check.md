status: completed

# Auto-allow spox check

## Intent
Now that each checkpoint has a stable hex identifier, agents can safely re-run `spox check <label>` without risk of ambiguity or runaway chaining. The init script and skill install script should automatically add a `spox check *` permission allowlist entry so users aren't prompted on every check invocation.

## Acceptance criteria
- [x] `spox init` adds `spox check *` to the project's `.claude/settings.json` allowlist
- [x] The skill install script (used by `spox install` or equivalent) also adds the same allowlist entry
- [x] Existing projects that already have the entry are not duplicated
- [x] Running `spox check <label>` in an agent context does not trigger a permission prompt after init

## Notes
The motivation: before stable hex labels, chaining `spox check` could be confusing because criterion numbers shifted as specs changed. With stable identifiers that concern is gone, making blanket allowlisting safe.
