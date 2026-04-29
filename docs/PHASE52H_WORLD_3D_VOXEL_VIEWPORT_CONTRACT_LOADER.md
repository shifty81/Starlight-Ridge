# Phase 52h — World 3D Voxel Viewport + Phase 52 Contract Loader

## Purpose

This phase establishes the foundation for the World menu to inspect Phase 52 worldgen/voxel data without attempting the full voxel editor in one patch.

## Implemented

- Unified `engine_assets::vox` so the real `crates/engine_assets/src/vox.rs` parser is exported instead of the older inline metadata-only module.
- Exposed `game_worldgen::phase52_contracts` from the `game_worldgen` crate.
- Added Phase 52 type aliases, loaders, registry fields, summary counts, and validation hooks in `game_data`.
- Loaded `content/contracts/phase52_manifest.ron` and its child catalogs into `ContentRegistry`.
- Added non-fatal missing `.vox` source reporting through `ContentRegistry::missing_phase52_vox_sources()`.
- Added World → `3D Viewport` in the egui editor.
- Added World → `Voxels` in the egui editor.
- Added a safe single-panel egui debug viewport with camera controls, grid drawing, and voxel/contract bounds preview.
- Added visible Phase 52 VOX contract listing and missing source warnings.

## Notes

Missing `.vox` files are warnings, not startup blockers. The current source intentionally ships VOX contracts before the actual MagicaVoxel source models exist.

## Next Recommended Phase

Phase 52i should add editable world voxel/object placement records:

- selected VOX contract/object placement state
- transform/anchor/footprint inspector
- viewport picking and selection outline
- saveable world-object layer contract
- first pass placement/remove tools

## Validation

Cargo/Rust is not installed in the patch container, so this phase was source-audited only. Run `cargo check` locally after extracting the source.
