Starlight Ridge Phase 18 — Semantic Autotile Resolver + Coast/Pond Test Scene

Drop this patch into the project root and overwrite files.

What changed:
- Adds crates/game_world/src/autotile.rs.
- Exposes game_world::autotile from game_world/src/lib.rs.
- Updates app rendering to detect semantic_terrain layers.
- Converts semantic terrain IDs into existing TileInstance render data.
- Adds test maps for a curved coastline and ponds.
- Adds placeholder named transition tiles to base_tileset.ron.
- Relaxes validation so semantic terrain layers may reference terrain IDs instead of raw atlas tile IDs.
- Fixes a Phase 17 validation typo in active_transition_sets validation.

Expected result:
- The app should boot into autotile_test_coast if the map exists.
- You should see a cleaner terrain-only coast/pond test scene with no decor clutter.
- This is still placeholder art. The important part is that base terrain and transitions are selected by terrain rules now.

Next step:
Phase 19 should add a dedicated coastal biome tile sheet with real transition art for the masks now being resolved here.
