# Starlight Ridge Phase 27 — Character Lab Foundation

## Goal

The current character workflow should be reset around a proper layered mannequin pipeline.

## Added base assets

- `assets/textures/characters/base_male_underwear_phase27.png`
- `assets/textures/characters/base_female_underwear_phase27.png`
- `content/metadata/character_bases_phase27.ron`

These are not final character art. They are cleaner project-style mannequin bases derived from the current player sheet proportions so Character Studio can begin using the right scale and frame layout.

## Non-destructive source layers

Character Studio should treat these as editable layers:

- Body
- Underwear
- Hair
- Eyes
- Face
- Shirt
- Pants
- Shoes
- Accessory
- Tool
- Equipment / R.I.G.

The editor should never permanently bake clothing into the body source unless exporting a runtime preview sheet.

## Character workflow

1. Choose base male or base female.
2. Draw clothing/hair/equipment over the mannequin.
3. Preview all directions and animation frames.
4. Export composed preview sheet.
5. Keep original layer files editable.

## Phase 30 target

The full native Character Lab should add:

- layer stack UI
- skin palette presets
- hair/eyes/clothing slots
- mirror frame helpers
- onion skin
- walk/idle preview
- export composed runtime sheet
