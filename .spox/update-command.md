status: draft

# Update Command

## Intent
Add a `spox update` command that pulls the latest spox changes from the remote repository. Mirrors the auto-update logic already used in other code paths (e.g. skill auto-update) so users can trigger a manual refresh without reinstalling.

## Acceptance criteria
- [ ] `spox update` pulls the latest changes from the remote (e.g. `git pull` in the spox install directory)
- [ ] Command prints a confirmation message showing what was updated (or "already up to date")
- [ ] Reuses or delegates to the existing auto-update logic rather than duplicating it
- [ ] Exits with a non-zero status code if the update fails, with a useful error message

## Notes
Check how auto-update is triggered in other code paths (skill-auto-update, install scripts) to find the shared update function to call.
