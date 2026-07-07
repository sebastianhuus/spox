status: reference

# Spec format

Each spec is a single Markdown file in this folder. The first line is always:

```
status: <value>
```

Valid statuses:

| Status | Meaning |
|---|---|
| `draft` | Being written, not ready to implement |
| `ongoing` | Actively being built |
| `completed` | Built and verified |
| `discarded` | Decided against — keep the file for context |

## File naming

Use short kebab-case names that describe the feature or system, not the task.
Good: `map-grid.md`, `pawn-needs.md`, `trust-system.md`
Avoid: `implement-map.md`, `fix-camera.md`

## Structure

After the status line, each spec follows this shape:

```markdown
status: ongoing

# Title

## Intent
What this is for and why it exists. One paragraph max.

## Acceptance criteria
- [ ] Something verifiable — can be checked by running the game or a test
- [ ] Another criterion

## Notes
Design decisions, open questions, or implementation detail worth preserving.
```

Acceptance criteria should be things you can actually check — visible in the running game,
covered by a test, or confirmed by reading the output. Vague criteria ("feels good", "works well")
belong in Notes, not criteria.
