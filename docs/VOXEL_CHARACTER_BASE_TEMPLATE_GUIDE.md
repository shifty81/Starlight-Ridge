# Voxel Character Base Template Guide

## Base-template rule

Character base templates are body/mannequin sources only.

They must not include:

- hair
- beard
- mustache
- eyebrows as hair chunks
- facial hair
- hats
- helmets
- final outfits

They may include:

- neutral mannequin body volume
- simple skin/material tone
- minimal underlayer markers
- editor-only joint/attachment markers
- blank head shape
- hand/foot proportions

## Recommended scale

The current Phase 53 templates use high-detail pixel-voxel proportions:

- adult base height target: roughly 60-72 voxels
- default character density profile: `HeroDetail`
- default `voxels_per_tile`: `64`
- recommended maximum budget: 65,536 voxels per assembled character before optimization/bake

## Why bald/clean base templates

Hair, facial hair, hats, outfits, and accessories must become swappable voxel parts. This keeps character customization, NPC variation, seasonal outfits, and animation overrides manageable.

## Later character parts

Recommended future folders:

```txt
content/voxels/characters/parts/hair/
content/voxels/characters/parts/facial_hair/
content/voxels/characters/parts/outfits/
content/voxels/characters/parts/hats/
content/voxels/characters/parts/tools_held/
content/voxels/characters/parts/backpacks/
```
