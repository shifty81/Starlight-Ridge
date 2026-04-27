# Editor Wiki and Tooltip System

## Goal

The editor must be self-documenting. Every field, tool, and validation message should explain itself.

## Help system layers

### 1. Hover tooltip

A short explanation shown when hovering over a field, button, layer, or validation badge.

### 2. Context help

A `?` icon opens the relevant wiki page inside the editor.

### 3. Validation help

Validation errors include:

- what is wrong;
- why it matters;
- how to fix it;
- link to the full wiki page.

### 4. Full wiki panel

Markdown-rendered help pages inside the editor.

## Wiki page structure

Recommended location:

```text
docs/wiki/
  editor/
  worldgen/
  materials/
  simulation/
  vox/
  camera/
  troubleshooting/
```

## Required first wiki pages

- `docs/wiki/editor/overview.md`
- `docs/wiki/editor/layers.md`
- `docs/wiki/editor/mobile_lan_editor.md`
- `docs/wiki/worldgen/overview.md`
- `docs/wiki/worldgen/biomes.md`
- `docs/wiki/materials/materials.md`
- `docs/wiki/simulation/liquids_weather_snow.md`
- `docs/wiki/vox/vox_pipeline.md`
- `docs/wiki/camera/orthographic_orbit_camera.md`
- `docs/wiki/troubleshooting/common_validation_errors.md`

## Tooltip registry

The tooltip registry should be data-driven. Example field ids:

```text
biome.temperature_range
biome.moisture_range
material.puddle_capacity
material.snow_retention
liquid.viscosity
liquid.flow_rate
weather.snow_delta
vox.required_facings
map_layer.protected
worldgen.seed
```

## Missing help validation

The editor should warn if a field is exposed without help metadata.
