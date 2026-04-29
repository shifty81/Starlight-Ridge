# Starlight Ridge Test Source Rollup — 2026-04-27

This rollup was generated from the two uploaded snapshots:

- Base source: `Starlight_Ridge_source_2026-04-27_07-31-41.zip`
- Additive/reference source: `Starlight-Ridge-main.zip`

## Merge decision

The timestamped Phase 51F source is the active base because it contains the real egui world-layer editor implementation:

- `EditorMapState`
- mutable `content/maps/<map_id>/layers.ron` loading
- brush / erase / fill / pick wiring
- drag-painting support
- dirty state
- `Ctrl+S` / save with backup support

The `Starlight-Ridge-main` snapshot contributed Phase 52A worldgen/material/season/liquid/weather/vox/editor-help contracts, docs, wiki pages, and the `.vox` asset scanner module.

## Conflict handling

These Phase 51F test-base files were intentionally kept active instead of being overwritten by the Phase 52A snapshot:

- `crates/app/src/egui_editor.rs`
- `crates/game_data/src/defs.rs`
- `crates/game_data/src/lib.rs`
- `crates/game_data/src/loader.rs`
- `crates/game_data/src/registry.rs`
- `crates/game_data/src/validate.rs`
- `crates/shared_types/src/lib.rs`
- `content/maps/starter_farm/layers.ron`
- `content/maps/town/layers.ron`
- `assets/textures/terrain_atlas_phase17_generated.png`

Phase 52A reference versions of the conflicting map/atlas files were preserved under:

- `docs/rollup_conflict_reference/phase52a_main/`

## Added from Phase 52A/main snapshot

- Phase 52A contracts: `content/contracts/`, `content/biomes/phase52/`, `content/materials/phase52/`, `content/liquids/phase52/`, `content/seasons/phase52/`, `content/weather/phase52/`, `content/map_layers/phase52/`, `content/worldgen/phase52/`, `content/voxels/phase52/`, `content/editor_help/phase52/`
- Phase 52A docs/wiki/worldgen specs
- `.vox` scan/parser support: `crates/engine_assets/src/vox.rs`
- Phase 52 contract structs: `crates/game_worldgen/src/phase52_contracts.rs`
- Mobile/LAN web editor shell updates from the main snapshot

## Build/test note

This environment does not have `cargo` installed in the shell container, so I could not run `cargo check` here. On Windows, test with:

```bat
BUILD_MENU.bat
```

or directly:

```bat
cargo check
cargo run -p app --bin editor
```

Generated: 2026-04-27T18:39:42Z
Files in rollup: 616
Main-only files added: 195
Safe overrides from main: 5
