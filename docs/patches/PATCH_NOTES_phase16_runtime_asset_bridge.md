# Starlight Ridge Phase 16 Runtime Asset Bridge Patch

Drop this zip into the project root and overwrite files when prompted.

## What this patch changes

1. Fixes starter farm validation by adding named tile aliases for:
   - `rock`
   - `sign`
   - `farmhouse_roof`
   - `farmhouse_wall`
   - `shipping_bin`
   - `well`

2. Updates `content/tiles/base_tileset.ron` to use the active Phase 16 atlas:
   - `assets/textures/terrain_atlas_phase16_refined.png`
   - `columns: 16`
   - `rows: 16`

3. Adds missing terrain variant IDs referenced by the coastal grassland biome pack so the semantic terrain resolver has real tile IDs to choose from instead of collapsing heavily to fallbacks.

4. Wires the existing `player_walk.png` metadata into the existing OpenGL sprite renderer.

5. Spawns a static player sprite from the active map's `player_start` / `player` spawn.

6. Adds a clean third-party intake area for a small Ocean’s Heart asset batch:
   - source PNG/DAT pairs under `assets/third_party/oceans_heart/`
   - attribution under `third_party/oceans_heart_free_assets/`
   - intake metadata under `content/third_party/oceans_heart/`

## Expected result

- Content validation should no longer fail because of missing starter-farm object tile IDs.
- The map should still render using the active Phase 16 atlas contract.
- A static player sprite should appear at `player_start`.
- Ocean’s Heart assets are available for later conversion without being mixed into first-party Starlight Ridge art or runtime combat systems.

## Not included yet

- Animated player walking.
- Collision movement.
- Ocean’s Heart combat/enemy spawning.
- DAT-to-runtime animation conversion.
- Final replacement art for starter farmhouse/well/bin object aliases.

## Verification note

This patch was source-audited and mechanically validated for referenced tile IDs. It was not compiled in this container because Cargo is unavailable here.
