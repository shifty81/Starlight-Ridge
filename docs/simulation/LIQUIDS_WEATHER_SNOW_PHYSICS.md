# Liquids, Weather, Puddles, and Snow Simulation Spec

## Philosophy

Use a gameplay-focused shallow-grid simulation, not full CFD.

The simulation must be:

- deterministic enough for save/load;
- cheap enough for large maps;
- modular by material/liquid/weather definitions;
- inspectable in the editor;
- compatible with layered tile rendering.

## Core buffers

Each cell may store:

- `liquid_id`
- `liquid_depth`
- `flow_x`
- `flow_y`
- `wetness`
- `puddle_depth`
- `snow_depth`
- `ice_amount`
- `slush_amount`
- `temperature`
- `exposure`
- `drainage`

## Liquid update rule

Per simulation tick:

1. For each liquid cell, evaluate neighbors.
2. Prefer lower elevation or lower fluid surface height.
3. Transfer limited amount based on `flow_rate` and `viscosity`.
4. Apply seepage if material absorbs liquid.
5. Apply evaporation if liquid supports it.
6. Apply hazard interactions: lava heat, oil flammability, freezing water.

## Liquid profiles

### Water

- Low viscosity.
- Moderate flow.
- Can puddle and seep.
- Can freeze if cold.
- Can wet soil and fill depressions.

### Swamp water

- Slightly slower.
- Higher visual contamination.
- Biome-limited.

### Icy water

- Freezes more easily.
- Can create ice overlay.

### Lava

- Very high viscosity.
- Very slow flow.
- Heat damage.
- Ignition behavior later.
- May cool into crust/volcanic stone later.

### Crude oil

- High viscosity.
- Slow spread.
- Flammable.
- Dark stain/wetness visual.
- Should not normally appear in farm/coastal starter maps.

## Rain and puddles

Rain writes to:

- wetness
- puddle depth
- liquid depth for exposed low cells

Rain accumulation is affected by:

- material puddle capacity;
- drainage;
- slope/elevation;
- roof/canopy exposure;
- existing water sources.

## Snow accumulation

Snow writes to snow depth.

Snow accumulation is affected by:

- season;
- weather intensity;
- temperature;
- exposure;
- canopy/roof cover;
- material snow retention;
- existing liquid/wetness state.

## Melt/freeze cycle

When temperature rises:

```text
snow -> slush -> water/wetness
```

When temperature drops:

```text
wetness/puddles/shallow water -> ice/slush
```

## Editor debug views

Required overlays:

- liquid depth heatmap
- flow arrows
- wetness amount
- puddle depth
- snow depth
- ice/slush amount
- drainage map
- exposure map

## Save/load strategy

For normal maps, store only meaningful non-zero simulation cells. Derived layers can be rebuilt, but persistent gameplay states must save:

- current liquid depth where non-default;
- snow/ice state;
- wetness/puddle state;
- weather system state.
