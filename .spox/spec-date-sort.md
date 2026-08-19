status: completed

# Spec date sort

## Intent
Specs are currently sorted by status group, then alphabetically by name (`cmd_list()`, [src/main.rs:801-858](../src/main.rs)). Within a status group there's no way to tell which specs are newer, which matters most for `draft` — the group most likely to have many active proposals at once. Rather than adding a CLI tool for spec creation, embed a `date:` field in the frontmatter template so the writing agent fills it in directly, and use that field as a secondary sort key.

## Acceptance criteria
- [x] `format.md` documents a `date: YYYY-MM-DD` line as the second frontmatter line (after `status:`)
- [x] The shipped template (`skills/spox/spec-template.md`) includes the `date:` line, and the `new-spec` skill instructs the agent to fill it in with the current date
- [x] `parse_spec()` ([src/main.rs:90-103](../src/main.rs)) parses the `date:` line into the `Spec` struct; a missing or unparseable date does not error — it sorts as oldest (empty string is fine)
- [x] `cmd_list()`'s sort ([src/main.rs:841-845](../src/main.rs)) orders by status group, then by date descending (newest first) within the group, then by name as a final tiebreaker
- [x] Existing specs without a `date:` line continue to list and view correctly (no crash, no required backfill)

## Notes
- No new CLI subcommand (`spox create`) — the agent already writes the file in one step via the `new-spec` skill; a wrapper command adds a process hop for no benefit.
- Considered using file mtime instead of a frontmatter field: rejected because mtime is clobbered by edits, git clone, and checkout, so it doesn't reliably reflect creation order.
- Backfilling dates onto the 8 existing specs is out of scope for this spec; can be done manually or as a follow-up if sorting undated specs together (e.g., all last) proves annoying in practice.
- **Depends on [format-md-backfill](format-md-backfill.md).** `format.md`'s `date:` documentation only reaches projects that already have `.spox/.format.md` if that spec's sync mechanism ships first — otherwise existing projects' `.format.md` stays stale indefinitely (it's currently write-once via `spox init`). Land format-md-backfill before or alongside this spec.
