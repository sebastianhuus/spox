---
name: spox
description: Use this skill when the user wants to check, list, or discuss project specs tracked in a .spox/ directory. Trigger whenever the user mentions "spox", asks about spec status, wants to see what's in progress, references a .spox directory, asks about open criteria or unchecked tasks in specs, or wants to create or update a spec file. When in doubt, use this skill — it's the right tool any time specs or project status tracking come up.
---

## What spox does

`spox` is an installed CLI tool that lists spec statuses from a `.spox/` directory, walking up the tree from CWD (git-style) to find it. Run it via Bash.

## Commands

```
spox              # list all specs with their status
spox -c           # also show open (unchecked) criteria under each spec
spox --criteria   # same as -c
spox skill install  # copy this skill into the project's .claude/skills/
```

## Reading the output

Each line is `<spec-name>  <status>`. With `-c`, open criteria appear indented below:

```
auth      in-progress
  ┣━ Write token refresh logic
  ┗━ Add integration tests
parser    done
```

## Spec file format

Specs are `.md` files inside `.spox/`. The first line must be `status: <value>`. Unchecked items (`- [ ] ...`) are open criteria; checked ones (`- [x] ...`) are complete and not shown.

```markdown
status: in-progress

Some notes about this spec.

- [x] Define the data model
- [ ] Write the parser
- [ ] Add tests
```

## Common tasks

**Check current spec status** — run `spox` or `spox -c` and summarise what you see for the user.

**Create a new spec** — write a `.md` file into `.spox/` with a `status:` first line and any relevant criteria as `- [ ]` items.

**Update a spec** — edit the relevant `.spox/<name>.md` file directly.

**Install the skill into a project** — run `spox skill install`; this embeds the skill at `.claude/skills/spox/SPOX.md` relative to the git root.

## If spox isn't installed

Direct the user to the spox repo and run `zsh install.zsh`. This builds the release binary and symlinks it into `~/.local/bin/spox`.
