# Terrain Autotile Reset - Phase 11

Phase 10 made the visual result worse because it treated the loose painted source sheet as if it were a strict tile atlas.

The actual sheet behaves like an 11-column by 13-row source with transparent gutters around the painted cells. Drawing those cells directly at map scale exposes the window clear color between every tile, which appears as a harsh black grid.

Phase 11 resets the terrain renderer to a stable base-layer path:

- use `terrain_atlas_phase11_packed.png`, a compact 11x13 atlas with 32x32 cells
- draw semantic base terrain only
- disable transition overlays temporarily
- keep decor/prop tiles as direct overlay cells
- keep the transition-layer code comments in place so the next pass can re-enable it correctly

Next proper autotile step:

1. Split source art into two atlas classes:
   - opaque packed base cells
   - transparent transition overlay masks
2. Define explicit roles for every overlay:
   - north edge
   - east edge
   - south edge
   - west edge
   - inner corners
   - outer corners
   - isolated/cap pieces
3. Generate transitions from a bitmask table instead of guessing atlas columns.
4. Render in fixed order:
   - base terrain
   - edge overlays
   - corner overlays
   - decor/props
