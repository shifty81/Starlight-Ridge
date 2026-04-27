# Phase 23 Visual Baseline Reset

This patch rolls the renderer back to a stable visual contract while preserving the compile fixes and atlas lookup safety from prior patches.

## Changes

- `crates/app/src/lib.rs`
  - Keeps `starter_farm` as the preferred launch map.
  - Keeps terrain lookup routed through `base_tileset.ron`.
  - Disables/removes incomplete legacy transition substitution for authored tile layers.
  - Keeps object/prop/decor layers out of terrain auto-tiling.

- `assets/textures/terrain_atlas_phase17_generated.png`
  - Regenerates the terrain rows as cleaner seamless 32x32 tiles.
  - Preserves the same atlas dimensions and named-tile contract.

- `content/maps/starter_farm/layers.ron`
  - Replaces repeated placeholder cliff border cells with grass so the map no longer shows a fake rocky wall around the top/left edge.

## Why

The prior transition pass was still trying to solve a visual problem with incomplete directional tile assets. The result was technically atlas-safe, but still visually wrong. This patch prioritizes a clean, stable baseline before adding a real transition contract.

## Next recommended step

Build a proper `terrain_transition_sets` implementation with complete authored tiles for cliff, grass/path, grass/sand, shore, water-depth, dry/wet soil, and field borders. Only then should runtime transition substitution be re-enabled.
