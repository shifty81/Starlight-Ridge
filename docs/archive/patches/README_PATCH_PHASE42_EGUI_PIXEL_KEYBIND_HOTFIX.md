# Starlight Ridge Phase 42 — egui trait + pixel editor keybind hotfix

Drop-in patch for the Phase 41 build.

## Fixes

- Adds the missing `eframe::App::ui` implementation required by the current `eframe` trait in this project.
- Keeps the existing egui `update` path but routes both `update` and `ui` through one shared `draw_app` function.
- Restores Pixel Editor-specific hotkeys in `tools/asset_lab.html`:
  - `B` pencil
  - `E` eraser
  - `G` fill bucket
  - `I` color picker
  - `M` marquee select
  - `Esc` deselect/cancel floating selection
  - `H` horizontal mirror toggle
  - `V` vertical mirror toggle
  - `Shift+G` or `#` pixel grid toggle
  - `Ctrl+C/X/V/Z/Y/D/S` copy/cut/paste/undo/redo/deselect/save atlas

## Notes

The deprecated egui panel warnings are not fixed here because they are warnings only. This hotfix intentionally keeps the patch narrow to clear the cargo check failure and fix the Pixel Editor shortcut path.
