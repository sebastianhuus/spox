status: completed

# check-chain false positive fix

## Intent
The `check-chain.sh` pre-tool hook fires on any Bash command that contains the word "spox" and a chain operator (`&&`, `||`, `;`). This causes false positives when the word "spox" appears as data inside another command — most commonly a `git commit -m` message. The fix should restrict the match to commands where `spox` is actually being invoked as a program, and the error message should be specific enough that an agent can diagnose the block without needing to read the script.

## Acceptance criteria
- [x] `git commit -m "update spox; bump version"` is not blocked by the hook
- [x] `spox check foo && spox check bar` is still blocked
- [x] `spox list; spox check foo` is still blocked
- [x] The `permissionDecisionReason` names what pattern was matched (e.g. `spox check foo && ...`) so the agent knows exactly what triggered the block

## Notes
Current match logic: `grep -qw 'spox'` on the full command string, which hits commit messages, echo statements, comments, etc.

Tighter match: check whether the command invokes `spox` as an executable — a token at the start of the command or after a chain operator — rather than just containing the word anywhere. One approach is to extract the first token of each semicolon/`&&`/`||`-delimited segment and check if any equals `spox`.

The error reason should quote or describe the offending pattern so the agent can self-correct without reading the hook source.
