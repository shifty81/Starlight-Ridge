# Phase 45 — Character Studio Real Assets + Offsets

Phase 45 moves Character Studio from a mannequin scaffold into a real asset contract.

## Added art

- `assets/textures/characters/mannequin_male_phase45.png`
- `assets/textures/characters/mannequin_female_phase45.png`
- `assets/textures/characters/overlay_hair_back_phase45.png`
- `assets/textures/characters/overlay_hair_front_phase45.png`
- `assets/textures/characters/overlay_beard_phase45.png`
- `assets/textures/characters/overlay_hat_phase45.png`
- `assets/textures/characters/overlay_shirt_phase45.png`
- `assets/textures/characters/overlay_pants_phase45.png`
- `assets/textures/characters/overlay_boots_phase45.png`
- `assets/textures/characters/overlay_held_tool_phase45.png`
- `assets/textures/characters/overlay_backpack_rig_phase45.png`

## Scale policy

Characters remain tile-relative:

| Class | World height | Frame target | Use |
|---|---:|---:|---|
| small_child | 1 block | 32x32 | small kids / tiny NPCs |
| tall_child_teen | 1.5 blocks | 32x48 | older kids and short villagers |
| standard_adult | 2 blocks | 32x64 | player and most adults |
| tall_adult_rig | 2.25 blocks | 48x72 | tall adults, backpacks, future rig parts |

The editor uses a larger 48x72 authoring frame so all scale classes can share anchor, offset, and socket logic.

## Contracts

- `content/editor_character/character_studio_phase45_assets.ron`
- `content/editor_character/paperdoll_offsets_phase45.ron`
- `content/editor_character/frame_anchors_phase45.ron`
- `content/editor_character/tool_sockets_phase45.ron`
- `content/editor/character_studio_workflow_phase45.ron`

## Rules

- Art height is not collision height.
- Feet Y is the sort key.
- Foot anchor is bottom-center.
- Base, clothing, hair, beard, hats, held tools, and rig parts stay as separate overlay layers.
- Tool behavior should bind through sockets and future logic graphs, not hardcoded sprite offsets.
