# Phase 5 Tilemap + Sprite Notes

This scaffold now includes generated starter art assets for the first tilemap/sprite pass.

## Added assets

- `assets/textures/terrain_atlas_phase5.png`
- `assets/textures/entity_sprite_sheet_phase5.png`

## Added content metadata

- `content/tiles/base_tileset.ron`
- `content/metadata/entity_sprite_sheet_phase5.ron`

## Intended next code step

Implement a small importer that:

1. Loads `base_tileset.ron`
2. Loads `terrain_atlas_phase5.png`
3. Builds UVs from tile `(x, y)` entries
4. Uses a tile layer file to draw quads in world space
5. Loads `entity_sprite_sheet_phase5.ron`
6. Draws a player marker and selected map entities as sprites

## Important note

The generated atlas and sprite sheet are best treated as **starter prototype art**. They are suitable for bootstrapping the renderer and content pipeline, but the exact atlas coordinates may need to be refined once a proper importer and per-sprite slicing preview tool exist.
