status: completed

# check-chain hook scope

## Intent
The check-chain hook should only block actual chained spox invocations in Bash commands, not incidental matches inside commit message text, comments, or heredoc content passed to other tools like `git commit -m`. Currently any Bash tool use whose full command string contains a spox subcommand name (e.g. "spox init", "spox check") anywhere — including in multi-line string arguments — triggers a false positive block.

## Acceptance criteria
- [x] `git commit -m "message mentioning spox init or similar"` is not blocked by the hook
- [x] A Bash tool use that genuinely chains spox (e.g. `spox list && spox check *`) is still blocked
- [x] The hook correctly ignores spox-keyword occurrences inside heredoc bodies and quoted string arguments to non-spox commands

## Notes
Reproducer: `git commit -F /tmp/msg.txt` where the message body contains "spox init now writes..." was blocked during the auto-allow-check implementation session. The fix likely involves extracting only the leading command token (before any `|`, `&&`, `;`, or string argument boundary) rather than scanning the entire raw command string.
