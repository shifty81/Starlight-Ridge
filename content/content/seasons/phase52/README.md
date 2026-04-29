# Phase 52A Content Contract Folder

This folder contains data-first Phase 52A contracts. These files define the shape of future editor/runtime systems and are intentionally safe to add before the full executor/renderer exists.

Rules:

- Keep IDs stable once editor panels start referencing them.
- Prefer adding new records over renaming old IDs.
- Every field that appears in an editor panel should eventually have a matching tooltip/help entry.
- Data here should validate before any generator bakes maps or assets.
