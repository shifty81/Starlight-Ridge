# Phase 29 — Native Asset Studio Panels

This phase turns the native editor shell from blank dock scaffolding into a readable workspace.

## Included now

- Center game viewport remains the main editing/playtest area.
- Left dock now communicates project flow:
  - Project
  - Asset Browser
  - Workflow steps
- Right dock now communicates selected tile workflow:
  - Inspector
  - Selected Tile metadata
  - Tile / Seam / Atlas preview cards
  - Role, collision, seam, export tool chips
- Bottom dock is shorter and split into:
  - Console
  - Validation
  - Hot Reload
  - Runtime Log
- Toolbar remains icon-first, with tool names represented in the editor descriptor for future hover tooltips.
- Added a small built-in bitmap text renderer for the native overlay so the panels are no longer unlabeled placeholder bars.

## Important boundaries

This is still not the finished native pixel editor. Phase 29 gives the native editor a readable panel layout and workflow structure. The next implementation should add hit testing, active tool switching, selected tile binding, and save/edit operations.

## Next recommended phase

`Starlight_Ridge_phase30_native_panel_interaction_and_tile_selection.zip`

Target features:

- Mouse picking in the center viewport.
- Selected tile readout from the actual map/atlas cell.
- Toolbar active tool switching.
- Collapsible docks.
- F5 hot reload button behavior exposed in the bottom panel.
- Real asset browser data from `assets/` and `content/`.
