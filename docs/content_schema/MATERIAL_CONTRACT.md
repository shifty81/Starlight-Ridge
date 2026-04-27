# Material Contract

## Purpose

A material is a semantic terrain/surface definition. It is not just an atlas coordinate.

Examples:

- lush grass
- wet shoreline sand
- swamp water
- dark volcanic sand
- snow overlay
- tilled soil

## Required fields

| Field | Type | Purpose |
|---|---|---|
| `id` | string | Stable material id. |
| `family` | enum | grass, sand, soil, stone, water_surface, snow, ash, etc. |
| `display_name` | string | User-facing name. |
| `physics_profile` | string | Movement/simulation behavior. |
| `season_variants` | object | spring/summer/fall/winter material or atlas variant ids. |
| `wet_variant` | optional string | Variant when wet. |
| `dry_variant` | optional string | Variant when dry. |
| `transition_profile` | string | Transition/autotile rule group. |
| `atlas_role` | string | Role used by atlas validation. |
| `absorbs_water` | bool | Whether rain/water can increase wetness. |
| `puddle_capacity` | number | Max shallow puddle depth before runoff. |
| `snow_retention` | number | How easily snow stays. |
| `editor_thumbnail` | string | Thumbnail or atlas tile reference. |
| `help_doc_id` | string | Editor wiki link. |

## Validation rules

- Materials exposed outdoors must declare seasonal handling.
- Materials that absorb water must define puddle capacity.
- Materials used by liquids must define valid transition profile.
- All atlas roles must resolve to atlas metadata.
- Help doc id must resolve.
