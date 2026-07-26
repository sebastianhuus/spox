status: completed
date: 2026-07-27
tagline: add spox list -t to show a glanceable one-line description per spec

# list-tagline

## Intent
`spox list -a` shows spec names and statuses, but nothing about what a spec actually does — you have to `spox view` each one to remember. Add a `-t`/`--tagline` flag to `spox list` that prints a glanceable, one-line description per spec, sourced from a new optional `tagline:` frontmatter field. Specs without a tagline must continue to list normally (no crash, no forced migration).

## Acceptance criteria
- [x] A spec file can have a `tagline: <text>` line in its frontmatter (alongside `status:`/`date:`), parsed by `parse_spec()`
- [x] `spox list -t` (and `--tagline`) prints each spec's tagline as a one-line, truncated-if-needed description next to its name/status
- [x] Specs with no `tagline:` line still appear under `spox list -t` without error (e.g. blank or a placeholder in place of the tagline)
- [x] `-t` composes with the existing `-a` and `-c` flags in any order (e.g. `spox list -a -t`, `spox list -t -c`), consistent with how `-a`/`-c` already combine
- [x] `spox list` without `-t` is unchanged — no tagline column appears
- [x] `spox help` documents the new flag in the same one-line style as the existing `list` entry

## Notes
- Precedent: `.spox/list-active-flag.md` added `-a` the same way — reused `cmd_list()`'s existing filter param rather than new plumbing. `-t` is additive display, not a filter, so it likely needs its own bool param on `cmd_list()`.
- `format.md` only documents `status` and `date` as frontmatter fields today; `tagline` would be the first addition beyond those and should be documented there too.
- Flag parsing in `main()` is manual slice-matching (no clap) — each valid combination of `list` flags is its own `match` arm. Adding a third flag (`-t`) alongside `-a`/`-c` means either more explicit permutation arms or a small refactor to parse flags order-independently. Worth deciding during implementation rather than pre-committing here.
- Open question: how to truncate long taglines (fixed char width? terminal width via a TTY check, similar to the glow-output spec's TTY detection?). Left to implementation.
