# Starlight Ridge Patch 3 — Atlas Contract Resolver Rewrite

## Patch focus

This patch replaces the loose atlas/test rendering path with a strict semantic terrain contract.

## What changed

- Added `TileRole` and `SemanticTerrainResolver` in `game_world`.
- Every map symbol now resolves through an explicit tile id and strict terrain role.
- Unknown tile ids now fail fast instead of silently picking unrelated atlas cells.
- Added deterministic same-row variation for grass, dirt, sand, floor, tilled, cliff, and water roles.
- Added a conservative water shoreline fix: water edge tiles resolve to water-bank variants instead of random non-water atlas cells.
- Replaced the debug checker quad with an OpenGL tilemap pipeline that renders the real `terrain_atlas_phase5.png` atlas.
- Added a semi-authored `starter_farm` scene with farm pathing, tilled/watered soil, wooded edges, pond, beach, and coast water.
- Changed bootstrap map selection to prefer `starter_farm` over `town`.
- Tightened map validation for layer dimensions, one-character legend symbols, and known tileset ids.
- Fixed the old town transition trigger so it points back to `starter_farm` and sits inside the town map bounds.

## Target files

- `crates/game_world/src/lib.rs`
- `crates/engine_render_gl/src/lib.rs`
- `crates/app/src/lib.rs`
- `crates/game_core/src/modes.rs`
- `crates/game_data/src/validate.rs`
- `content/tiles/base_tileset.ron`
- `content/maps/starter_farm/map.ron`
- `content/maps/starter_farm/layers.ron`
- `content/maps/starter_farm/props.ron`
- `content/maps/starter_farm/spawns.ron`
- `content/maps/starter_farm/triggers.ron`
- `content/maps/town/triggers.ron`

## Expected visual result

The map should no longer look like a shuffled atlas test. The starter farm should appear as an intentional top-down tilemap with:

- readable grass/wooded areas,
- a clear central path,
- farmhouse placeholder flooring,
- tilled and watered crop plots,
- a west pond,
- a sand-to-water coastal edge on the east/south side,
- no large random atlas-cell jumbles.

This is still a renderer/data-contract patch, not a full prop/entity overlay pass. Props are still metadata placeholders until the next overlay/sprite pass.
