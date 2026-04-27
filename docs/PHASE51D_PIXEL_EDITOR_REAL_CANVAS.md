# Phase 51d — Pixel Editor Real Canvas

This patch upgrades the native egui `Assets > Pixel Editor` tab from a placeholder note panel into a PNG-backed pixel editor for the active terrain atlas.

## Implemented

- Loads the active tileset PNG from the active map tileset metadata.
- Stores an editable `image::RgbaImage` buffer in editor state.
- Uploads the RGBA buffer to an egui texture with nearest-neighbor sampling.
- Adds a scrollable zoomable pixel canvas.
- Adds transparency checkerboard rendering behind the atlas.
- Adds true black pixel grid overlay at useful zoom levels.
- Adds tile-bound overlay based on tileset columns, rows, tile width, and tile height.
- Adds selected tile crop preview.
- Adds selected tile white outline and red center guides.
- Adds Pencil, Eraser, and Eyedropper tools.
- Adds RGBA sliders and color swatch.
- Adds tile-local Mirror X and Mirror Y painting.
- Tracks dirty state for PNG edits.
- Blocks auto-reload over unsaved pixel edits.
- Saves the edited PNG back to disk.
- Creates timestamped `.bak.png` backups before overwriting the atlas.
- Refreshes the egui atlas texture after save.
- `Ctrl+S` now saves dirty pixel edits when the pixel editor is active/dirty, while preserving map-layer save behavior.

## Files changed

- `crates/app/src/egui_editor.rs`
- `crates/app/Cargo.toml`

## Notes

The world preview still uses the existing debug-colored map preview path. The real atlas texture now appears in the Pixel Editor tab and the selected tile crop preview. A later atlas-preview/import pass should reuse this texture-backed canvas code for the large atlas workbench.

## Next recommended patch

`Starlight_Ridge_phase51e_pixel_tools_undo_clipboard.zip`

Recommended focus:

- Undo / redo stack.
- Rectangular selection.
- Copy tile / copy selection.
- Paste preview.
- Mirror/rotate paste transforms.
- Line tool.
- Fill tool.
- Replace-color fill.
- Brush shapes and dither brush.
