# Phase 53n — Voxel Composition 3D Rotation / Mirror / Constraints

Patch type: overlay patch on top of Phase 53m.1.

## Scope

Editor-only update for the Voxel Panel Designer 3D Preview mode.

No gameplay files were intentionally changed.

## Added

- Selected-instance rotation controls in the 3D Preview toolbar.
- Clickable 3D rotation handle on the selected instance.
- Keyboard `R` shortcut while the preview is hovered to rotate selected instance clockwise.
- Selected-instance Mirror X / Mirror Y controls in 3D Preview mode.
- Locked instances block 3D rotate/mirror edits.
- Selected-instance rotation is clamped to normalized 0/90/180/270 degree values.
- Selected instance is clamped back into composition bounds after rotation.
- 3D connection hit testing.
- Click/select existing connection lines in the 3D preview.
- Selected connection highlighting.
- Hovered connection highlighting.
- Delete selected connection button.
- Delete key deletes a hovered connection before falling back to selected instance deletion.
- Snap-ghost label for socket connection drafts, including offset and snapped status.
- Overlap/collision diagnostics for baked instance bounds.
- Preview diagnostics success text now includes overlap checks.

## Preserved

- Single-root egui shell guard.
- 53l instance/socket selection and translate handles.
- 53m 3D socket connection editing.
- Undo/redo flow for 3D edits.
- Editor-only scope.

## Local validation

Run:

```text
cargo check --workspace
RUN_EDITOR_DEBUG.bat
```

Manual test path:

```text
Assets > Voxel Panels > 3D Preview
Export/load preview
Select instance
Click R rotation handle
Use Rotate 90° CW / CCW buttons
Use Mirror X / Mirror Y buttons
Hover/click existing connection lines
Delete selected/hovered connection
Begin connection draft and hover target socket to see snap ghost offset
Run Preview diagnostics and confirm overlap warnings appear when bounds intersect
Confirm only one top-level egui shell is rendered
```

## Next recommended phase

Phase 53o — Voxel Composition 3D Save Commit / Constraint Inspector

Focus:

- commit 3D preview edits back to RON explicitly
- unsaved preview-edit dirty state
- stronger per-instance constraint inspector
- connection details inspector
- collision-aware move blocking or warning mode
- save/load workflow cleanup
