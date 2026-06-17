#!/usr/bin/env bash

COMMAND=$(jq -r '.tool_input.command // ""' 2>/dev/null) || COMMAND=""

if echo "$COMMAND" | grep -qw 'spox' && echo "$COMMAND" | grep -qE '(&&|\|\||;)'; then
  jq -n '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: "deny",
      permissionDecisionReason: "Run spox commands one at a time — do not chain them with &&, ||, or ;. Make each spox call a separate tool use so its output can be reviewed before the next one runs."
    }
  }'
fi
