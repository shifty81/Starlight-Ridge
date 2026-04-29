# Phase 52c — Biome Rule Packs + Seasonal WorldGen Resolver

## Goal

Move world generation away from one generic terrain contract and into explicit, season-aware biome rule packs. The generator can now emit semantic terrain such as grass, forest floor, sand, shallow water, deep water, coast foam, mud, snow, rocks, and protected masks. The resolver maps those semantic IDs into concrete tile IDs by biome and season before the later autotile bake pass creates transition layers.

## Added

- `content/worldgen/biome_rule_packs/coastal_peninsula_v1.ron`
  - Canonical starter farm/coastal peninsula biome rule pack.
  - Defines spring, summer, fall, and winter tile selection.
  - Carries liquid, puddle, snow accumulation, and collision/walkability hints.
  - Includes future-facing mud/snow behavior so weather physics can hook into it later.

- `crates/game_data/src/defs.rs`
  - `WorldgenBiomeRulePackDef`
  - `WorldgenBiomeTerrainRuleDef`
  - `WorldgenSeasonTileDef`
  - `WorldgenTerrainVariantRuleDef`
  - `WorldgenSeasonalOverlayDef`

- `crates/game_data/src/loader.rs`
  - `load_worldgen_biome_rule_pack`

- `crates/game_data/src/registry.rs`
  - `worldgen_biome_rule_packs`
  - `has_phase52c_worldgen_biome_rule_packs`
  - `active_worldgen_biome_rule_pack`

- `crates/game_data/src/validate.rs`
  - Validates phase 52c packs, seasons, semantic rule uniqueness, fallback tile IDs, seasonal tiles, variant weights, and overlay targets.

- `crates/game_worldgen/src/seasonal_resolver.rs`
  - `WorldgenSeason`
  - `TerrainResolveRequest`
  - `ResolvedTerrainTile`
  - `ResolvedSeasonalScene`
  - `resolve_terrain_tile`
  - `resolve_generated_scene_to_tile_ids`
  - `semantic_terrain_to_rule_id`

- `crates/app/src/egui_editor.rs`
  - World → Terrain Rules is now a real Phase 52c resolver panel instead of placeholder notes.
  - Shows terrain contract counts, transition counts, biome rule pack counts, bake safety counts, and a per-season resolver sample table.
  - Includes a resolver report log action.

## What this enables next

Phase 52d can expand the tile atlas and preview families using this resolver contract instead of hardcoding terrain choices. The next bake step should consume `ResolvedSeasonalScene` and write base terrain, water, overlay, transition, foam, and animation layer output into editable map layers.

## Notes

Some tile IDs are intentionally future placeholders. This is deliberate: the resolver contract is now in place before the full seasonal atlas art pass. Validation enforces shape and safety but does not require every placeholder tile to already exist in the atlas.
