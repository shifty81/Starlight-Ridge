Starlight Ridge Phase 22 - Atlas Lookup Render Stabilizer

This patch fixes the visual regression where terrain started rendering as flowers,
chairs, fences, roofs, and other object/proptype cells.

Root cause:
crates/app/src/lib.rs was still using old hard-coded terrain atlas row numbers
inside TerrainKind::atlas_row() and TerrainKind::base_column(). The active Phase 17
atlas is defined by content/tiles/base_tileset.ron instead. When the hard-coded rows
no longer matched the active atlas, water/cliff/sand/path cells sampled prop and
object rows.

What changed:
- Replaces crates/app/src/lib.rs.
- Stops terrain rendering from guessing atlas row/column coordinates.
- Resolves terrain cells through the active base_tileset.ron atlas_lookup.
- Keeps starter_farm as the normal launch map.
- Converts the legacy transition logic from unsafe overlay row guessing into
  atlas-safe tile-id substitution using named tiles such as grass_path_n,
  grass_sand_e, shore_w, depth_s, etc.
- Keeps objects/props out of the terrain resolver.

Expected result:
- Water should render as water again.
- Cliff/sand/path/soil should stop sampling prop rows.
- The right side of the map should no longer become repeated flowers/chairs/fences.
- Static props remain separate; this patch does not mix props into terrain.

Apply:
Extract this zip over the project root and overwrite files, then run cargo check.
