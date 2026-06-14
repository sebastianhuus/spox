# Spox – Spec + Oxide

I was missing spec driven development from Kiro so I made my own little cli. Probably worse than other SDD tools on GitHub, but suits my needs :)

**Features**
- `spox` cli tool for your bin. build the executable and put it into your bin. ships with a zsh install script.
- `spox` will traverse current repo/folder until it finds `.spox`, then list all specs (md files) in that folder. 
  - each spec is prefixed with a status, which will also be listed in the tool output
- `spox -c` / `spox --criteria` lists open tasks for a spec. A task is marked with `- [ ]` in markdown and checked off with `- [x]`

**Sample output**
```
> spox
camera-distance-lock       draft
camera-fog-of-war          draft
camera-navigation          implemented
character-creation         draft
font-loading               draft
format                     reference
game-scaffold              completed
map-generation             draft
map-layers                 in progress
```

```
> spox -c
map-generation             draft
  ┣━ Generated map has at least three visually distinct terrain regions
  ┣━ No two adjacent runs produce the same map (random seed)
  ┣━ Map generates in under one second on first load
  ┗━ Swapping in a new `TileKind` still requires only one variant + one path mapping
```