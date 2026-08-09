status: completed

# Set status

## Intent
`spox status <spec> <value>` already exists and updates a spec's status line without opening an editor. The gap is that it accepts any arbitrary string — there's no guard against typos or values outside the defined set (`draft`, `ongoing`, `pending-verification`, `completed`, `discarded`). This spec adds validation so the command fails fast with a clear message instead of silently writing garbage into the file.

## Acceptance criteria
- [x] `spox status <spec> <invalid>` exits with a non-zero code and prints the set of valid values
- [x] `spox status <spec> <valid>` continues to work as before, printing `<spec>: <old> → <new>`
- [x] Valid statuses are the same set defined in `format.md`: `draft`, `ongoing`, `pending-verification`, `completed`, `discarded`
- [x] `spox status` with no arguments prints usage, including the list of valid values

## Notes
Valid statuses are read from `format.md` or hardcoded to match it — whichever avoids duplicating the source of truth. The `parse_spec` function already reads `status:` from the first line; this builds on the same convention.
