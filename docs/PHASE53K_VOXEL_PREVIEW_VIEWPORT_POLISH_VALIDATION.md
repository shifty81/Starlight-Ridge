# Phase 53k — Voxel Preview Viewport Polish + Validation

Phase 53k polishes the Phase 53j read-only voxel composition preview before moving into live 3D editing.

## Scope

- Keeps the preview inside `Assets -> Voxel Panels`.
- Consumes the existing Phase 53i preview RON export.
- Keeps the viewport read-only.
- Preserves the single-root egui editor shell.
- Does not touch gameplay/runtime systems.

## Added

- Projected floor/grid plane in composition space.
- XYZ axis gizmo overlay.
- Material-aware voxel rendering using baked export RGBA.
- Material legend with voxel counts.
- Hover labels for sockets, connections, and instance centers.
- Selected composition instance highlight in the 3D viewport.
- Selected socket highlight for the currently selected instance/panel socket.
- Preview export history/file picker sourced from the selected composition export directory.
- Preview diagnostics panel.

## Diagnostics covered

- Header count mismatch checks.
- Missing material references in preview voxels.
- Disconnected required socket warnings.
- Connection socket-gizmo existence checks.
- Preview-metadata compatibility checks for connection pairs.
- Snapped connection endpoint mismatch checks.
- Unsnapped connection warnings with offset reporting.

## Test checklist

1. Run `cargo check --workspace`.
2. Launch `RUN_EDITOR_DEBUG.bat`.
3. Open `Assets -> Voxel Panels`.
4. Use `Export + load` in the `3D Preview` panel.
5. Verify the floor grid and XYZ axis appear.
6. Select a composition instance in the 2D composition canvas and verify the 3D selected-instance bounds highlight.
7. Select a socket and verify the selected socket highlight in 3D.
8. Hover sockets/connections/instance centers and verify labels appear.
9. Toggle material legend and preview diagnostics.
10. Use `Refresh export list` and load a selected preview export.
11. Confirm the editor still has exactly one top bar, one side panel stack, and one bottom console/status area.
