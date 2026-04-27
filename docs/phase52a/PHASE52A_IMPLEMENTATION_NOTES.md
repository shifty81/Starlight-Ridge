# Phase 52A Implementation Notes

Phase 52A is intentionally a foundation patch. It adds the source-of-truth contracts needed before building larger systems.

## Why this phase comes first

The requested target has many coupled systems: 12 biomes, 5 grass families, 5 sand families, 5 water families, lava, crude oil, snow accumulation, rain puddling, `.vox` authored assets, a rotating orthographic camera, and mobile/editor help. Without shared data contracts, every system would invent its own IDs and assumptions.

## Core decision

Phase 52A treats biomes, materials, liquids, weather, layers, `.vox` assets, and editor help as modular data catalogs.

Future runtime/editor systems should consume these catalogs instead of hardcoding behavior.

## Important ids

- Manifest: `phase52_core_contract_manifest`
- Validation profile: `phase52_default_validation`
- Starter worldgen preset: `starter_farm_coastal_peninsula_phase52`
- Debug worldgen preset: `biome_test_matrix_phase52`

## Build risk

This phase adds a new module to `game_worldgen` and uses dependencies already present in that crate: `serde`, `ron`, and `anyhow`.
