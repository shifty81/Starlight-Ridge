# Phase 53f — Voxel Pixel Panel Designer Foundation

## Goal

Add the first real editor foundation for modular voxel-pixel panels without touching gameplay or adding full 3D rendering.

## Added

- `Assets → Voxel Panels` egui subtab.
- RON-backed voxel panel kit contracts.
- Kit loading from `content/editor_voxel_panels/panel_kits/*.ron`.
- Save-with-backup for active panel kits.
- 2D voxel-pixel depth-slice canvas.
- Paint, erase, and pick tools.
- Depth/layer selector.
- Palette/material selector with swatches.
- Panel metadata editing.
- Basic panel add/duplicate actions.
- Socket listing plus add/remove center socket helpers.
- Validation for:
  - empty kit/panel/material IDs
  - missing default palette
  - duplicate panel IDs
  - zero or oversized dimensions
  - out-of-bounds cells
  - missing material references
  - duplicate/out-of-bounds sockets
  - unsupported socket edges

## Added example content

- `content/editor_voxel_panels/panel_kits/starter_gui_panel_kit.ron`
- `content/editor_voxel_panels/panel_kits/starter_building_wall_panel_kit.ron`

## Guardrails preserved

- The patch does not call the full editor shell from inside any tab/panel.
- The patch uses only a nested workspace subtab under the existing single-root egui shell.
- The patch does not touch gameplay.
- The patch does not add full 3D rendering yet.
- The patch keeps VOX model scanning separate from voxel-pixel panel authoring.

## Manual test checklist

Run locally because this container does not include Rust/Cargo:

```bat
cargo check
RUN_EDITOR_DEBUG.bat
```

Then verify:

1. Editor opens with one top bar, one left panel, one right panel, and one bottom panel.
2. No nested copy of the editor appears inside the editor.
3. Open `Assets → Voxel Panels`.
4. Switch between the GUI and building wall panel kits.
5. Select materials and paint on the grid.
6. Change depth layer and paint another slice.
7. Use erase and pick.
8. Add a blank panel.
9. Add/remove a center socket.
10. Save the kit and confirm a `.phase53f.*.bak.ron` backup appears.
11. Reopen/reload and confirm saved cells persist.
