# Phase 17 Generated Asset Pipeline

This patch turns the generated sheets into runtime-sized Starlight Ridge assets.

## Runtime files

| Purpose | Runtime file | Contract |
|---|---|---|
| Terrain | `assets/textures/terrain_atlas_phase17_generated.png` | 11x13 grid, 32x32 cells |
| Player | `assets/textures/player_walk.png` | 4x5 grid, 32x48 cells |
| Ocean bridge batch | `assets/textures/oceans_heart_bridge_phase17.png` | 4x4 grid, 32x32 cells |

## Why the generated masters are not direct drop-ins

The generated source images are larger RGB images with a visible checkerboard/grid baked into the pixels. The runtime renderer expects fixed-size sheets where frame UVs are derived from `columns` and `rows`.

The conversion pass normalized them by:

1. slicing each generated image by its intended grid,
2. removing border-connected checkerboard/grid background,
3. cropping visible sprite/tile content,
4. resizing each cell into the project's runtime cell size,
5. writing true PNG alpha where appropriate.

## In-game usage status

### Active now

- `base_tileset.ron` now points at the Phase 17 terrain atlas.
- `entity_sprite_sheet_phase5.ron` still points at `player_walk.png`, now replaced with the Phase 17 generated player sheet.
- The Phase 16 app-side sprite wiring is preserved, so the player sprite should still spawn at `player_start`.

### Prepared but not spawned yet

- `oceans_heart_bridge_phase17.png`
- `content/metadata/oceans_heart_bridge_phase17.ron`

The current app selects `phase5_entities` as the active sprite sheet, so the bridge sheet is metadata-ready but not rendered until the app supports either:
- multiple sprite sheets at once, or
- static prop spawning from metadata, or
- merging selected bridge sprites into the terrain atlas as named tile objects.

## Recommended next implementation

Add a small static prop renderer pass that supports a second sprite sheet and reads `props.ron` placements. That would allow `seagull`, `weak_tree`, `big_stone`, `driftwood_log`, and FX sprites to appear without mixing third-party-derived bridge art into the terrain atlas.
