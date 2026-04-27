# Terrain Transition Resolver - Phase 12

This pass wires the existing terrain transition helpers into the active render path.

## What changed

- Semantic terrain layers now render in two passes:
  1. stable full base cells
  2. conservative transition overlay cells
- `same_transition_group_at`, `TerrainGroup`, and `transition_column` are now used by `push_terrain_layer_tiles`.
- Transition overlays are counted in the existing render prep log as `transition_overlays`.
- The resolver only uses packed atlas columns that already exist in `terrain_atlas_phase11_packed.png`.

## Current conservative atlas contract

The current atlas is not yet a full named-role autotile atlas. The resolver therefore uses a narrow set of safe columns:

- columns `0..=2`: base/detail cells
- column `3`: rough/all-around border fallback
- column `4`: north/south straight exposure family
- columns `5..=7`: corner/east/west exposure families

This is intended to improve terrain borders without reintroducing the earlier clear-grid/gutter failure.

## Remaining visual gaps

These are the next gaps to solve after this pass is tested in-game:

1. **True named overlay roles**
   - Add explicit IDs like `water_edge_n`, `sand_corner_sw`, `tilled_inner_ne` instead of relying on column assumptions.

2. **8-neighbor masks**
   - Current resolver uses north/east/south/west only.
   - Full shoreline/path blending should inspect diagonals for inner corners.

3. **Priority-based terrain composition**
   - Current cells still start from their own base terrain.
   - Better shoreline blending should allow a lower-priority base terrain and a higher-priority overlay terrain.

4. **Editor debug overlay**
   - Add a toggle that draws tile IDs, transition group, and mask value per tile.

5. **Autotile test maps**
   - Add small dedicated maps with water islands, path intersections, tilled fields, cliff edges, and sand/grass coast tests.
