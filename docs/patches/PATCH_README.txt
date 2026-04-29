Starlight Ridge Patch 11 - Terrain Atlas Contract Reset

Why this patch exists:
- Patch 10 made the scene worse because the renderer used a loose transparent source atlas as if it were a strict atlas.
- The sheet is effectively 11 columns by 13 rows, not the 12-column contract in base_tileset.ron.
- Transparent gutters were being drawn as part of every tile cell, exposing the dark clear color as a black grid.

Files changed:
- crates/app/src/lib.rs
- content/tiles/base_tileset.ron
- assets/textures/terrain_atlas_phase11_packed.png
- docs/data_contracts/terrain_autotile_reset_phase11.md

What changes:
- Uses a packed 11x13 terrain atlas with 32x32 cells.
- Disables guessed transition overlays for now.
- Keeps stable base terrain drawing so the scene stops degrading.
- Leaves the transition plan documented for the next real autotile pass.

After extracting:
  ./build.sh check
  ./build.sh release
  ./build.sh run

Expected log marker:
  prepared packed base tile renderer data

Next real autotile pass:
- Build explicit alpha overlay tiles for edges/corners.
- Add a role map for each overlay tile.
- Use bitmask transitions instead of guessed atlas columns.
