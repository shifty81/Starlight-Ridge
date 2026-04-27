# Starlight Ridge Phase 31 — Native Editor Clickable Asset Studio

This patch turns the phase 30 native editor shell from a mostly visual scaffold into a clickable first-pass Asset Studio workflow.

## Replaced files

- `crates/app/src/lib.rs`
- `crates/engine_render_gl/src/lib.rs`

## Main changes

- Removes the unused phase 29 static panel drawing helpers from `engine_render_gl` so the dead-code warnings should disappear.
- Adds clickable routing for:
  - top toolbar tools
  - left dock tabs
  - left asset cards
  - right inspector tabs
  - right inspector action buttons
  - bottom dock tabs
  - viewport tile picking
- Adds selected asset state to the native shell render state.
- Adds selected tile role/collision display to the inspector.
- Starts the native Asset Studio workflow:
  - click a rendered viewport tile to inspect its atlas cell
  - click the Tile/Atlas preview cards to cycle named tiles
  - click `Role` to cycle the selected tile role
  - click `Block` to cycle collision state
  - click `Clean` to focus seam diagnostics
  - click `Export` to write `artifacts/native_asset_studio_selection.ron`
- Saves role/collision edits into `content/tiles/base_tileset_roles.ron` with a `.ron.phase31.bak` backup.
- Calls live reload after role/collision metadata saves.

## Controls

- `F1` toggle left dock
- `F2` toggle right dock
- `F3` toggle bottom dock
- `F5` live reload
- `V` Select
- `Space` Pan
- `B` Brush
- `E` Eraser
- `G` Fill
- `I` Eyedropper
- `T` Tile Picker
- `C` Collision Paint
- `A` Asset Studio
- `P` Playtest

## Notes

This phase intentionally does not add full native pixel painting yet. It first makes the editor panels clickable and makes tile metadata editing persistent. Pixel-level atlas editing should come next, after the hitbox and metadata workflow are stable.
