---
name: spox
description: Use this skill when the user wants to check, list, or update project specs tracked in a .spox/ directory. Trigger whenever the user mentions "spox", asks about spec status, wants to see what's in progress, references a .spox directory, asks about open criteria or unchecked tasks in specs, or wants to create or update a spec file. When in doubt, use this skill — it's the right tool any time specs or project status tracking come up.
allowed-tools: Bash(spox list) Bash(spox list -c) Bash(spox list --criteria) Bash(spox list -t) Bash(spox list --tagline) Bash(spox view *) Bash(spox view -c *) Bash(spox view --criteria *) Bash(spox check *) Bash(spox check * all) Bash(spox version) Bash(spox path) Bash(spox format) Bash(spox attention) Bash(spox -C * list) Bash(spox -C * list -c) Bash(spox -C * list --criteria) Bash(spox -C * view *) Bash(spox -C * check *) Bash(spox -C * path) Bash(spox -C * format) Bash(spox -C * attention) Bash(cat *)
hooks:
  PreToolUse:
    - matcher: Bash
      hooks:
        - type: command
          command: "${CLAUDE_SKILL_DIR}/check-chain.sh"
          timeout: 5
---

## What spox does

`spox` is an installed CLI tool that lists and updates spec statuses from a `.spox/` directory. It walks up the tree from CWD (git-style) to find it; if there is none above, it looks below the git root for a nested one (up to 5 levels, skipping hidden dirs, `node_modules` and `target`) and uses it when exactly one is found. If several are found it exits with an error naming them. Pick one with `-C` (see below). Run it via Bash.

## Commands

```
spox list                      # list all specs with their status
spox list -c                   # also show labelled open criteria under each spec
spox list --criteria           # same as -c
spox list -t                   # show each spec's one-line tagline (if set)
spox list --tagline            # same as -t
spox view <spec>               # show the full raw spec file
spox view -c <spec>            # show a single spec's open criteria in dashboard format
spox path                      # print the resolved .spox directory (absolute)
spox format                    # print the project's .format.md (spec format conventions)
spox attention                 # report stale ongoing/pending-verification specs (silent when none)
spox attention --scope nearest # only the project containing CWD (default scope: all, incl. sub-projects)
spox attention --hook          # same report, at most once per 24h per project (for session hooks, not for agents)
spox -C <dir> <command> ...    # run any command as if started in <dir>: the project dir, a dir inside it, or its .spox
spox init                      # create .spox/ in the current directory (new projects)
spox check <spec> <label>      # check off the criterion with the given label (stable, preferred)
spox check <spec> <n>          # check off the nth open criterion (positional — avoid in scripts)
spox check <spec> all          # check off all remaining open criteria at once
spox status <spec> <value>     # set the status field of a spec
spox skill install             # copy this skill into the project's .claude/skills/
spox version                   # print the installed version
spox help                      # show usage
```

## Attention report

`spox attention` is a read-only maintenance report, normally called by a session hook rather than by hand. It prints nothing (exit 0) when there is nothing to flag or no `.spox/` at or above CWD. Otherwise the output looks like this:

```
spox attention: 4 specs need a look
services/api
  ┗━ auth  ongoing  30d  2 open
services/web
  ┣━ ui  pending-verification  15d  1 open
  ┗━ draft-new  ongoing  uncommitted  2 open
mono
  ┗━ orchestration  ongoing  5d  1 open
Agent: mention this briefly at a natural moment, offer to go through it, and do not change any spec without asking.
```

The report is a tree grouped by project. Each spec line is `<name>  <status>  <age>d  <open> open` (age = days since the spec's last git commit; oldest first, at most 5). Specs that are `ongoing`/`pending-verification` but untracked or with uncommitted changes show `uncommitted` instead of an age (at most 5) — specs should be committed once finalized. Several projects mean several groups. `spox attention --hook` prints the same report but at most once per 24 hours per project; it exists for session hooks, so run plain `spox attention` yourself, never `--hook`. When you see this report (in a hook's output or by running the command), follow its last line: mention it briefly at a natural moment, offer to go through it, and never change a spec without asking.

## Which project? (nested `.spox/`)

- `.spox/` at or above CWD: used as-is, nothing else to do. This is the common case.
- Otherwise, one nested `.spox/` below the git root (e.g. `notes/app/.spox`): used automatically; spox notes `using <path>` on stderr.
- Several nested ones: every command fails with `multiple spox projects found:` and the project dirs. Ask the user which one, then put `-C <project-dir>` first on every call: `spox -C app list -c`. `SPOX_PROJECT=<dir>` in the environment does the same; `-C` wins over it.
- Outside a git repo there is no downward search; use `-C`.
- Always write new specs into the directory `spox path` prints, never an assumed `.spox/` at the repo root. Keep each spox call a single command (no `cd dir && spox ...`; `-C` replaces that).

## Reading the output

Each line is `<spec-name>  <status>`. With `list -c`, open criteria appear labelled below:

```
auth      in-progress
  ┣━ [a3f2] Write token refresh logic
  ┗━ [b8c1] Add integration tests
parser    done
```

Each `[label]` is a stable 4-char hex identifier derived from the criterion text. Labels survive reordering and partial checks — **always use labels with `spox check`, never position numbers**.

## Checking off criteria

`spox check <spec> all` marks every remaining open criterion as done in one atomic write, and sets status to `completed`. Use this after finishing all the work on a spec.

