# Editor Voxel Panels

Voxel panel kits are RON-backed source data for modular voxel-pixel panels.
They are used by the egui editor under **Assets -> Voxel Panels**.

## Kit location

`content/editor_voxel_panels/panel_kits/*.ron`

## Current phase

Phase 53j adds the first read-only 3D visual proof for voxel panel compositions:

- load Phase 53i composition preview RON exports
- export and immediately load the selected composition preview
- draw baked voxel boxes from `VoxelPanelCompositionMeshExportDef`
- draw inclusive bounds
- draw socket gizmos
- draw connection gizmos
- provide orbit/pan/zoom camera controls
- keep the source panel/composition RON as the source of truth
- keep the 3D panel read-only with no live mesh editing yet

## Guardrail

The Voxel Panel Designer must remain inside the single top-level egui editor shell.
Do not call full-shell render functions from the voxel panel workspace, composition canvas, preview panel, validation panel, or any nested asset tab.

## Preview exports

Generated preview files are written to:

`content/editor_voxel_panels/preview_exports/`

The expected workflow is:

1. Open `Assets -> Voxel Panels`.
2. Select a kit and composition.
3. Click `Export + load` in the `3D Preview` panel, or use `Export 3D preview RON` and then `Load RON`.
4. Inspect voxels, bounds, sockets, and connections in the read-only viewport.

## Phase 53k

Phase 53k polishes the read-only composition preview with a projected floor grid, XYZ axis gizmo, hover labels, material legend, selected instance/socket highlights, preview export history, and diagnostics for missing materials, required sockets, invalid connections, and preview stats.
