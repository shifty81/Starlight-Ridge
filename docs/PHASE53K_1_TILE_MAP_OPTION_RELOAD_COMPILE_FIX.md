# Phase 53k.1 — Tile Map Option Reload Compile Fix

## Purpose

Fix the compile failure reported after Phase 53k in the non-blocking reload path.

## Reported error

`ContentReloadPayload` still expected a concrete `TileMapRenderData`, while `build_tile_map_render_data(...)` now returns `Option<TileMapRenderData>` so maps with missing preview inputs can fall back safely instead of hard-failing the editor.

## Fix

- Updated `ContentReloadPayload.tile_map` from `TileMapRenderData` to `Option<TileMapRenderData>`.
- Updated `apply_content_reload_payload(...)` to assign the optional preview data directly.
- Preserved the background reload behavior.
- Preserved single-root egui shell behavior.
- No gameplay changes.

## Validation target

Run:

```bat
cargo check --workspace
```

Then run the editor and verify:

1. `Reload F5` does not hang the editor.
2. `Assets -> Voxel Panels -> 3D Preview` still loads Phase 53i preview exports.
3. Missing/fallback map preview data does not crash reload.
4. No nested editor shell appears.
