# Worldgen Pipeline Spec

## Design goal

The generator should produce scene-based maps that can be edited after generation. Generation creates a draft; the editor can bake that draft into normal editable scene/layer files.

## Required properties

- Deterministic by world seed + scene seed + pass seed.
- Multi-pass and inspectable.
- Each pass can generate a debug overlay.
- Each pass writes to a named layer or derived buffer.
- Authored content can override or protect generated content.
- Generator outputs must validate before bake.

## Generation pass order

### Pass 0 — Generator context

Inputs:

- world seed
- scene id
- biome preset
- scene template
- map width/height
- tile size
- season/month hint
- protected authored regions

Outputs:

- deterministic pass seeds
- generation coordinate system
- debug session metadata

### Pass 1 — Macro terrain fields

Generate continuous fields:

- elevation
- moisture
- temperature
- roughness
- fertility
- salinity
- volcanic pressure
- crude oil seep probability
- distance-to-ocean
- distance-to-river

These fields are not final tiles. They drive later decisions.

### Pass 2 — Land/water/coast classification

Classify cells as:

- ocean
- deep water
- shallow water
- beach/intertidal
- river
- pond/lake
- wetland
- dry land
- cliff/slope candidate

### Pass 3 — Hydrology routing

Use elevation and moisture to place:

- streams
- rivers
- ponds
- marsh basins
- flood basins
- drainage direction
- puddle-prone depressions

Outputs feed liquid-source layers and weather simulation layers.

### Pass 4 — Biome assignment

Use elevation, moisture, temperature, salinity, and special masks to assign one of the canonical 12 biomes.

Biome edges should be smoothed with neighbor voting and transition bands.

### Pass 5 — Material family selection

Choose the base material family per cell:

- grass family
- sand/soil family
- water family
- stone/mud/ash/snow support material
- liquid source type, if present

### Pass 6 — Autotile and transition intent

Generate semantic transition intent:

- shoreline edge
- beach-to-grass transition
- dirt-to-grass transition
- marsh edge
- snow edge
- mud edge
- path edge
- cliff edge
- water depth transition

This pass should not pick final atlas coordinates directly. It writes semantic transition records used by render resolution.

### Pass 7 — Local feature placement

Place procedural local features:

- trees
- shrubs
- reeds
- rocks
- driftwood
- flowers
- mushrooms
- resource nodes
- caves/entrance candidates
- ruins/structure candidates

Placement is biome-aware and should respect collision, slope, water, and protected zones.

### Pass 8 — Authored scene overlays

Apply authored content:

- starter farm
- roads
- town entrances
- City Hall/land plot connectors
- farm boundaries
- docks/bridges
- quest areas
- reserved expansion plots

Authored overlays can be protected from regeneration.

### Pass 9 — Simulation initialization

Initialize derived simulation buffers:

- liquid depth
- liquid velocity hint
- soil moisture
- puddle capacity
- snow depth baseline
- ice state
- wetness visual state
- temperature local modifiers

### Pass 10 — Validation and bake

Run validation before saving:

- no missing materials
- no missing biome definitions
- no missing atlas roles
- no invalid liquid/material combos
- no impassable required routes
- no generated props without sprite/vox metadata
- no editor layers with unsupported cell types

## Debug overlays required in editor

- biome map
- elevation field
- moisture field
- temperature field
- hydrology flow arrows
- material family map
- transition intent map
- feature placement map
- protected authored regions
- validation error heatmap
