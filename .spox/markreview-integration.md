status: completed
date: 2026-09-04

# Markreview integration

## Intent
Integrate the MarkReview app into the spox skill workflow. Right now, opening a spec in
MarkReview means finding its path by hand and running `open -a MarkReview` yourself; and once
you've reviewed a spec and saved annotations there, the agent has no way to notice and act on
that feedback. This makes launching a review effortless and makes acting on saved annotations
automatic.

## Acceptance criteria
- [x] Whenever a new spec is created, the skill offers to open it in MarkReview by running
      `open -a MarkReview <path-to-spec>` on the user's behalf, so they don't need to locate the
      file themselves.
- [x] Before offering, the skill checks whether MarkReview is installed via
      `mdfind "kMDItemCFBundleIdentifier == 'com.markreview.app'"` — not `open -a`, which would
      launch it as a side effect just to check, and not scanning `/Applications`, which misses
      other install locations. If it's not installed, the skill doesn't offer, and says so if asked.
- [x] Whenever the skill subsequently reads or discusses a spec (e.g. `spox view <spec>`) and a
      sibling `<spec>.markreview` file exists next to it, the skill reads the JSON and surfaces
      the `open` annotations (comment, selected text, section) to the agent, following the file's
      own `agentInstructions` field.
- [x] Annotations with `status: muted` are ignored — not surfaced, not acted on.

## Non-goals
- A native `spox review <spec>` command that parses and prints `.markreview` files in spox's own
  dashboard format — useful later, but out of scope here; this spec is skill-level only (cat the
  JSON, no Rust changes).
- Auto-launching MarkReview without the offer being accepted, or acting on annotations without
  telling the user what was found first.

## Notes
- `.markreview` sidecar files are only created by the MarkReview app itself, after a spec has
  been opened, reviewed, and saved there — never by spox or the agent. There's no sidecar at
  spec-creation time; the offer to open MarkReview is unconditional on that.
- MarkReview annotations are the user's main channel for giving the model feedback on a spec, so
  surfacing them well matters more than volume — comment text + selected text + section is
  probably enough context to act on each one.
- Bundle id confirmed on this machine: `com.markreview.app`, app name `MarkReview.app`.
