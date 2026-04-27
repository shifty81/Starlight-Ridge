# Phase 7 Tilemap + Editor Implementation

This pass turns the Phase 5 content into the live render path.

## Implemented

- `crates/engine_render_gl/src/lib.rs` now builds a world-space OpenGL/glow renderer.
- `content/maps/town/layers.ron` is loaded through `game_data` and converted into static tile geometry.
- `content/tiles/base_tileset.ron` maps layer symbols to atlas coordinates.
- `assets/textures/terrain_atlas_phase5.png` is bound as the terrain atlas.
- `content/metadata/entity_sprite_sheet_phase5.ron` and `assets/textures/entity_sprite_sheet_phase5.png` are bound for player, NPC, and prop sprites.
- `crates/app/src/bin/editor.rs` launches the same project in editor-overlay mode.
- Editor scaffolding now includes tools, inspector rows, content bridge snapshots, and undo command types.

## Editor controls

- `Tab` toggles the overlay in the game binary.
- `1` selects the Select tool.
- `2` selects Terrain Paint.
- `3` selects Prop Place.
- `4` selects NPC Place.
- `5` selects Trigger Place.

## Current editor scope

This is the first in-engine editor pass. It renders the map grid and content-object overlays against the real map data. The next pass should add mouse picking, edit application, RON write-back, and hot-reload.
