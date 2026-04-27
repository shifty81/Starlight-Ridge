# Worldgen and Editor Layer Requirements

## Layer model summary

Use 10 editor-visible layers and 4 derived/runtime layers.

The editor-visible layers are authored, generated, or both. The derived layers are produced by simulation, worldgen, or render-resolution systems.

## Editor-visible layers

### 1. Base Terrain

Purpose: core ground material.

Examples:

- grass
- dirt
- sand
- stone
- snow base
- ash

Required fields:

- material id
- variant id
- biome id
- protected flag

### 2. Terrain Overlay / Transition

Purpose: edge and blend overlays.

Examples:

- shoreline edge
- grass/dirt blend
- path border
- snow edge
- marsh-to-water transition

Required fields:

- transition type
- source material
- target material
- resolver mask

### 3. Liquid Region

Purpose: liquid source placement and authored liquid bodies.

Examples:

- water source
- river route
- pond basin
- lava pool
- oil seep

Required fields:

- liquid id
- initial depth
- source/sink behavior
- flow lock flag

### 4. Ground State

Purpose: farm and temporary ground states.

Examples:

- tilled soil
- watered soil
- fertilized soil
- muddy soil
- frozen ground

Required fields:

- state id
- strength/depth
- decay rule

### 5. Ground Cover

Purpose: small decorative overlays.

Examples:

- flowers
- pebbles
- moss
- leaf litter
- reeds
- snow caps

Required fields:

- cover id
- density
- random seed
- seasonal visibility

### 6. Props / Vegetation

Purpose: placeable object layer.

Examples:

- trees
- bushes
- rocks
- logs
- crops
- resource nodes

Required fields:

- object id
- asset id
- rotation/facing
- variant seed
- footprint id

### 7. Structures

Purpose: constructed objects and architecture.

Examples:

- fences
- bridges
- walls
- machines
- buildings
- docks

Required fields:

- structure id
- asset id
- rotation/facing
- interaction id
- footprint id

### 8. Collision / Interaction

Purpose: explicit gameplay collision and interaction metadata.

Examples:

- blocked cell
- interact prompt
- door trigger
- harvest action
- climb/vault marker

Required fields:

- collision type
- action type
- prompt id
- linked entity id

### 9. Spawns / Regions

Purpose: gameplay zones and spawn metadata.

Examples:

- NPC spawn
- fish zone
- weather region
- encounter region
- biome boundary
- land plot ownership region

Required fields:

- region id
- region type
- ruleset id
- spawn table id

### 10. Lighting / Ambience Markers

Purpose: local lighting, ambience, fog, and visual zones.

Examples:

- light probe
- shadow mask
- fog zone
- ambient sound region
- weather exposure override

Required fields:

- ambience id
- intensity
- radius/shape
- priority

## Derived/runtime layers

### 11. Fluid Depth/Flow

Generated from Liquid Region + rain/weather + terrain slope.

Fields:

- liquid id
- depth
- flow vector
- temperature
- contamination flags

### 12. Snow / Ice Deposition

Generated from weather + temperature + exposure.

Fields:

- snow depth
- ice amount
- slush amount
- compaction

### 13. Wetness / Puddle Accumulation

Generated from rain + drainage + material absorption.

Fields:

- wetness amount
- puddle depth
- drying timer
- visual darkness amount

### 14. Autotile Resolution / Render Composite

Generated from terrain/material/transition context.

Fields:

- resolved atlas tile ids
- overlay draw list
- animation frame binding
- sorting priority

## Editor requirements

The editor must support:

- showing/hiding each visible layer;
- locking each visible layer;
- inspecting cell stack at cursor;
- showing derived layer overlays;
- running layer validation;
- baking generated layers into editable layers;
- rebuilding derived layers without destructive edits;
- assigning help doc and tooltip for every field.
