# Starlight Ridge Phase 17 Generated Assets In-Game Patch

Drop this zip into the project root and overwrite files when prompted.

## What this patch does

1. Keeps the Phase 16 runtime sprite wiring patch.
2. Adds normalized generated runtime textures:
   - `assets/textures/terrain_atlas_phase17_generated.png`
   - `assets/textures/player_walk.png`
   - `assets/textures/oceans_heart_bridge_phase17.png`
3. Updates `content/tiles/base_tileset.ron` to use the generated Phase 17 terrain atlas.
4. Keeps player metadata compatible with the current renderer:
   - 4 columns
   - 5 rows
   - 32x48 frame expectation from app-side sprite setup
5. Adds Ocean bridge sprite metadata without spawning it yet:
   - `content/metadata/oceans_heart_bridge_phase17.ron`
6. Preserves the large generated source images under:
   - `assets/art_source/generated_phase17/`

## Expected result

- The map renders using the cleaner generated Phase 17 terrain sheet.
- The player sprite appears using the cleaner generated player sheet.
- Ocean bridge assets are available as normalized grid sprites, but they do not spawn yet.
- The project remains aligned with named asset metadata instead of hard-coded raw atlas positions.

## Important note

The generated terrain sheet is an art improvement pass, not a final auto-tiler fix by itself. If the map still places the wrong terrain shapes, the next patch should improve resolver rules and map layer semantics, not only the art.
