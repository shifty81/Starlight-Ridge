# Terrain Transition Layers - Phase 10

The renderer now treats semantic terrain layers as layered draw output instead of a single resolved tile stream.

## Draw order

1. **Terrain base pass**
   - Draws one stable full-body tile for every terrain cell.
   - This keeps the world filled and prevents transparent transition art from showing the debug grid or clear color.

2. **Terrain transition pass**
   - Draws edge/corner variants after all base terrain cells are down.
   - Neighbor checks use terrain families instead of only exact tile IDs.
   - Example: `water_shallow` and `water_deep` are both `Water`, and `tilled_dry` plus `tilled_watered` are both `Tilled`.

3. **Direct/decor pass**
   - Non-terrain layers such as flowers, bushes, props, fences, and later placed objects draw after terrain.
   - These layers are never autotiled.

## Current atlas role assumptions

The current atlas is not a full 47-tile Wang set. The resolver uses only conservative terrain roles:

- column 0: stable full base
- columns 1-2: sparse detail variation
- column 3: rough/all-around border fallback
- column 4: straight horizontal exposure
- columns 5-7: corner/vertical exposure family

Avoid using hole/detail/prop columns for automatic terrain transitions until a dedicated transition atlas is authored.

## Next best visual upgrade

Create a dedicated terrain transition atlas with explicit named roles:

- `<terrain>_base`
- `<terrain>_edge_n`
- `<terrain>_edge_e`
- `<terrain>_edge_s`
- `<terrain>_edge_w`
- `<terrain>_corner_ne`
- `<terrain>_corner_nw`
- `<terrain>_corner_se`
- `<terrain>_corner_sw`
- `<terrain>_inner_ne`
- `<terrain>_inner_nw`
- `<terrain>_inner_se`
- `<terrain>_inner_sw`

Once those roles exist, the resolver can move from conservative 4-neighbor matching to full 8-neighbor mask matching.
