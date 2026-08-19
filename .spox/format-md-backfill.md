status: completed

# Format-md backfill

## Intent

`spox init` writes `.spox/.format.md` when creating a new project, but it bails early if `.spox/` already exists. Any repo initialized before `.format.md` was introduced will never get the file. The `new-spec` skill tries to `cat` it at load time and silently gets nothing (or errors), so format conventions are invisible to the AI. Spox should automatically write `.format.md` whenever it runs in a `.spox/` directory that doesn't have one yet — and, since the binary's embedded format doc can change across versions, it should keep an existing `.format.md` in sync the same way `.claude/skills/` files already are (`skill_status()` / `write_skill_file()` in [src/main.rs](src/main.rs)), rather than writing it once and never touching it again.

## Acceptance criteria

- [x] When any spox command runs in a `.spox/` dir that lacks `.format.md`, the file is created automatically before the command proceeds
- [x] A one-line notice is printed when the backfill occurs (e.g. `spox: created .spox/.format.md`)
- [x] When `.format.md` exists but its content differs from the embedded `FORMAT_MD` const, it is overwritten and a one-line notice is printed (e.g. `spox: updated .spox/.format.md`), mirroring the outdated-skill-file update notice
- [x] When `.format.md` exists and already matches `FORMAT_MD`, nothing is written and no output is produced
- [x] `spox init` behaviour is unchanged — it still creates `.format.md` on first init

## Notes

`FORMAT_MD` is already embedded as a const in [src/main.rs:12](src/main.rs). The backfill/sync check should run early in the main dispatch path — after the `.spox/` dir is located but before any subcommand executes. It can reuse the same hash-comparison approach as `skill_status()`/`write_skill_file()` rather than introducing a new mechanism: missing → create, present-but-different → overwrite, present-and-matching → no-op.
