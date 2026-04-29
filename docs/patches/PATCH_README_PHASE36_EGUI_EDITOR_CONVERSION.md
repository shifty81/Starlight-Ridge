# Starlight Ridge Phase 36 — egui Editor Conversion

## Purpose

This patch changes the normal `editor.exe` path from the custom OpenGL overlay shell to an egui/eframe desktop editor shell.

The game executable still uses the existing OpenGL renderer. The editor executable now launches an egui interface with dock-style panels, tool buttons, map preview, tile selection, role/collision metadata editing, and checkpoint export.

## Replaced files

- `crates/app/Cargo.toml`
- `crates/app/src/lib.rs`

## Added files

- `crates/app/src/egui_editor.rs`
- `docs/PATCH_README_PHASE36_EGUI_EDITOR_CONVERSION.md`
- `README_PATCH_PHASE36_EGUI_EDITOR_CONVERSION.txt`

## What changed

- Adds `eframe = "0.34.1"` to the app crate.
- Redirects `app::run_editor()` to the new egui editor path.
- Keeps the previous OpenGL overlay editor behind `run_legacy_gl_editor()` for renderer debugging only.
- Adds a real egui top toolbar, left Project/Textures/Maps panel, right Tile/Seams/Export inspector, bottom Console/Validation/Hot Reload/Runtime panel, and central World Preview.
- Adds selectable map preview cells.
- Adds tile list filtering.
- Adds role/collision combo boxes and persistent tile role metadata saving.
- Adds egui checkpoint export to `artifacts/egui_asset_studio_selection.ron`.
- Keeps the web Asset Lab launch button as a bridge until atlas compare/import is ported natively.

## Expected result

After rebuilding, `editor.exe` should open as an egui application rather than the previous custom GL overlay UI.

The interface should be much more usable for the next workflows:

- larger atlas workflows
- tile metadata editing
- map preview picking
- future side-by-side atlas compare/import
- future animation and character panels

## Important boundary

This is the editor shell conversion and workflow migration. It does not yet implement true pixel editing inside egui. The old web Asset Lab still contains the most advanced pixel tools, grid overlay, and mirror modes. Those should be ported into egui next.

## Next patch recommendation

`Starlight_Ridge_phase37_egui_atlas_compare_import.zip`

Target:

- side-by-side source/destination atlas panels
- import external tile sheet button
- click/drag tile copy
- overwrite mode
- append mode
- mirror-aware paste
- 4-season tile set slots
- validation before saving
- automatic `.bak` backup
- runtime-safe atlas export
