# spox

`spox` lists spec statuses from a `.spox/` directory, walking up the tree from your current working directory (git-style) to find it.

## Usage

```
spox              # list all specs with their status
spox -c           # include open (unchecked) criteria under each spec
spox --criteria   # same as -c
```

## Spec file format

Each spec is a `.md` file inside `.spox/`. The first line sets the status, and unchecked task items are treated as open criteria:

```markdown
status: in-progress

- [x] Define the data model
- [ ] Write the parser
- [ ] Add tests
```

Running `spox -c` in any directory at or below the `.spox/` parent will print:

```
my-spec  in-progress
  ┣━ Write the parser
  ┗━ Add tests
```

## Installation

From the repo root:

```zsh
zsh install.zsh
```

This builds the release binary and symlinks it into `~/.local/bin/spox` (or `$SPOX_BIN` if set).
