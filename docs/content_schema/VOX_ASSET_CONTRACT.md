# VOX Asset Contract

## Purpose

`.vox` files are source assets. The runtime should consume baked sprite and metadata outputs.

## Required fields

| Field | Type | Purpose |
|---|---|---|
| `id` | string | Stable asset id. |
| `source_vox` | path | Source `.vox` file path. |
| `display_name` | string | User-facing name. |
| `category` | enum | prop, tree, structure, tool, decor, terrain_feature, creature. |
| `bake_profile` | string | Bake rules profile. |
| `required_facings` | enum/list | `four_way` initially: north/east/south/west. |
| `origin` | object | Anchor/origin point. |
| `footprint` | object | Placement footprint in cells. |
| `collision_mask` | optional path/object | Blocking mask. |
| `interaction_points` | list | Optional action points. |
| `seasonal_overrides` | object | Optional season-specific source/bake overrides. |
| `damage_variants` | list | Optional damage/destruction variants. |
| `baked_outputs` | object | Generated sprite/mask paths. |
| `help_doc_id` | string | Editor wiki link. |

## Minimum baked outputs

For `four_way` assets:

```text
north.png
east.png
south.png
west.png
shadow_north.png
shadow_east.png
shadow_south.png
shadow_west.png
footprint.mask.ron
collision.mask.ron
thumbnail.png
```

## Validation rules

- Source `.vox` exists and parses.
- Required facing outputs exist after bake.
- Footprint exists for placeable assets.
- Collision mask exists if blocking.
- Origin is inside output bounds.
- The editor can show missing bake warnings.
