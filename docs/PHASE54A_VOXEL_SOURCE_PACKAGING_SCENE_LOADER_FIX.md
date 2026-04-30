# Phase 54a — Voxel Source Packaging + Scene Loader Fix

## Goal

Stabilize the voxel-first data path before extracting the Character Editor into a standalone executable.

## Changes

- Kept `crates/game_data/src/vox_rig.rs` and the `pub mod vox_rig;` module export in the source bundle.
- Promoted voxel asset registry contracts into `game_data::defs`.
- Promoted scene voxel object-set contracts into `game_data::defs`.
- Added `load_voxel_asset_registry` and `load_voxel_object_set` helpers in `game_data::loader`.
- Registered voxel asset registries and scene voxel object sets in `ContentRegistry`.
- Added Phase 54a validation for voxel asset IDs, object IDs, object-to-asset links, scale, source path, layer, and scene/map references.
- Updated the egui scene voxel preview loader to use the promoted `game_data` contracts instead of app-local placeholder structs.
- Fixed the RON root-shape mismatch by deserializing the existing `VoxelAssetRegistry(...)` and `VoxelObjectSetDef(...)` files through matching struct-like file wrapper enums.

## Expected result

The scene voxel preview path should no longer fail with:

```text
Expected struct `SceneVoxelObjectSetDef` but found `id`
```

The voxel scene/object registry is now stable enough to build the next phase:

```text
Phase 54b — Character Preview Assembler + Modular VOX Loader
```

## Notes

This phase intentionally does not extract a standalone Character Editor yet. The data path is now prepared first so the extracted tool does not inherit broken placeholder contracts.
