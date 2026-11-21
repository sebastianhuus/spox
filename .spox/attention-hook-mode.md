status: pending-verification
date: 2026-11-21
tagline: `spox attention --hook` — the same report, at most once a day per project, for session-start hooks

# Attention hook mode

## Intent
`spox attention` is only useful if someone sees it, and the user wants it to appear on its own at session start, like their stale-repos hook. A hook runs on every session start, so printing the report every time would be noise. `spox attention --hook` prints exactly the report that plain `spox attention` prints, but only if this project has not shown one in the last 24 hours, and records when it did. Everything else stays silent, so a global hook (`spox attention --hook 2>/dev/null || true`) can pass the output straight to the agent.

## Acceptance criteria
- [ ] `spox attention --hook` takes the same options as `spox attention` (`--scope`, `-C`, `SPOX_PROJECT`, `SPOX_ATTENTION_SCOPE`) and uses the same project discovery, scope rules and report; plain `spox attention` without `--hook` is unchanged
- [ ] When the project is due, `--hook` prints a report byte-for-byte identical to what plain `spox attention` prints in the same state, then exits 0
- [ ] A project is due when it has no recorded report, its last recorded report is 24 hours (86400 seconds) or more old, or the recorded time is in the future; otherwise `--hook` prints nothing, writes nothing, and exits 0
- [ ] The throttle is keyed by the project containing the current directory (the nearest `.spox` at or above it), not by scope or flags; two different projects throttle independently
- [ ] When there is nothing to report, or there is no `.spox` at or above the current directory, `--hook` prints nothing, records nothing (so the next report is not delayed), and exits 0
- [ ] State lives in `~/.cache/spox/attention_hook` (next to the existing `last_check`), one line per project: `<unix seconds>`, a tab, the project's absolute path; the directory is created if missing, and lines older than 24 hours are dropped whenever the file is rewritten
- [ ] The time is recorded before the report is printed; if the state cannot be written (missing and uncreatable directory, read-only cache, full disk), `--hook` prints nothing, writes nothing to stderr, and exits 0
- [ ] A report identical to the previous one still counts as shown: the rule is simply once per 24 hours per project, with no comparison of report contents
- [ ] Plain `spox attention` always prints when there is something to report and never reads, creates or changes the throttle state
- [ ] `--hook` stays read-only for the project, never runs the background self-update or the skill auto-update, writes nothing to stderr for runtime errors, and exits 0; only usage errors (an unknown flag, an invalid `--scope`) still print an `error:` line and exit 1, as for plain `attention`
- [ ] Tests (temp project, `HOME` pointed at a temp dir) cover: due (no state), not due (just reported), due again after 24 hours (state written with an old time), unwritable state, no `.spox`, nothing to report, two projects throttled independently, plain `attention` leaving the state untouched, and `--hook` output equal to plain `attention` output
- [ ] `spox help`, `readme.md` and the `spox` skill (`skills/spox/SKILL.md`, then `spox skill install`) document `--hook` and say it is meant for session hooks, not for agents to run by hand

## Non-goals
- Wiring the hook into `settings.json`, and the existing local beta-reminder hook
- Any window other than 24 hours, or a flag to change it
- Reporting only when the report changed
- New statuses, priority lanes, an opt-out for self-update, or changes to the report format

## Notes
- Implementation shape: build the report in one function that both modes call, so `--hook` cannot drift from `attention`; `--hook` only decides whether to print. `main()` already skips the self-update whenever the first argument is `attention`, which covers `--hook`.
- Cache path: reuse the `$HOME/.cache/spox/` directory that `last_check_path()` already uses. A single small file rewritten on each print is enough; entries older than 24 hours are useless (they would be "due" anyway), so dropping them keeps it tiny.
- Decision: an unwritable state means no output. The alternative, printing anyway, would repeat the report at every session start whenever the cache is broken, which is the spam this mode exists to prevent. The cost is that a broken cache silently disables the hook, since `--hook` stays quiet by design; if the report stops appearing, check the permissions of `~/.cache/spox/`.
- Decision: identical reports count as shown. Simple and predictable; a "changed since last time" rule would need to store a hash of the report and would make the format harder to change.
- Open question: two sessions starting in the same project within the same moment can both see "due" and both print, since there is no locking. Acceptable for a once-a-day nudge; flag if it turns out to matter.
- Open question: should a usage error under `--hook` also be silent? As drafted it is not: a bad flag is a configuration mistake worth seeing when the hook is first set up by hand, and the usual `2>/dev/null || true` wrapper hides it in daily use anyway.
- zsh-completion: `attention` is already in `known_cmds` in `src/main.rs`, and the planned `KNOWN_COMMANDS` check only covers command names, not flags. When completion is implemented it should offer `attention` and its flags (`--scope all|nearest`, `--hook`) explicitly; that spec's criteria do not mention `attention` yet.
