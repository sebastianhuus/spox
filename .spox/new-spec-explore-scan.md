status: completed

# new-spec explore scan

## Intent
Before drafting a spec, the new-spec skill triggers a lightweight Explore subagent to scan the repo for relevant context — related specs, adjacent code, naming conventions. Findings are injected as grounding context so the main agent never silently assumes things that are already visible in the codebase, and never asks the user for information the repo already provides.

## Acceptance criteria
- [x] The new-spec skill spawns an Explore subagent when invoked, before any spec is drafted
- [x] The Explore agent searches for: existing specs related to the feature name, adjacent code files that the feature likely touches, and any naming/pattern conventions in the area
- [x] Findings from the Explore agent are passed as context to the main agent writing the spec
- [x] The main agent does not ask the user for information already resolved by the Explore scan
- [x] The scan does not block simple cases — if the feature name is unambiguous and the Explore agent finds nothing relevant, the spec is written directly without extra prompting

## Notes
First phase of a broader interactive spec sparring initiative. Future phases will add a Haiku-based structural gap classifier and targeted user sparring for unresolved integration questions. This phase stands alone and is independently testable: invoke new-spec with a feature that touches existing code and verify the generated spec reflects actual repo conventions rather than plausible-looking assumptions.
