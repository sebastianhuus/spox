---
description: Create a new spox spec. Use when the user wants to add a spec, track a new feature, or start new work in a .spox/ project.
---

## Current specs

!`spox 2>/dev/null || echo "(no spox project found)"`

## Project format conventions

!`f="$(git rev-parse --show-toplevel 2>/dev/null)/.spox/.format.md"; test -f "$f" && cat "$f" || echo "(no .format.md — use defaults)"`

## Instructions

Create a new spec file in `.spox/`.

If the user named a feature in their message, derive the spec name from it. Otherwise ask for a feature name first. Names must be short kebab-case identifiers (`map-grid`, `auth-refresh`) — not task descriptions (`implement-map`, `fix-auth-bug`).

Use this template:

```
status: draft

# [Feature name]

## Intent
[One paragraph: what this is for and why it exists.]

## Acceptance criteria
- [ ] [Something verifiable — visible in the running app, covered by a test, or confirmed by output]
- [ ] [Another criterion]

## Notes
[Design decisions, open questions, or implementation detail. Omit section if empty.]
```

Fill in the template from the user's description. Write the result to `.spox/<name>.md`. After writing, run `spox <name>` to confirm it was picked up.
