# Phase 53l — Voxel Composition 3D Selection + Transform Handles

## Scope

This patch turns the Phase 53k Voxel Panel Designer 3D Preview from inspection-only into the first interactive composition-selection pass.

The work remains editor-only. It does not touch gameplay, runtime simulation, player controls, map loading, or shipped-game behavior.

## Implemented

- 3D preview hover hit testing for:
  - baked panel instances
  - baked socket gizmos
  - selected-instance XYZ translate handles
- Click selection in the 3D viewport:
  - clicking an instance selects the matching composition instance
  - clicking a socket selects the matching composition instance and socket
  - 3D selection syncs back to the 2D composition state and selected panel/socket state
- Selected instance transform handles:
  - X handle
  - Y handle
  - Z handle
  - colored handle rays and endpoint markers
  - hover-highlighted active handle
- Drag-to-translate for selected composition instances:
  - handle dragging translates the selected instance along one axis
  - drag distance is projected along the handle screen vector
  - movement respects configurable grid snap
  - live preview data is rebuilt in memory after movement
- Undo/redo for composition transforms:
  - transform start pushes a composition snapshot
  - Undo 3D transform restores prior composition data
  - Redo 3D transform reapplies it
- 2D/3D selection sync:
  - selecting in the 2D composition canvas updates the selected source panel
  - selecting in the 3D preview updates the same composition selection state
- Nested-shell safety remains unchanged:
  - no new call path renders the full editor shell inside Voxel Panels
  - all work stays inside the existing Voxel Panel Designer workbench mode

## Files touched

- `crates/app/src/egui_editor.rs`
- `docs/PHASE53L_VOXEL_COMPOSITION_3D_SELECTION_TRANSFORM_HANDLES.md`

## Notes

The 3D transform system is still a scaffold-level editor interaction pass. It uses the existing software-projected preview rather than a true GPU picking/rendering backend. That is intentional for this phase.

The preview is now interactive, but it is still driven by the same composition RON data and Phase 53i mesh-preview bake contract.

## Validation checklist

Run locally:

```text
cargo check --workspace
RUN_EDITOR_DEBUG.bat
```

Manual editor checks:

```text
Open Assets > Voxel Panels
Switch to 3D Preview
Click Export + load
Hover instances and sockets
Click an instance and confirm the 2D composition selection changes
Click a socket and confirm the selected socket changes
Drag X/Y/Z handles and confirm the selected instance moves
Use Undo 3D transform and Redo 3D transform
Switch back to Composition and confirm the same instance is selected
Save the kit and reload
Confirm the editor shell is not nested inside itself
```

## Next recommended phase

Phase 53m should move from transform scaffolding into stronger composition editing:

```text
Starlight_Ridge_phase53m_voxel_composition_3d_connection_editing_snap_preview.zip
```

Suggested focus:

- click/select connections in 3D
- connect sockets from 3D selection
- preview candidate snap targets while dragging
- show valid/invalid snap target colors
- add transform gizmo plane constraints
- add keyboard nudging for selected instance
- add delete/duplicate instance from 3D selection
- keep nested-shell guard intact
- no gameplay changes
