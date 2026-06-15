---
name: spox
description: Use this skill when the user wants to check, list, or update project specs tracked in a .spox/ directory. Trigger whenever the user mentions "spox", asks about spec status, wants to see what's in progress, references a .spox directory, asks about open criteria or unchecked tasks in specs, or wants to create or update a spec file. When in doubt, use this skill — it's the right tool any time specs or project status tracking come up.
---

## What spox does

`spox` is an installed CLI tool that lists and updates spec statuses from a `.spox/` directory, walking up the tree from CWD (git-style) to find it. Run it via Bash.

## Commands

```
spox                        # list all specs with their status
spox -c                     # also show numbered open criteria under each spec
spox --criteria             # same as -c
spox init                   # create .spox/ in the current directory (new projects)
spox check <spec> <n>       # check off the nth open criterion (1-indexed, matches spox -c output)
spox status <spec> <value>  # set the status field of a spec
spox skill install          # copy this skill into the project's .claude/skills/
```

## Reading the output

Each line is `<spec-name>  <status>`. With `-c`, open criteria appear numbered below:

```
auth      in-progress
  ┣━ 1. Write token refresh logic
  ┗━ 2. Add integration tests
parser    done
```

The numbers from `spox -c` are the indices to use with `spox check`.

## Checking off criteria

`spox check <spec> <n>` marks the nth open criterion as done. If it was the last open criterion, the spec's status is automatically set to `completed`.

```
$ spox check auth 1
checked: auth #1 — Write token refresh logic

$ spox check auth 1
checked: auth #1 — Add integration tests
status:  auth → completed (all criteria done)
```

Always run `spox -c` first to get the current numbered list before calling `spox check`, since indices shift as criteria are checked off.

## Setting status

`spox status <spec> <value>` rewrites the `status:` line in the spec file.

```
$ spox status auth in-progress
auth: draft → in-progress
```

Common values: `draft`, `ongoing`, `completed`, `discarded`.

## Spec file format

Specs are `.md` files inside `.spox/`. The first line must be `status: <value>`. Unchecked items (`- [ ] ...`) are open criteria; checked ones (`- [x] ...`) are complete and not shown by `spox -c`.

```markdown
status: in-progress

Some notes about this spec.

- [x] Define the data model
- [ ] Write the parser
- [ ] Add tests
```

## Common tasks

**Check current spec status** — run `spox` or `spox -c` and summarise what you see for the user.

**Initialise spox in a new project** — run `spox init`. This creates `.spox/` and writes `.spox/.format.md`, which is the canonical reference for spec format, naming conventions, and valid statuses. Read it before creating specs in an unfamiliar project.

**Create a new spec** — use the template in `spec-template.md` (in this skill directory). Write the filled-in file to `.spox/<name>.md`. Names should be short kebab-case feature identifiers, not task descriptions (`map-grid.md`, not `implement-map.md`).

**Check off a criterion** — run `spox -c` to get the numbered list, then `spox check <spec> <n>`.

**Update spec status** — run `spox status <spec> <value>`.

**Install the skill into a project** — run `spox skill install`; this embeds the skill at `.claude/skills/spox/SKILL.md` relative to the git root.

## Supporting files in this skill

- `sdd.md` — what Spec-Driven Development is and the intended workflow. Read this when starting work in a spox project for the first time or when the user asks how to use spox.
- `spec-template.md` — blank spec template. Use this when creating a new spec file.

## If spox isn't installed

Direct the user to the spox repo and run `zsh install.zsh`. This builds the release binary and symlinks it into `~/.local/bin/spox`.
