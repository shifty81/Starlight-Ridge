# Phase 51b — egui nested editor fix

This patch fixes the editor UI being drawn inside itself after the Phase 51 compatibility patch.

## Cause

eframe 0.34 uses `App::ui(&mut Ui, &mut Frame)` as the primary app entry point. The prior compatibility patch kept the old `update(&Context, &mut Frame)` path and also called the full editor panel tree from `ui()` through `ui.ctx()`. In this eframe path, that caused the full editor frame to be drawn twice, making the editor appear nested inside itself.

## Fix

- Moves the editor draw path to `App::ui` only.
- Removes the old `update` implementation.
- Changes top/left/right/bottom/status/central panel calls to `show_inside(root_ui, ...)`.
- Updates deprecated panel sizing calls:
  - `default_width` -> `default_size`
  - `width_range` -> `size_range`
  - `exact_height` -> `exact_size`
- Wires the previously-unused `open_web_asset_lab` helper to a button in the Textures panel.
- Adds a visible validation warning count for the loaded world manifest.

## Validation

Run:

```bash
cargo check --workspace
```

Then relaunch the editor. The editor should render once, with no nested duplicate top bars, side panels, inspector, or bottom console.
