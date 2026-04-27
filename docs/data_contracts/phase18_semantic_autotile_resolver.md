# Phase 18 Semantic Autotile Resolver

Phase 18 wires the Phase 17 terrain contracts into a first runtime visual path.
The goal is not final art yet. The goal is to make the game render terrain from
semantic terrain IDs first, then resolve atlas tile IDs second.

## New runtime path

1. `content/maps/*/layers.ron` may now contain a `semantic_terrain` layer.
2. That layer maps symbols to terrain IDs such as `grass`, `sand`,
   `shallow_water`, `deep_water`, `path`, `tilled_dry`, and `tilled_watered`.
3. `crates/game_world/src/autotile.rs` resolves the semantic grid into draw
   tiles:
   - base terrain layer
   - transition overlay layers
   - collision-blocking metadata for water cells
4. `crates/app/src/lib.rs` converts the resolved tile IDs into the existing
   OpenGL tile renderer's `TileInstance` list.

## Why this exists

The previous renderer could only draw visual tile IDs from map layers. That made
coastlines and ponds depend on manually placed atlas cells. Phase 18 starts the
proper pipeline:

semantic terrain -> resolver masks -> atlas tile IDs -> renderer

This is the bridge needed before better biome art, procedural world generation,
and editor terrain brushes can work correctly.

## Test scenes

Two semantic test scenes are included:

- `content/maps/autotile_test_coast`
- `content/maps/autotile_test_pond`

The game now prefers `autotile_test_coast` on boot when it exists so the resolver
is visible immediately after patching.

## Current resolver limits

This is a first resolver pass. It uses deterministic weighted base variants and
4-bit cardinal transition masks. The structure is ready for full 47-tile and
quarter-tile solving, but the current atlas still only contains placeholder
transition cells.

Phase 19 should replace those placeholders with a proper coastal biome sheet.
