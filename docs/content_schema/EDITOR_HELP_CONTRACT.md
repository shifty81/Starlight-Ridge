# Editor Help and Tooltip Contract

## Purpose

Every editor-facing field should carry help metadata. This makes the editor self-documenting and enables validation messages to link directly to wiki pages.

## Field help metadata

| Field | Type | Purpose |
|---|---|---|
| `field_id` | string | Stable id, e.g. `material.puddle_capacity`. |
| `display_name` | string | Label shown in UI. |
| `tooltip_short` | string | One-sentence hover tooltip. |
| `help_doc_id` | string | Full wiki page id. |
| `validation_hint` | string | Explanation shown near validation errors. |
| `examples` | list | Optional example values. |
| `advanced` | bool | Whether to hide under advanced UI. |

## Required editor behavior

- Hover field label: show `tooltip_short`.
- Click `?`: open full wiki page.
- Validation error: show `validation_hint` and wiki link.
- Missing help metadata: validation warning.

## Help document ids

Use stable ids like:

```text
wiki.editor.layers
wiki.worldgen.biomes
wiki.materials.puddle_capacity
wiki.vox.bake_profiles
wiki.weather.snow_accumulation
```
