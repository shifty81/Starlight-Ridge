# Phase 53g — Voxel Panel Designer Polish + Composition Prep

Phase 53g builds on the Phase 53f egui-only Voxel Pixel Panel Designer foundation.
It keeps the single-root egui editor shell intact and does not add gameplay or full 3D rendering.

## Added

- Panel preview thumbnails in the `Assets → Voxel Panels` panel list.
- Editable socket workflow:
  - selectable socket list
  - socket id editing
  - edge selector
  - x/y/z sliders
  - required flag
  - accepts-list editing
  - remove selected socket instead of only removing the last socket
- Depth-layer clipboard tools:
  - copy active layer
  - paste active layer
  - clear active layer
- Transform tools:
  - mirror X
  - mirror Y
  - rotate clockwise
  - rotate counter-clockwise
  - active-layer-only scope toggle
  - full-panel scope support for rectangular rotations that swap dimensions
- Kit-level composition metadata:
  - target view
  - snap unit
  - allowed panel kinds
  - notes
- Panel-level composition metadata:
  - group id
  - anchor
  - snap priority
  - rotation permission
  - mirror X/Y permission
  - tags
- Future 3D viewport prep metadata:
  - voxel unit
  - layer gap
  - default camera label
  - socket gizmo visibility
  - depth separation visibility
- Validation improvements:
  - kit snap unit
  - preview voxel unit
  - allowed panel kind checks
  - panel composition group/anchor checks
  - duplicate cell detection

## Updated data contracts

`game_data::defs` now includes defaulted fields so older Phase 53f RON kits remain loadable:

- `VoxelPanelKitDef.composition`
- `VoxelPanelKitDef.preview_3d`
- `VoxelPanelDef.composition`

These are source-authoring contracts only. They are intended for the later panel composer and 3D voxel viewport but are safe for the current 2D depth-slice editor.

## Updated content

- `content/editor_voxel_panels/panel_kits/starter_gui_panel_kit.ron`
- `content/editor_voxel_panels/panel_kits/starter_building_wall_panel_kit.ron`

Both starter kits now include composition metadata and future preview hints.

## Guardrail

Do not call the full editor shell from inside the Voxel Panel Designer. This phase only extends the existing `Assets → Voxel Panels` workspace branch and preserves the single top-level egui shell.

## Local test checklist

1. Run `cargo check`.
2. Launch `RUN_EDITOR_DEBUG.bat`.
3. Open `Assets → Voxel Panels`.
4. Verify panel thumbnails render in the panel list.
5. Select a panel and edit a socket.
6. Copy a depth layer, clear it, paste it back.
7. Mirror and rotate a square panel layer.
8. Switch transform scope to full panel and rotate a rectangular panel.
9. Save the kit and verify a `.phase53g.*.bak.ron` backup is created.
10. Reopen the editor and confirm the saved kit loads.
