# Voxel Panel Preview Exports

Phase 53i writes non-rendering 3D preview handoff files here.

These RON files are generated from `Assets -> Voxel Panels -> Compositions` with the `Export 3D preview RON` button.

They are intended for the later real 3D voxel viewport and should not be treated as runtime gameplay content yet.


## Phase 53j

The egui Voxel Panel Designer now includes a read-only 3D Preview panel that loads Phase 53i preview RON exports and draws baked voxel boxes, bounds, socket gizmos, and connection gizmos with orbit/pan/zoom camera controls.

## Phase 53k

The 3D Preview panel now treats this directory as preview export history. Use `Refresh export list` to discover saved preview RON files, then load a selected export for read-only inspection with floor grid, XYZ axis gizmo, material legend, hover labels, selected instance/socket highlights, and diagnostics.
