status: draft

# List Active Flag

## Intent
Add a `-a` / `--active` flag to `spox list` that hides specs in the "completed" category so users can focus on work still in progress. The completion detection reuses the existing `status_group()` substring matching ("complet", "done", "finish", "implement"), meaning any status the user sets that reads as done is automatically excluded — no configuration needed for the common cases.

## Acceptance criteria
- [ ] `spox list -a` and `spox list --active` both produce output that omits specs in the completed group (status containing "complet", "done", "finish", or "implement") and specs with status `discarded`
- [ ] Specs with any other status (e.g. `draft`, `ongoing`) appear normally under their existing group headings
- [ ] `spox list` without the flag retains its current behaviour exactly — no regressions
- [ ] When all non-completed specs are absent (empty project), the command exits cleanly with no output or a "nothing active" notice
- [ ] Help text for `spox list` documents the new flag

## Notes
`cmd_list()` in `src/main.rs` already accepts a `filter` parameter that is always `None` today. The flag wires up a completed-group predicate to that parameter rather than introducing new filtering infrastructure.

The default (show all specs) is intentional, not a fallback. Agents scanning the spec list to understand prior work, find relevant implementations, or avoid duplication benefit from seeing completed specs — they provide context that active specs alone don't. `--active` is an opt-in for human-focused views where the signal-to-noise ratio matters.

`discarded` is excluded explicitly by string match (it doesn't fall into the completed-group keywords), alongside the completed group.
