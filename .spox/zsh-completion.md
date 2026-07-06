status: draft

# Zsh completion plugin

## Intent
Provide a `_spox` zsh completion function that completes subcommands, spec names, and status values for the `spox` CLI. Spec names are read dynamically from `.spox/*.md` in the current directory, so completions reflect the actual project. The function is installed via `install.zsh` and requires no manual user setup.

## Acceptance criteria
- [ ] Tab after `spox` completes the set of known subcommands (`list`, `view`, `check`, `status`, `skill`, `init`, `version`, `help`)
- [ ] Tab after `spox view` / `spox check` / `spox status` completes spec names sourced from `.spox/*.md` in the working directory
- [ ] Each spec name candidate is annotated with its current status (e.g. `my-feature  [draft]`) so users can distinguish specs without leaving the completion menu
- [ ] Tab after `spox status <spec>` completes the four valid status values: `draft`, `ongoing`, `completed`, `discarded`
- [ ] Tab after `spox list` and `spox view` completes `-c` / `--criteria`
- [ ] The completion function is bundled in the repo (e.g. `completions/_spox`) and registered during `spox install` / `install.zsh` execution via `fpath`
- [ ] When `.spox/` is absent or empty, spec-name completion returns nothing gracefully (no error)
- [ ] A static check (script or Rust test) verifies that every command name in the `KNOWN_COMMANDS` constant in `main.rs` is present in the completion function; CI fails if they diverge
- [ ] The `known_cmds` local variable in `main.rs:951` is promoted to a module-level `const KNOWN_COMMANDS` so the static check has a single unambiguous source to parse

## Notes
**Convention:** Any PR that adds or renames a subcommand must update `KNOWN_COMMANDS` in `main.rs` and the `completions/_spox` file in the same commit. The static check enforces this at CI time.

**Static check approach:** A shell script (e.g. `ci/check-completions.sh`) extracts the command names from `KNOWN_COMMANDS` and greps for each in `completions/_spox`. This keeps the check dependency-free. Long-term, migrating to `clap` + `clap_complete` would make drift structurally impossible — worth revisiting if spox grows a larger command surface.

**Handling many specs:** zsh's native menu completion handles cycling automatically. Annotating each candidate with `[status]` lets users narrow down by typing a prefix before tabbing — no custom filtering needed.

**Grouping by state vs annotating:** Annotation (`spec-name  [draft]`) is chosen over zsh completion groups because groups require zsh's `compdescr` machinery and add setup complexity. Groups could be added later if demand arises.

**Dynamic sourcing:** The completion function reads spec names and statuses at completion time by parsing the `status:` front-matter line from each `.spox/*.md`. This keeps the plugin self-contained with no dependency on `spox` itself being in PATH during completion.

**`skill install` subcommand:** `spox skill` takes `install` as a fixed argument, not a spec name. Completion should offer `install` after `spox skill`. Subcommands like this are not captured by `KNOWN_COMMANDS` (which only tracks top-level names) — the static check covers top-level commands only.
