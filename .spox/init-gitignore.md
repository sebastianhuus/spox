status: draft

# Init gitignore

## Intent
When `spox init` creates `.spox/` in a git repo, it should automatically append `.spox/.cache/` to the repo's `.gitignore` (or create one if absent). The cache is an implementation detail and should never appear as an untracked file.

## Acceptance criteria
- [ ] Running `spox init` in a git repo with an existing `.gitignore` appends `.spox/.cache/` to it
- [ ] Running `spox init` in a git repo with no `.gitignore` creates one containing `.spox/.cache/`
- [ ] Running `spox init` outside a git repo skips the `.gitignore` step silently
- [ ] If `.spox/.cache/` is already present in `.gitignore`, it is not added again
- [ ] The init output reports the `.gitignore` action (e.g. `updated: .gitignore`)

## Notes
Git root detection: walk up from CWD looking for a `.git` directory, same pattern already used for `.spox/` discovery. The `.gitignore` to update is the one at the git root.
