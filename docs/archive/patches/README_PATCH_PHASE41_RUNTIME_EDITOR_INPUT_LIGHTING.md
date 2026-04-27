# Starlight Ridge Phase 41 — Runtime Player, Day/Night Lighting, and Editor Input Stabilizer

## Focus

- Fix native egui editor compile regression from the duplicated unfinished `draw_top_bar` block.
- Update the egui editor implementation to the current `eframe::App::update` method shape.
- Wire the existing generated `player_walk.png` sheet into the OpenGL game renderer.
- Add 8-direction WASD/arrow movement with normalized diagonal speed.
- Add a simple 3-real-hour day/night cycle tint pass.
- Render phase 17 static prop sprites from `oceans_heart_bridge_phase17.png` where prop metadata matches sheet entries.
- Restore web editor keyboard shortcuts for brush/erase/eyedrop/grid/save/zoom/reload.

## Files changed

- `crates/app/src/egui_editor.rs`
- `crates/app/src/lib.rs`
- `crates/engine_render_gl/src/lib.rs`
- `tools/web_editor/app.js`

## Expected result

After extracting this patch over the project root:

1. `editor.exe` should no longer fail on the duplicated `draw_top_bar` / wrong eframe method issue.
2. The game renderer should show the player sprite from `assets/textures/player_walk.png` at `player_start`.
3. WASD and arrow-key movement should move the player in 8 directions without diagonal speed boost.
4. The scene should have a visible day/night tint cycle using the current target of 1 in-game day = 3 real-world hours.
5. Props such as `seagull`, `weak_tree_full`, `big_stone_large`, `driftwood_log`, and `coastal_shrub` should render when present in map props.
6. The web editor should respond to:
   - `V` inspect/select
   - `B` brush paint
   - `E` erase
   - `I` eyedropper
   - `G` grid toggle
   - `+` / `-` zoom
   - `R` reload map
   - `Ctrl+S` save when write mode is enabled

## Still not solved in this patch

- Collision blocking is not yet enforced against terrain/props.
- Player depth sorting is still basic: static props render before the player.
- The day/night pass is a full-screen tint, not per-tile/per-light-source lighting.
- The web editor still edits map layer text only; it does not yet share the full native atlas compare/import workflow.

## Recommended next phase

Phase 42 should be `Collision + Interaction Runtime Pass`:

- terrain collision mask from `base_tileset_roles.ron`
- prop collision mask
- interact prompt when facing interactable tiles/props
- simple sign/shipping-bin/well interaction hook
- map boundary cleanup
- editor validation for blocked player spawn
