# Starlight Ridge Phase 51e — Pixel Tools Undo + Clipboard

## Focus

Adds the next pixel-editor implementation layer for the egui editor path.

Implemented in `crates/app/src/egui_editor.rs`:

- Undo / redo stacks for PNG pixel edits.
- Rectangular selection state.
- Copy selected atlas tile.
- Copy selected pixel region.
- Internal clipboard buffer.
- Paste tool with hover preview.
- Paste rotation in 90-degree steps.
- Paste flip X / flip Y.
- Pencil and eraser tools.
- Eyedropper tool.
- Flood fill.
- Replace-color fill.
- Line tool.
- Brush size control.
- Square, circle, diamond, and dither brush shapes.
- Mirror X / Mirror Y painting.
- Transparency checkerboard.
- Thin black pixel grid when zoomed in enough.
- Selected atlas tile overlay.
- Save edited PNG with automatic `.phase51e.<timestamp>.bak.png` backup.

## Cargo change

`crates/app/Cargo.toml` now depends on the workspace `image` crate so the egui editor can load, edit, and save PNG atlas data directly.

## Build-log recovery notes

The uploaded cargo check log failed on two editor compile blockers:

1. `eframe::App` missing `ui`.
2. A `#[derive(Debug)]` type containing `egui::TextureHandle`, which does not implement `Debug`.

This patch keeps `TextureHandle` inside a non-`Debug` pixel editor state and adds a compatibility `ui` method to the `eframe::App` implementation.

## Expected result

After extracting over the project root and running option `2) cargo check`, the editor should compile past the Phase 51d/51e pixel editor blockers. In the editor, open:

`Assets -> Pixel Editor`

Use:

- `Ctrl+Z` / `Ctrl+Y` for undo and redo.
- `Rect Select` then `Copy selection` for region copy.
- `Copy tile` to copy the currently selected atlas tile.
- `Paste` for hover-preview placement.
- `Rotate paste 90°`, `Flip paste X`, and `Flip paste Y` before placing.
- `Save PNG` or `Ctrl+S` to write the PNG with backup.
