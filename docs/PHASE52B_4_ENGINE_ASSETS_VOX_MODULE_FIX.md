# Phase 52b.4 — Engine Assets VOX Module Fix

## Purpose

The app editor imports `engine_assets::vox::{scan_vox_files, VoxAssetInfo}` for the VOX Models asset tab, but the `engine_assets` crate did not expose a `vox` module. This caused `cargo check` to fail in `crates/app/src/egui_editor.rs` with unresolved import `engine_assets::vox`.

## Changes

- Added `crates/engine_assets/src/vox.rs`.
- Exported the module from `crates/engine_assets/src/lib.rs` with `pub mod vox;`.
- Added `VoxAssetInfo` with the fields already consumed by the egui VOX panel.
- Added `scan_vox_files(project_root)` scanning:
  - `assets/voxels/`
  - `assets/models/`
  - `content/voxels/`
- Added lightweight MagicaVoxel `.vox` parsing for:
  - `SIZE` dimensions
  - `XYZI` voxel counts
  - `RGBA` palette color counts
- Bad `.vox` files are logged as warnings and listed with zeroed metadata instead of crashing the editor scan.

## Expected result

`cargo check` should move past the previous `engine_assets::vox` unresolved import and continue checking the remaining crates.
