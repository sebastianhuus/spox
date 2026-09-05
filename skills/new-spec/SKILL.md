---
name: new-spec
description: Create a new spox spec. Use when the user wants to add a spec, track a new feature, or start new work in a .spox/ project.
allowed-tools: Bash(spox list -c), Bash(spox view *), Bash(cat *), Bash(git rev-parse *), Bash(mdfind "kMDItemCFBundleIdentifier == 'com.markreview.app'"), Bash(open -a MarkReview *), Read, Write, Glob, Grep, Agent, AskUserQuestion
---

## Current specs

!`spox list -c`

## Project format conventions

!`cat "$(git rev-parse --show-toplevel 2>/dev/null)/.spox/.format.md" 2>/dev/null || true`

## Instructions

Create a new spec file in `.spox/`.

If the user named a feature in their message, derive the spec name from it. Otherwise ask for a feature name first. Names must be short kebab-case identifiers (`map-grid`, `auth-refresh`) — not task descriptions (`implement-map`, `fix-auth-bug`).

### Step 1: Explore the repo

Before drafting, use the Agent tool with `subagent_type=Explore` to scan for relevant context. Give it this prompt (substituting the feature name):

> Search this repo for context relevant to a new spec called "<feature-name>". Find: (1) any existing .spox/ specs with similar names or overlapping topics, (2) source files or modules the feature is likely to touch — grep for the feature name and related terms, (3) naming or structural conventions visible in adjacent code. Return a concise summary: related specs, relevant file paths, conventions to follow. If nothing relevant is found, say so briefly.

### Step 2: Confirm direction with the user — before drafting

This is the main control point, and it happens *before* any spec content is written. Don't draft first and ask for review after — by the time a full Intent/Acceptance/Notes draft exists it's too heavy to negotiate line by line, and wrong assumptions are already baked in.

Turn the Explore findings and the user's request into a short, concrete check-in. The goal is to catch codebase-specific misses while fixing them is a one-line answer, not a rewrite. Cover only what isn't already resolved:

- **Technical approach.** If Explore surfaced a specific integration point, existing pattern, or a small set of candidate modules, name them and ask which is right — e.g. "Explore found `X` already handles `Y` in `path/to/file` — is that where this should hook in, or somewhere else?" This is the question most likely to surface something the agent would otherwise get wrong. If Explore found one clear, unambiguous answer, state it and move on instead of asking.
- **Done condition / acceptance criteria**, if the user's message didn't already give concrete, verifiable ones.
- **Non-goals** — what's explicitly out of scope, so it doesn't get silently assumed either way.
- **Constraints**, if any (must use/avoid a specific approach, library, or pattern).

Use `AskUserQuestion` when Explore turned up a small set of concrete candidates (integration point, module, pattern) — picking from options is cheaper for the user than free text. Use a plain question in your reply for open-ended items (done condition, non-goals). Keep this to one round: ask what's unresolved, get answers, move on — don't loop back for a second pass, and don't ask about anything the user's message or Explore already settled.

**Acceptance criteria must come from the user, not from exploration.** Explore findings can inform *how* a criterion is phrased, but never invent criteria based on what you found in the codebase.

### Step 3: Write the spec

Draft using the confirmed direction from Step 2 — not agent assumptions. Use this template:

```
status: draft
date: [YYYY-MM-DD, today's date]

# [Feature name]

## Intent
[One paragraph: what this is for and why it exists.]

## Acceptance criteria
- [ ] [Something verifiable — visible in the running app, covered by a test, or confirmed by output]
- [ ] [Another criterion]

## Non-goals
[What's explicitly out of scope, if the user named any. Omit section if empty.]

## Notes
[Design decisions, open questions, or implementation detail. Omit section if empty.]
```

Fill in the `date:` line with today's date. Fill in `Intent` and `Notes` from the user's description, the confirmed technical approach, and Explore findings (conventions, relevant files, related specs).

Write the result to `.spox/<name>.md`, then run `spox view <name>` to confirm it was picked up and show the user what was written. Because the substantive decisions (approach, scope, done condition) were already confirmed in Step 2, this isn't a second approval gate — just say it's written and that they can adjust anytime.

## MarkReview

!`mdfind "kMDItemCFBundleIdentifier == 'com.markreview.app'" | grep -q . && echo "MarkReview is installed on this machine. After writing a new spec, offer to open it by putting the command in its own bash-tagged fenced code block: open -a MarkReview .spox/<name>.md, using the path you just wrote. Do not ask in prose and do not run the command yourself — the harness renders a Run button on a bash code block, so the user launches it themselves with one click if they want to." || echo "MarkReview is not installed on this machine. Do not offer to open specs in it. If asked, say it is not installed."`
