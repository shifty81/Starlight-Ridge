# Worldgen Preset Contract

## Purpose

A worldgen preset defines how to generate a scene or family of scenes.

## Required fields

| Field | Type | Purpose |
|---|---|---|
| `id` | string | Stable preset id. |
| `display_name` | string | Editor-facing name. |
| `scene_template` | string | Template id. |
| `allowed_biomes` | list | Biomes available to the generator. |
| `biome_weights` | object | Weighted biome bias. |
| `noise_profiles` | object | Elevation/moisture/temperature noise settings. |
| `hydrology_profile` | string | River/lake/marsh routing settings. |
| `feature_profiles` | list | Tree/rock/structure placement settings. |
| `authored_overlays` | list | Protected authored regions to apply. |
| `validation_profile` | string | Validation ruleset id. |
| `help_doc_id` | string | Editor wiki link. |

## Validation rules

- Every allowed biome exists.
- Every noise profile is deterministic and serializable.
- Scene template exists.
- Hydrology profile exists.
- Feature profiles exist.
