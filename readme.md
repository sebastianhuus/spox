# Spox – Spec + Oxide

I was missing spec driven development from Kiro so I made my own little cli. Probably worse than other SDD tools on GitHub, but suits my needs :)

**Features**
- `spox` cli tool for your bin. build the executable and put it into your bin. ships with a zsh install script.
- `spox` will traverse current repo/folder until it finds `.spox`, then list all specs (md files) in that folder.
  - specs are sorted by status group: active first, then draft/backlog, then completed
  - each spec is prefixed with a status, which will also be listed in the tool output
- if there is no `.spox` above the current directory, `spox` looks below the git root (5 levels, skipping hidden dirs, `node_modules`, `target`) and uses it when exactly one is found. With several it lists them and exits; pick one with `spox -C <dir> <command>` or `SPOX_PROJECT=<dir>`. Outside a git repo there is no downward search.
- `spox list -c` / `spox list --criteria` lists open criteria for each spec. A criterion is marked with `- [ ]` in markdown and checked off with `- [x]`

**Commands**
```
spox list                      # list all specs with their status
spox list -c / --criteria      # also show labelled open criteria under each spec
spox view <spec>               # show a single spec (raw markdown)
spox view -c <spec>            # show a single spec's open criteria dashboard
spox attention                 # report stale ongoing/pending-verification specs; prints nothing if none
spox attention --scope nearest # only the project containing the current directory (default: all; also SPOX_ATTENTION_SCOPE)
spox path                      # print the resolved .spox directory
spox format                    # print the project's .format.md
spox -C <dir> <command>        # act as if started in <dir> (project, a dir inside it, or its .spox); also SPOX_PROJECT=<dir>
spox init                      # create .spox/ in the current directory
spox check <spec> <label>      # check off the criterion with the given label (stable across reorders)
spox check <spec> <n>          # check off the nth open criterion (positional fallback)
spox check <spec> all          # check off all remaining open criteria at once
spox status <spec> <value>     # set the status field of a spec
spox skill install             # install the Claude Code skill into .claude/skills/
spox version                   # print the installed version
```

**Attention report**

`spox attention` is a read-only maintenance report meant to be called from a session hook. It only runs where spox is set up (a `.spox` at or above the current directory — it never searches below the git root for one) and looks at specs that are `ongoing` or `pending-verification`. With nothing to report it prints nothing and exits 0. Otherwise it prints a count line, up to 5 committed specs oldest first (age = days since the file's last git commit, or since its modified time outside a git repo), an `uncommitted:` line for specs that are untracked or have uncommitted changes, and a final line instructing the agent. By default it also scans every `.spox` project below the current directory (a monorepo top folder that has its own `.spox`), prefixing names with the project path; `--scope nearest` / `SPOX_ATTENTION_SCOPE=nearest` limits it to the containing project. It never runs the background self-update or writes any file.

```
> spox attention
spox attention: 3 specs need a look
services/api:auth  ongoing  30d  2 open
.:orchestration  ongoing  5d  1 open
uncommitted: services/web:ui
Agent: mention this briefly at a natural moment, offer to go through it, and do not change any spec without asking.
```

**Sample output**
```
> spox list
map-layers                 in progress
camera-distance-lock       draft
camera-fog-of-war          draft
character-creation         draft
font-loading               draft
map-generation             draft
camera-navigation          implemented
game-scaffold              completed
```

```
> spox list -c
map-generation             draft
  ┣━ [a1b2] Generated map has at least three visually distinct terrain regions
  ┣━ [c3d4] No two adjacent runs produce the same map (random seed)
  ┣━ [e5f6] Map generates in under one second on first load
  ┗━ [a7b8] Swapping in a new `TileKind` still requires only one variant + one path mapping
```

**Claude Code integration**

Run `spox skill install` in any project to drop the spox skill into `.claude/skills/`. Claude will then know how to read specs, create new ones, check off criteria, and update statuses — and understand the spec-driven workflow without any explanation from you.

**Glow integration**

If [glow](https://github.com/charmbracelet/glow) is installed (`brew install glow`), `spox view` automatically pipes output through it for styled markdown rendering. Falls back to plain text when piped to another command, or set `SPOX_NO_GLOW=1` to disable it.
