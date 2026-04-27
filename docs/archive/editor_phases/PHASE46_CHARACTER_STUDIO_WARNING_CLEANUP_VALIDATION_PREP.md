# Phase 46 — Character Studio warning cleanup + validation prep

## Purpose

Phase 46A preserves the successful `cargo check` checkpoint and removes the remaining `unused_must_use` warning from the Character Studio animation preview scaffold.

## Code fix

The animation preview clip list now checks `.clicked()` on each `ui.selectable_label(...)` response instead of dropping the return value.

## Validation scaffold

Added:

- `content/editor_character/character_validation_phase46.ron`
- `content/editor/character_studio_workflow_phase46.ron`

These define the next live validation targets for Character Studio:

- mannequin scale class validation
- overlay/base frame parity
- paperdoll offset checks
- per-frame foot anchors
- tool sockets
- export readiness warnings

## Next full implementation

The next larger phase should load the Phase 45/46 RON contracts into editor state and make the Scale Validator tab show real pass/fail results from the PNGs and metadata.
