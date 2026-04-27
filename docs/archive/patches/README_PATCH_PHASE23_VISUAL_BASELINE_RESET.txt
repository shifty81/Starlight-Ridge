Starlight Ridge Phase 23 - Visual Baseline Reset

Purpose:
- Stop the visual regression loop caused by incomplete runtime terrain transition substitution.
- Remove the highly visible tile-grid look from the Phase 17 generated terrain atlas.
- Remove the fake repeated cliff border from starter_farm until a real cliff transition set exists.

Replaced files:
- crates/app/src/lib.rs
- assets/textures/terrain_atlas_phase17_generated.png
- content/maps/starter_farm/layers.ron

What changed:
- Terrain layers now draw authored tile IDs directly through base_tileset.ron.
- The old legacy transition substitution path is disabled/removed from active rendering.
- Object/prop/decor layers remain separate from terrain rendering.
- The terrain rows in the active atlas were regenerated as cleaner seamless 32x32 tiles.
- starter_farm ground rows no longer use the repeated placeholder cliff border around the top/left edge.

Expected result:
- No flower/chair/object atlas row sampling for terrain.
- Much less visible grid lining across grass, sand, water, paths, and farm soil.
- Coast should be readable as sand/shallow water/deep water.
- Top/left fake cliff wall should be gone.

Notes:
- This is intentionally a visual-safe reset, not the final auto-tiler.
- A proper next auto-tiler should use a complete terrain transition contract and authored transition atlas, not guessed row/column substitutions.
