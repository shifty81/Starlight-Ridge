# Chat Rollup — Phase 42

## User direction captured

- Add more brushes to the editor using a submenu/drawer that opens when a tool is clicked.
- Keep the actual pixel/tile viewport static while the tool drawer expands.
- Add zoom/pan to the asset editor.
- Character editing needs the same core editor features for clothing, beards, hats, hair, equipment, and later rig/exosuit overlays.
- Character mannequin is too rudimentary and needs a real scale model.
- Lock down character scale so adults are near 2 blocks tall, with children at about 1 block and intermediate children/teens around 1.5 blocks.

## Phase 42 decisions

- Use one shared pixel editor core.
- Expose that core through specialized workspaces instead of creating many unrelated editors.
- Use the current 32x32 tile grid as the world scale unit.
- Adopt scale classes: 1.0 tile child, 1.5 tile teen/older child, 2.0 tile adult, 2.25 tile tall adult/rig.
- Store scale rules in metadata, not hardcoded rendering guesses.
- Render sorting uses bottom-center foot anchors.
- Collision uses separate smaller capsules, not the full sprite height.

## Patch contents

- Fixes `crates/app/src/egui_editor.rs` unclosed delimiter from duplicate `draw_top_bar` header.
- Adds `content/metadata/character_scale_phase42.ron`.
- Adds `content/editor/pixel_editor_workflow_phase42.ron`.
- Adds `docs/editor/PHASE42_CHARACTER_SCALE_AND_PIXEL_EDITOR_WORKFLOW.md`.
- Adds this rollup.
