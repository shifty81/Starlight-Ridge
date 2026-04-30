# Phase 54a_1 — Build Fix + Standalone Editor Extraction Foundation

## Goal

Repair the Phase 54a compile regression and keep the project moving toward separate focused native editor executables instead of the web-editor path.

## Build fixes

- Removed the stale `ScenePivotMode` import from `egui_editor.rs`.
- Switched scene voxel preview selection bounds to use `game_data::defs::VoxelPivotMode`.
- Added handling for all `VoxelPivotMode` variants in the egui scene object screen-rect path:
  - `FeetCenter`
  - `Center`
  - `Origin`
  - `GripPoint`
- Added handling for all `VoxelPivotMode` variants in `voxel_scene.rs`.

## Standalone editor split direction

The extraction path should be native-first and staged in this order:

1. Character Editor
2. Voxel Panel Editor
3. World Editor
4. Pixel/Atlas Editor
5. Animation/Rig Editor
6. Asset Import/External Tools Editor

The main egui editor should become a launcher/status hub over time. Heavy editor code should move out of `egui_editor.rs` into focused modules/crates before each standalone executable is introduced.

## Next implementation phase

`Phase 55a — Native Editor App Split Manifest + Launcher Contracts`

Expected focus:

- Add native editor app manifest.
- Add app ids, executable names, route names, and responsibilities.
- Add launcher contract structs.
- Add docs for extraction boundaries.
- No large UI move yet.

After that:

`Phase 55b — Character Editor Extraction`

Expected focus:

- Move character VOX preview state and rig preview data out of the monolithic egui editor.
- Add a standalone Character Editor binary shell.
- Keep the main editor as a launcher/status hub.
