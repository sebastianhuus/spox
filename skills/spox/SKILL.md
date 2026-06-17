---
name: spox
description: Use this skill when the user wants to check, list, or update project specs tracked in a .spox/ directory. Trigger whenever the user mentions "spox", asks about spec status, wants to see what's in progress, references a .spox directory, asks about open criteria or unchecked tasks in specs, or wants to create or update a spec file. When in doubt, use this skill — it's the right tool any time specs or project status tracking come up.
allowed-tools: Bash(spox) Bash(spox -c) Bash(spox --criteria) Bash(spox -c *) Bash(spox --criteria *) Bash(spox check *) Bash(spox check * all)
hooks:
  PreToolUse:
    - matcher: Bash
      hooks:
        - type: command
          command: "${CLAUDE_SKILL_DIR}/check-chain.sh"
          timeout: 5
---

## What spox does

`spox` is an installed CLI tool that lists and updates spec statuses from a `.spox/` directory, walking up the tree from CWD (git-style) to find it. Run it via Bash.

## Commands

```
spox                        # list all specs with their status
spox -c                     # also show numbered open criteria under each spec
spox --criteria             # same as -c
spox <spec>                 # show a single spec's status
spox -c <spec>              # show a single spec's status and open criteria
spox init                   # create .spox/ in the current directory (new projects)
spox check <spec> <n>       # check off the nth open criterion (1-indexed, matches spox -c output)
spox check <spec> all       # check off all remaining open criteria at once
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

`spox check <spec> all` marks every remaining open criterion as done in one atomic write, and sets status to `completed`. Use this after finishing all the work on a spec.

```
$ spox check auth all
checked: auth — all 2 open criteria done
status:  auth → completed
```

`spox check <spec> <n>` marks a single criterion (useful mid-session for partial completions). If it was the last open criterion, status is set to `completed` automatically.

```
$ spox check auth 1
checked: auth #1 — Write token refresh logic
```

Both forms require reading the spec first — run `spox -c <spec>` before checking, since the mtime guard will reject stale checks. After `check all`, the cache is invalidated, so re-read before any further checks.

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

**Check current spec status** — run `spox` or `spox -c` for the full dashboard, or `spox -c <spec>` to focus on one spec.

**Initialise spox in a new project** — run `spox init`. This creates `.spox/` and writes `.spox/.format.md`, which is the canonical reference for spec format, naming conventions, and valid statuses. Read it before creating specs in an unfamiliar project.

**Create a new spec** — use the template in `spec-template.md` (in this skill directory). Write the filled-in file to `.spox/<name>.md`. Names should be short kebab-case feature identifiers, not task descriptions (`map-grid.md`, not `implement-map.md`).

**Check off all criteria** — run `spox -c <spec>` to read the spec (required by the mtime guard), then `spox check <spec> all`. Use `spox check <spec> <n>` for partial mid-session completions.

**Update spec status** — run `spox status <spec> <value>`.

**Install the skill into a project** — run `spox skill install`; this embeds the skill at `.claude/skills/spox/SKILL.md` relative to the git root.

## Supporting files in this skill

- `sdd.md` — what Spec-Driven Development is and the intended workflow. Read this when starting work in a spox project for the first time or when the user asks how to use spox.
- `spec-template.md` — blank spec template. Use this when creating a new spec file.

## If spox isn't installed

Direct the user to the spox repo and run `zsh install.zsh`. This builds the release binary and symlinks it into `~/.local/bin/spox`.
