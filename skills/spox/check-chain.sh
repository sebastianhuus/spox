#!/usr/bin/env bash

COMMAND=$(jq -r '.tool_input.command // ""' 2>/dev/null) || COMMAND=""

# Block only when spox is the actual executable — first token of any chain segment.
# Avoids false positives where "spox" appears in a file path, commit message, or string.
MATCHED_SEGMENT=""
while IFS= read -r segment; do
  trimmed="${segment#"${segment%%[![:space:]]*}"}"
  first_token="${trimmed%% *}"
  if [ "$first_token" = "spox" ]; then
    MATCHED_SEGMENT="$trimmed"
    break
  fi
done < <(printf '%s' "$COMMAND" | sed 's/&&/\n/g; s/||/\n/g; s/;/\n/g')

if [ -n "$MATCHED_SEGMENT" ]; then
  jq -n --arg reason "Chained spox command blocked (matched: \"$MATCHED_SEGMENT\"). Each spox call must be a separate tool use — do not chain with &&, ||, or ;. Run the command alone so its output can be reviewed before the next call." '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: $reason
    }
  }'
fi
