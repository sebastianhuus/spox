status: completed

# Self-update command

## Intent
Spox already has a background `maybe_self_update()` that silently `git pull`s and rebuilds once every 24 hours on startup ([src/main.rs:801-825](../src/main.rs)). There's no way to trigger this on demand or see what happened — a user who wants the latest build right now has to `cd` into the repo and run `git pull && cargo build --release` themselves. `spox update` exposes the same pull-and-rebuild logic as an explicit, user-facing command with visible output, independent of the 24-hour throttle.

## Acceptance criteria
- [x] `spox update` runs `git pull` in the spox repo directory (via `spox_repo_dir()`), then `cargo build --release` if new commits were pulled
- [x] `spox update` ignores the 24-hour throttle used by the background check (`should_check_updates()` / `mark_checked()`) — it always runs when invoked directly
- [x] Output is visible to the user: prints whether the repo was already up to date, what commit it updated to, and whether the rebuild succeeded — unlike the silent background path
- [x] If the binary isn't running from a git checkout (`spox_repo_dir()` returns `None` or `.git` doesn't exist), `spox update` prints a clear error and exits non-zero instead of failing silently
- [x] If `git pull` fails (e.g. local changes, no network), `spox update` prints the failure and exits non-zero without attempting a rebuild
- [x] If `cargo build --release` fails after a successful pull, `spox update` prints the build failure and exits non-zero
- [x] `spox help` lists `update` alongside the other subcommands ([src/main.rs:903-916](../src/main.rs))
- [x] Running `spox update` still updates the throttle cache (`mark_checked()`) so it doesn't trigger a redundant background check immediately after

## Notes
- `cmd_update()` should share the pull/rebuild logic with `maybe_self_update()` rather than duplicating it — factor out a common helper that both the background path and the explicit command call, with the throttle check and output verbosity as the difference between them.
- Follows existing conventions: `cmd_<subcommand>()` naming, `eprintln!("spox: <action> <target>")` status style, `std::process::Command` for `git`/`cargo` invocations, non-zero exit + `error:` prefix on failure.
- Add a match arm in `main()`'s command dispatch (around [src/main.rs:918-995](../src/main.rs)) for `[a] if a == "update"`.
