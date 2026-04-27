# Map Layer Contract

## Purpose

A map is a stack of semantic layers, not a flat tile array.

## Required top-level map fields

| Field | Type | Purpose |
|---|---|---|
| `map_id` | string | Stable map/scene id. |
| `display_name` | string | User-facing scene name. |
| `dimensions` | object | Width/height in cells. |
| `tile_size_px` | integer | Usually 32. |
| `world_seed` | integer | World seed if generated. |
| `scene_seed` | integer | Scene-specific seed. |
| `layers` | list | Editor-visible layer data. |
| `derived_layer_metadata` | object | Runtime rebuild hints. |
| `protected_regions` | list | Authoring locks. |
| `validation_state` | object | Cached validation metadata. |

## Cell stack expectation

A cell can have entries from multiple layers:

```text
Base Terrain -> Transition -> Liquid -> Ground State -> Ground Cover -> Props/Structures -> Collision/Interaction -> Regions -> Ambience
```

The editor inspector should show this full stack at the cursor.

## Validation rules

- Map dimensions must match all layer buffers.
- Required layers must exist.
- Derived layers are rebuildable and should not be directly hand-edited by default.
- Protected regions prevent generator overwrites.
- Layer ids must resolve to known layer types.
