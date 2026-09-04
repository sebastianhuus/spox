# Spec-Driven Development with spox

SDD is a lightweight discipline: **write the spec before you write the code**. The spec defines what done looks like before implementation begins, so criteria drive the work rather than the other way around.

## The workflow

1. **Write a spec** — create `.spox/<feature>.md` with a status of `draft`. Add acceptance criteria as `- [ ]` items. Criteria should be verifiable: something you can check by running the app, reading output, or running a test. Vague goals ("feels good", "works well") belong in Notes.

2. **Start work** — set status to `ongoing` (`spox status <spec> ongoing`). This signals active implementation.

   Before making code changes — especially anything user-facing (UI, layout, interaction) — prefer entering Claude Code's Plan Mode rather than editing directly. A spox spec covers the *what/why* and a verifiable done-condition; it deliberately doesn't try to pin down visual or interaction detail, because that's not something exploration alone can get right. Plan mode is the complementary step: it explores the actual code being changed (not a summary), asks clarifying questions grounded in what it just read, and requires explicit approval before touching any file. That combination — real code context plus a hard approval gate — is what makes it worth reaching for before UI work in particular, where "what it should look and feel like" is exactly the kind of preference a spec's acceptance criteria won't capture.

3. **Check off criteria as they're met** — `spox check <spec> <label>` using the label shown by `spox -c`. When the last criterion is checked, status automatically becomes `completed`.

4. **Discard if the approach is abandoned** — `spox status <spec> discarded`. Keep the file; it's useful context.

## When to create a spec

- Any feature with more than one moving part
- Anything the user mentions planning before coding
- When asked to "spec out" or "write a spec for" something

Don't create specs for one-line fixes, trivial renames, or work the user is clearly doing ad-hoc.

## What the agent should do on first contact with a spox project

1. Run `spox` to see what's in progress.
2. If `.spox/.format.md` exists, read it — it's the project-local format reference.
3. Before implementing anything, check if there's an existing spec for it. If yes, read it; if no, ask or propose writing one.
