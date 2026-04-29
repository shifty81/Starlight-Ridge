# Phase 52i — World Voxel Object Placement Scaffold

Phase 52h made the Phase 52 contracts load and added safe World → 3D Viewport / World → Voxels tabs. Phase 52i adds the first editable bridge between those contracts and real world content.

## Implemented

- Added editable voxel/world-object placement records for the active map.
- Added `content/maps/starter_farm/voxel_objects.ron` as the starter placement file.
- Added in-editor load/save support with timestamped backups.
- Added selected Phase 52 VOX contract placement.
- Added selected parseable `.vox` placement when actual `.vox` files exist.
- Added a placed-object list and inspector in World → Voxels.
- Added transform, yaw, footprint, height, collision kind, source path, lock state, and notes editing.
- Updated World → 3D Viewport to draw all placed object bounds and highlight the selected object.
- Preserved the single-shell egui rule: no full editor shell render calls are made from world/asset panels.

## Data contract

`content/maps/<map_id>/voxel_objects.ron` stores:

- `schema_version`
- `map_id`
- `objects[]`
- object id/display name
- source kind/id/path
- x/y/z transform
- yaw
- footprint width/height
- height
- anchor
- collision kind
- locked flag
- notes

This is intentionally separate from the existing 2D `props.ron` so the 3D/voxel pipeline can mature without breaking the current 2D runtime props path.

## Current limitations

- This is still a debug bounds viewport, not full cube mesh rendering.
- Viewport selection/picking is not implemented yet.
- Drag-to-move handles are not implemented yet.
- `.vox` source files are still expected to be missing until real MagicaVoxel assets are added.
- Runtime gameplay does not consume `voxel_objects.ron` yet.

## Recommended next phase

Phase 52j should add viewport picking and drag-to-move handles:

1. Click object bounds in World → 3D Viewport to select the placement.
2. Drag selected object in grid space.
3. Add snap-to-grid and free-move modes.
4. Add duplicate selected object.
5. Add per-object validation warnings.
6. Add object source existence and footprint collision checks.
