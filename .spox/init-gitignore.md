status: completed

# Init gitignore

## Intent
When `spox init` creates `.spox/`, it should ensure `.spox/.cache/` is never tracked by git. It does this with a self-contained `.spox/.gitignore` rather than editing the project's own `.gitignore` — this keeps the ignore rule scoped to `.spox/`, works no matter where `.spox/` lives in the tree, and requires no git-root detection.

## Acceptance criteria
- [x] Running `spox init` creates `.spox/.gitignore` containing `.cache/`
- [x] The project's own `.gitignore` (if any) is never modified by `spox init`
- [x] The init output reports the file (e.g. `created: .spox/.gitignore`)

## Notes
Previous design (superseded): walked up from CWD for a `.git` directory and appended `.spox/.cache/` to the repo-root `.gitignore`. Dropped in favor of the self-contained approach above — simpler, and doesn't touch a file the user owns.
