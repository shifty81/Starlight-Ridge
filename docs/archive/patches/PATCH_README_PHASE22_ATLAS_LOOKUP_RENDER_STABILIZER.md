# Phase 22 - Atlas Lookup Render Stabilizer

This is a corrective patch after the Phase 21 transition activation exposed an older renderer assumption.

The bug was not the map data. The active `base_tileset.ron` points at `terrain_atlas_phase17_generated.png`, but the legacy terrain renderer still guessed atlas rows directly from `TerrainKind`. Those guessed rows belong to an older atlas contract, so the renderer sampled object/prop rows for terrain.

## Main fix

`push_terrain_layer_tiles` now resolves every terrain draw through the active tileset lookup instead of direct row/column guesses.

## Transition behavior

Legacy transitions are now tile-id substitutions, not overlay draws from guessed atlas rows. For example:

- `path_sand` beside grass/dirt can resolve to `grass_path_n/e/s/w`.
- `sand` beside grass can resolve to `grass_sand_*`.
- `water_shallow` beside sand can resolve to `shore_*`.
- `water_deep` beside shallow water can resolve to `depth_*`.

Only named tiles in `base_tileset.ron` are drawn.
