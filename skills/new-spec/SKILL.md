---
description: Create a new spox spec. Use when the user wants to add a spec, track a new feature, or start new work in a .spox/ project.
---

## Current specs

!`spox list -c`

## Project format conventions

!`cat "$(git rev-parse --show-toplevel 2>/dev/null)/.spox/.format.md" 2>/dev/null || true`

## Instructions

Create a new spec file in `.spox/`.

If the user named a feature in their message, derive the spec name from it. Otherwise ask for a feature name first. Names must be short kebab-case identifiers (`map-grid`, `auth-refresh`) — not task descriptions (`implement-map`, `fix-auth-bug`).

### Step 1: Explore the repo

Before drafting, use the Agent tool with `subagent_type=Explore` to scan for relevant context. Give it this prompt (substituting the feature name):

> Search this repo for context relevant to a new spec called "<feature-name>". Find: (1) any existing .spox/ specs with similar names or overlapping topics, (2) source files or modules the feature is likely to touch — grep for the feature name and related terms, (3) naming or structural conventions visible in adjacent code. Return a concise summary: related specs, relevant file paths, conventions to follow. If nothing relevant is found, say so briefly.

Use the findings to ground the spec. Do not ask the user for anything the Explore agent already resolved. If nothing relevant is found, proceed directly to drafting.

### Step 2: Write the spec

Use this template:

```
status: draft
date: [YYYY-MM-DD, today's date]

# [Feature name]

## Intent
[One paragraph: what this is for and why it exists.]

## Acceptance criteria
- [ ] [Something verifiable — visible in the running app, covered by a test, or confirmed by output]
- [ ] [Another criterion]

## Notes
[Design decisions, open questions, or implementation detail. Omit section if empty.]
```

Fill in the `date:` line with today's date. Fill in the rest of the template from the user's description and the Explore findings. Write the result to `.spox/<name>.md`. After writing, run `spox view <name>` to confirm it was picked up.
