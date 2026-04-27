# Editor Layer Model

## Required panels

The editor should expose:

- Layer Stack panel
- Cell Inspector panel
- Material/Biome inspector
- Worldgen preview panel
- Simulation debug overlays
- VOX asset browser
- Validation panel
- Help/wiki panel

## Layer Stack panel

For each visible layer:

- visibility toggle
- lock toggle
- opacity slider
- validation badge
- quick help button
- selected layer highlight

## Cell Inspector panel

When clicking a cell, show the full stack:

1. Base Terrain
2. Transition
3. Liquid
4. Ground State
5. Ground Cover
6. Props/Vegetation
7. Structures
8. Collision/Interaction
9. Spawns/Regions
10. Lighting/Ambience
11. Derived simulation values

Each field should show:

- value;
- short tooltip;
- `?` full help button;
- validation state;
- allowed values dropdown where possible.

## Tool categories

### Terrain tools

- paint material
- erase overlay
- fill material
- replace material
- transition inspect
- autotile refresh

### Simulation tools

- paint liquid source
- set liquid depth
- paint wetness
- set snow depth
- freeze/thaw debug
- drain/fill basin test

### Object tools

- place prop
- place structure
- rotate object
- edit footprint
- edit collision
- edit interaction points

### Worldgen tools

- generate draft
- bake draft
- rerun pass
- lock/protect region
- show debug overlay
- compare seeds

### Pixel/atlas tools

- pixel editor
- atlas picker
- tile metadata
- clipboard/mirror/fill tools
- variant preview

## Mobile LAN editor requirements

The mobile editor should not expose every panel at once. Use modes:

- map-first mode;
- tile picker mode;
- pixel editor mode;
- object placement mode;
- inspector mode;
- help mode.

Bottom command bar should provide:

- paint
- erase
- fill
- pick
- undo
- redo
- save
- tools drawer

## Validation integration

Every panel should show validation status locally and link to the global validation panel.
