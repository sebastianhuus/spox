status: ongoing
date: 2026-11-02
tagline: find .spox below the git root, pick a project with -C / SPOX_PROJECT

# Nested project discovery

## Intent
Let spox work when the `.spox/` folder is not at or above the current directory, for example a notes repo whose git root holds several projects and the spox project lives one level down. Today `find_spox_dir()` only walks up from cwd, and the `new-spec` skill hardcodes `$(git rev-parse --show-toplevel)/.spox`, so both fail there. Repos with `.spox/` at or above cwd must behave exactly as before.

## Acceptance criteria
- [ ] With a `.spox` in cwd or any ancestor, the nearest one is used, exactly as before (no downward search, no extra output)
- [ ] With none above, spox searches downward (depth 5; skipping hidden dirs, `node_modules`, `target`; not following symlinks) from the git root (only inside a git repo) and uses a single match, printing a one-line note to stderr only
- [ ] With several matches below, every command exits 1 with "multiple spox projects found" naming each path and pointing at `-C` / `SPOX_PROJECT`
- [ ] With no `.spox` anywhere, the existing "no .spox directory found" error is unchanged
- [ ] `spox -C <dir> <command>` behaves as if spox was started in `<dir>`; `<dir>` may be the project, a subdir of it, or the `.spox` dir itself; a nonexistent `<dir>` is an error
- [ ] `SPOX_PROJECT=<dir>` has the same meaning as `-C <dir>`, and `-C` wins when both are set
- [ ] `spox path` prints the resolved absolute `.spox` directory and `spox format` prints that project's `.format.md`; both are single commands needing no shell substitution
- [ ] The `new-spec` and `spox` skills use `spox path` / `spox format` instead of `git rev-parse`, and their `allowed-tools` cover the `-C` forms
- [ ] Discovery, `-C`, `SPOX_PROJECT`, `path` and `format` are covered by tests that cannot trigger the daily self-update

## Non-goals
Merged or grouped `list` output across several projects. A second flag name such as `--project`. Changing where `spox skill install` puts skills (still the git root's `.claude/skills`).

## Notes
Decided with Sebastian on 2026-11-02: multiple matches are an error (spec names can collide across projects), depth 5.

The downward search only runs inside a git repo. Crawling an arbitrary directory such as `~` is slow and can trigger macOS privacy prompts (Library, Pictures); outside git, `-C` is the way in. This narrows the original proposal, which also searched from cwd outside git.

`-C` is implemented like `git -C`: change the process cwd before dispatch, so discovery, `init` and `skill install` all follow it.
