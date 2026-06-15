# Spox – Spec + Oxide

I was missing spec driven development from Kiro so I made my own little cli. Probably worse than other SDD tools on GitHub, but suits my needs :)

**Features**
- `spox` cli tool for your bin. build the executable and put it into your bin. ships with a zsh install script.
- `spox` will traverse current repo/folder until it finds `.spox`, then list all specs (md files) in that folder.
  - specs are sorted by status group: active first, then draft/backlog, then completed
  - each spec is prefixed with a status, which will also be listed in the tool output
- `spox -c` / `spox --criteria` lists open tasks for a spec. A task is marked with `- [ ]` in markdown and checked off with `- [x]`

**Commands**
```
spox                        # list all specs with their status
spox -c / --criteria        # also show numbered open criteria under each spec
spox init                   # create .spox/ in the current directory
spox check <spec> <n>       # check off the nth open criterion (auto-completes spec when last one is checked)
spox status <spec> <value>  # set the status field of a spec
spox skill install          # install the Claude Code skill into .claude/skills/
```

**Sample output**
```
> spox
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
> spox -c
map-generation             draft
  ┣━ 1. Generated map has at least three visually distinct terrain regions
  ┣━ 2. No two adjacent runs produce the same map (random seed)
  ┣━ 3. Map generates in under one second on first load
  ┗━ 4. Swapping in a new `TileKind` still requires only one variant + one path mapping
```

**Claude Code integration**

Run `spox skill install` in any project to drop the spox skill into `.claude/skills/`. Claude will then know how to read specs, create new ones, check off criteria, and update statuses — and understand the spec-driven workflow without any explanation from you.