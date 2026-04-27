# Liquid Contract

## Purpose

A liquid defines physical and visual behavior for grid-based shallow liquid simulation.

Canonical Phase 52 liquids:

- clear freshwater
- coastal saltwater
- swamp water
- deep water
- icy/glacial water
- lava
- crude oil

## Required fields

| Field | Type | Purpose |
|---|---|---|
| `id` | string | Stable liquid id. |
| `display_name` | string | User-facing name. |
| `viscosity` | number | Higher means slower flow. |
| `flow_rate` | number | Max transfer per step. |
| `min_depth_visible` | number | Smallest depth rendered. |
| `max_depth` | number | Maximum cell depth. |
| `evaporation_rate` | number | Depth loss over time. |
| `seep_rate` | number | Loss into absorbent ground. |
| `temperature` | number | Default liquid temperature/hazard. |
| `flammable` | bool | Can ignite. |
| `damages_entities` | bool | Causes damage on contact. |
| `animation_profile` | string | Render animation id. |
| `allowed_materials` | string list | Materials this liquid can occupy. |
| `blocked_by_materials` | string list | Materials that block or contain it. |
| `help_doc_id` | string | Editor wiki link. |

## Validation rules

- Lava must have damage/hazard behavior.
- Crude oil must be flammable.
- Water liquids must define freeze behavior if used in cold biomes.
- Every liquid must provide animation/render fallback.
