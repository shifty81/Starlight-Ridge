# Phase 53q — Voxel Composition 3D Connection Graph Inspector

## Goal

Add an inspector layer to the Voxel Panels 3D Preview so modular panel socket wiring can be audited and repaired without leaving the 3D composition workflow.

## Implemented

- Added a **Connection graph inspector** inside the 3D Preview mode.
- Added connection graph filters:
  - All connections
  - Invalid
  - Selected
  - Required sockets
  - Orphan sockets
- Added graph summary counts:
  - connection count
  - invalid connection count
  - open required socket count
  - orphan socket count
- Added connection rows with:
  - connection id
  - source socket
  - target socket
  - valid/invalid state
  - compatibility/snapped/offset status
  - repair suggestion text
- Added socket rows with:
  - instance id
  - socket id
  - panel id
  - world position
  - required/open/connected state
  - compatible target count
- Added inspector actions:
  - Select first invalid connection
  - Select first open required socket
  - Select individual graph connection rows
  - Select individual graph socket rows
  - Auto-connect nearest compatible socket from the currently selected source socket
- Selection jumps sync back into the existing 3D preview selection/socket tools.
- Existing 53p.1 group-select, 53p group operations, 53o undo routing, 53n rotate/mirror, and 53m connection editing paths are preserved.
- Single-root egui shell guard remains untouched.
- No gameplay changes.

## Manual validation

1. Open `Assets > Voxel Panels > 3D Preview`.
2. Click `Export + load`.
3. Expand `Connection graph inspector`.
4. Verify connection counts and socket counts are shown.
5. Change graph filters.
6. Select an invalid connection row and confirm the related source/target instances highlight in 3D.
7. Select an open required socket and confirm the selected socket/instance syncs into the 3D preview.
8. Select a socket, press `Auto-connect nearest compatible`, and confirm a compatible nearby connection is created or a clear failure reason is shown.
9. Use existing delete/undo/redo paths to confirm graph edits remain reversible.
10. Confirm the editor still renders a single top-level shell.

## Local validation

```text
cargo check --workspace
RUN_EDITOR_DEBUG.bat
```

## Notes

This phase intentionally does not add a new data contract. It derives the graph report from the existing composition instances, panel sockets, and composition connections so the inspector remains a safe editor-only overlay.
