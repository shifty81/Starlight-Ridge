# Phase 6 In-Game Integration

This update moves the generated terrain atlas and entity sprite sheet from loose assets into the runtime bootstrap path.

## Data flow
1. `game_data` loads tileset metadata from `content/tiles/*.ron`
2. `game_data` loads sprite sheet metadata from `content/metadata/*.ron`
3. `game_data` loads map tile layers from `content/maps/<map>/layers.ron`
4. `engine_render_gl` loads the matching textures from `assets/textures`
5. The renderer draws terrain tiles first, then sprite instances for props and spawns

## Current active sample map
- `content/maps/town/map.ron`
- `content/maps/town/layers.ron`
- `content/maps/town/props.ron`
- `content/maps/town/spawns.ron`

## Current limitations
- The renderer issues one quad draw per tile/sprite; batching should come later.
- The sprite placements are prototype placements.
- The generated sprite sheet is used as starter art, not final production content.
