# Spox – Spec + Oxide

I was missing spec driven development from Kiro so I made my own little cli. Probably worse than other SDD tools on GitHub, but suits my needs :)

**Features**
- `spox` cli tool for your bin. build the executable and put it into your bin. ships with a zsh install script.
- `spox` will traverse current repo/folder until it finds `.spox`, then list all specs (md files) in that folder.
  - specs are sorted by status group: active first, then draft/backlog, then completed
  - each spec is prefixed with a status, which will also be listed in the tool output
- `spox list -c` / `spox list --criteria` lists open criteria for each spec. A criterion is marked with `- [ ]` in markdown and checked off with `- [x]`

**Commands**
```
spox list                      # list all specs with their status
spox list -c / --criteria      # also show labelled open criteria under each spec
spox view <spec>               # show a single spec (raw markdown)
spox view -c <spec>            # show a single spec's open criteria dashboard
spox init                      # create .spox/ in the current directory
spox check <spec> <label>      # check off the criterion with the given label (stable across reorders)
spox check <spec> <n>          # check off the nth open criterion (positional fallback)
spox check <spec> all          # check off all remaining open criteria at once
spox status <spec> <value>     # set the status field of a spec
spox skill install             # install the Claude Code skill into .claude/skills/
spox version                   # print the installed version
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
