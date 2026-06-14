status: active

Walks up the directory tree to find a `.spox/` directory (git-style) and lists
all spec files with their status line and optional open criteria.

## Usage

```
spox              # list all specs
spox -c           # include open criteria under each spec
spox --criteria   # same as -c
```

## Spec format

First line must be `status: <value>`. Open criteria are lines starting with
`- [ ] `.
