status: completed

# Skill auto-update

## Intent

When spox detects that installed Claude Code skill files are outdated (content differs from the version embedded in the binary), it should automatically update them without any user action. Currently, `maybe_suggest_install()` prints a message asking the user to run `spox skill install` manually — that's friction that causes repos to accumulate stale skills. The binary-embedded skill is always authoritative; no per-repo local variant should exist or win.

## Acceptance criteria

- [x] When any installed skill is outdated, spox auto-runs the install and prints a one-line notice per updated file (e.g. `spox: skill updated spox/SKILL.md`)
- [x] The "run `spox skill install` to update" suggestion message is removed from console output
- [x] No user action is required to keep skills current — update happens inline on next spox invocation
- [x] When skills are already current, the auto-update path is silent (no output)
- [x] Existing `spox skill install` command still works explicitly for debugging / force-reinstall

## Notes

`maybe_suggest_install()` in [src/main.rs:81-97](src/main.rs) is the call site to replace — instead of printing a suggestion, call `cmd_skill_install()` directly (or extract the write logic into a shared fn). The `write_skill_file()` return values (`unchanged`/`updated`/`installed`) can drive the per-file notice output.

The "upstream SPOX skill" principle: the binary-embedded consts (`SKILL_MD`, `NEW_SPEC_SKILL_MD`, etc.) are the single source of truth. No per-repo `.claude/skills/` override should shadow them. Auto-update enforces this invariant automatically.
