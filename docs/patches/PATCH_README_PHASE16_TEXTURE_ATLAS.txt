Starlight Ridge Phase 16 - Refined Texture Atlas

Drop-in files:
- assets/textures/terrain_atlas_phase16_refined.png
- content/tiles/base_tileset.ron
- docs/data_contracts/phase16_texture_atlas_contract.md

What changed:
- Replaced the rudimentary 11x13 atlas reference with a cleaner 16x16 atlas.
- Kept the tileset id as base_tiles so starter_farm and town keep loading.
- Preserved all tile ids currently referenced by maps.
- Added reserved rows/columns for water animation, buildings, debug overlays, and future auto-tile transitions.
- Fixed the old fence_v / stone_wall coordinate overlap by moving stone_wall to its own atlas cell.

Expected result:
- The current farm map should render with less harsh repeated grid/noise.
- Terrain should look cleaner because base terrain cells are opaque and do not depend on black/transparent atlas gutters.
- This does not yet implement the full metadata-driven auto-tiler; it prepares the atlas contract for it.
