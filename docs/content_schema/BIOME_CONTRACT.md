# Biome Contract

## Purpose

A biome defines the environmental rules used by worldgen, simulation, asset placement, weather, and editor validation.

## Required fields

| Field | Type | Purpose |
|---|---|---|
| `id` | string | Stable registry id. |
| `display_name` | string | User-facing name. |
| `description` | string | Editor/wiki description. |
| `temperature_range` | range | Preferred normalized temperature. |
| `moisture_range` | range | Preferred normalized moisture. |
| `elevation_range` | range | Preferred normalized elevation. |
| `salinity_range` | range | Preferred normalized salinity. |
| `primary_materials` | string list | Preferred terrain material ids. |
| `allowed_liquids` | string list | Liquid ids allowed in this biome. |
| `spawn_tables` | string list | Biome spawn/prop tables. |
| `weather_profile` | string | Default weather profile id. |
| `season_profile` | string | Default season profile id. |
| `adjacency_rules` | object | Allowed/weighted neighboring biome ids. |
| `help_doc_id` | string | Editor wiki link. |

## Validation rules

- `id` must be unique.
- All material ids must resolve.
- All liquid ids must resolve.
- Weather and season profiles must resolve.
- Adjacency rules must not reference missing biome ids.
- Every biome must include help metadata.