```
$ spox check auth all
checked: auth — all 2 open criteria done
status:  auth → completed
```

`spox check <spec> <label>` marks a single criterion by its stable label. The label comes from `spox view -c` output and stays the same even after other criteria are checked off or the list is reordered.

```
$ spox check auth a3f2
checked: auth [a3f2] — Write token refresh logic
```

All check forms require reading the spec first — run `spox view -c <spec>` before checking, since the mtime guard will reject stale checks. After each check, the cache is invalidated, so re-read before the next one.

**Never check multiple criteria in one chained command.** Each check must be a separate tool call:
```
# right
spox check auth a3f2
spox view -c auth
spox check auth b8c1

# wrong — second label resolves from stale cache
spox check auth a3f2 && spox check auth b8c1
```

## Setting status

`spox status <spec> <value>` rewrites the `status:` line in the spec file.

```
$ spox status auth ongoing
auth: draft → ongoing
```

Valid values: `draft`, `ongoing`, `pending-verification`, `completed`, `discarded`. Any other value is rejected with a non-zero exit.

## Spec file format

Specs are `.md` files inside `.spox/`. The first line must be `status: <value>`, followed by `date: <YYYY-MM-DD>` (the spec's creation date, set once and never edited — used to sort specs newest-first within a status group). Unchecked items (`- [ ] ...`) are open criteria; checked ones (`- [x] ...`) are complete and not shown by `spox view -c`.

```markdown
status: in-progress
date: 2026-03-14

Some notes about this spec.

- [x] Define the data model
- [ ] Write the parser
- [ ] Add tests
```

Real specs follow the structure in `spec-template.md`. That includes an optional `## Open questions` section: one bullet per unresolved, non-blocking question, with `Recommend: ...` when you have a suggestion. Ask blocking questions at once instead; move answered ones to Notes as `Decided: ...`; never write a question as `- [ ]` (spox counts those as criteria). See `sdd.md`.

## Checking for MarkReview annotations

Specs can be reviewed and annotated in the MarkReview app, which saves annotations to a
`<spec>.markreview` JSON sidecar next to `.spox/<spec>.md`. Whenever you read or discuss a spec —
after `spox view <spec>`, `spox view -c <spec>`, or similar — check whether that sidecar exists
and, if so, read it and surface its annotations before acting on the spec.

**Run this as its own Bash call, never chained onto the `spox view` command.** Chaining
`spox view <spec> && cat .spox/<spec>.markreview` is blocked by this skill's PreToolUse hook,
which denies any chained command where `spox` is the first token of a segment — the same reason
`spox check` calls must never be chained (see above). Run `spox view <spec>` first, read its
output, then run separately:

```
cat .spox/<spec>.markreview
```

If the file doesn't exist, `cat` will error ("No such file or directory") — that's expected and
means there's nothing to surface, not a problem to fix. Move on silently.

If the file exists, parse the JSON:
- Follow the top-level `agentInstructions` string field first — it's the file's own instruction
  for how to treat its annotations and takes precedence over the generic handling below.
- Ignore any annotation with `"status": "muted"` entirely — don't mention it, don't act on it.
- For each remaining (`"status": "open"`) annotation, surface its `comment`, `selectedText`, and
  `section` to the user before proceeding, so they know what feedback exists and where it applies.

## Specs are repo-scoped, not global

A spec name only resolves within the `.spox/` directory of the repo it was created in — `spox view`/`spox list` find *that repo's* `.spox/` (walking up from CWD, or one nested below the git root) and nowhere else. If you're handed a spec name or its contents that references a **different** repository (e.g. relayed from another agent's exploration, or from the user pasting in cross-repo context), do **not** try to `spox view` or search for it in your own workspace — it won't be there, and that's expected, not an error. Treat the content you were given as authoritative data and use it directly instead of re-deriving it. Only fall back to a local lookup if it's genuinely ambiguous which repo the reference belongs to.

## Common tasks

**Check current spec status** — run `spox list` or `spox list -c` for the full dashboard, or `spox view -c <spec>` to focus on one spec.

**Initialise spox in a new project** — run `spox init`. This creates `.spox/` and writes `.spox/.format.md`, which is the canonical reference for spec format, naming conventions, and valid statuses. Read it before creating specs in an unfamiliar project.

**Create a new spec** — use the template in `spec-template.md` (in this skill directory), filling in the `date:` line with today's date. Write the filled-in file to `.spox/<name>.md`. Names should be short kebab-case feature identifiers, not task descriptions (`map-grid.md`, not `implement-map.md`).

**Check off all criteria** — run `spox view -c <spec>` to read the spec (required by the mtime guard), then `spox check <spec> all`. Use `spox check <spec> <label>` for partial mid-session completions — the label comes from `spox view -c` output and is stable across checks.

**Update spec status** — run `spox status <spec> <value>`.

**Install the skill into a project** — run `spox skill install`; this embeds the skill at `.claude/skills/spox/SKILL.md` relative to the git root.

## Supporting files in this skill

- `sdd.md` — what Spec-Driven Development is and the intended workflow. Read this when starting work in a spox project for the first time or when the user asks how to use spox.
- `spec-template.md` — blank spec template. Use this when creating a new spec file.

## If spox isn't installed

Direct the user to the spox repo and run `zsh install.zsh`. This builds the release binary and symlinks it into `~/.local/bin/spox`.
