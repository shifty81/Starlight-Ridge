# Phase 44 — Character Studio Mannequin + Paperdoll Overlay Scaffold

Phase 44 turns the Character placeholder into a real native egui workspace scaffold.

## Implemented in `editor.exe`

The Character workspace now has first-pass subtabs:

- Mannequin
- Paperdoll Layers
- Equipment / Overlays
- Animation Preview
- Scale Validator
- Export / Metadata

The left editor panel now exposes Character Studio controls when the Character workspace is active:

- character scale selector
- layer visibility toggles
- shared pixel editor tool stack
- brush drawer
- RGBA / display controls

## Character scale model

Characters should be authored relative to the world tile/block size instead of being freely stretched at runtime.

| Class | World height | Frame target | Use |
|---|---:|---:|---|
| Small child | 1 block | 32x32 | young kids / tiny NPCs |
| Tall child / teen | 1.5 blocks | 32x48 | older kids, teens, short villagers |
| Standard adult | 2 blocks | 32x64 | player and most adult NPCs |
| Tall adult / rig | 2.25 blocks | 48x72 | large NPCs, backpacks, future rig/exosuit silhouettes |

The runtime rule should remain:

```text
art height != collision height
```

Sprite sorting should use the bottom-center foot anchor / feet Y position.

## Paperdoll model

The scaffold uses this intended layer stack:

```text
shadow
base_body
under_clothes
shirt
pants
boots
hair_back
face_detail
beard
hair_front
hat
held_tool
backpack_or_rig
```

Equipment should be authored as overlay sets, not baked into the base body.

## Slots

Initial slot targets:

- head
- face
- torso
- legs
- feet
- hands
- tool_hand
- back
- accessory

## Contracts added

```text
content/editor_character/character_studio_phase44.ron
content/editor/character_studio_workflow_phase44.ron
```

The character studio contract is intentionally kept out of `content/metadata` because that folder is currently used by the runtime sprite-sheet registry.

## Next phase

Recommended Phase 45:

```text
Starlight_Ridge_phase45_character_studio_real_assets_and_offsets.zip
```

Focus:

- generate or import clean mannequin sheets
- define actual male/female base assets
- add real overlay sheet references
- save per-slot offsets
- add per-frame foot anchors and tool sockets
- validate overlay frame parity
