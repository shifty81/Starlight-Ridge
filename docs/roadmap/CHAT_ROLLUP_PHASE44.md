# Chat Rollup — Phase 44

Current checkpoint before Phase 44:

- `cargo check` passed after the Phase 42 fix.
- Phase 43 added the shared pixel editor brush drawer scaffold.
- The next requested step was Character Studio mannequin + paperdoll overlay scaffolding.

Phase 44 implementation direction:

- one shared pixel editor core
- Character Studio as a specialized workspace using that core
- block-relative character scale classes
- bottom-center foot anchors
- paperdoll overlays for clothing, hair, hats, beards, tools, and later rig parts
- art height kept separate from collision height

Implemented files:

```text
crates/app/src/egui_editor.rs
content/editor_character/character_studio_phase44.ron
content/editor/character_studio_workflow_phase44.ron
docs/editor/PHASE44_CHARACTER_STUDIO_PAPERDOLL.md
docs/roadmap/CHAT_ROLLUP_PHASE44.md
docs/patches/README_PATCH_PHASE44_CHARACTER_STUDIO_PAPERDOLL.md
```

Recommended next:

```text
Phase 45 — Character Studio real assets + offsets
```
