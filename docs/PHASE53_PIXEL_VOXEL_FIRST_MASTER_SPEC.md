# Phase 53 - Pixel-Voxel First Master Spec

Starlight Ridge is now a **pixel-voxel-first** project. Characters, NPCs, tools, buildings, props, foliage, terrain forms, furniture, machines, pickups, and world items should be authored from voxels as the source of truth.

The visual target is **micro-voxel / pixel-voxel detail**, not chunky block art. Runtime rendering may still use optimized meshes, baked sprites, impostors, or 2.5D presentation, but source assets are voxel-authored.

## Phase 53 lock-ins

- Characters and NPCs use the highest practical voxel density for readable farm/life-sim characters.
- Tools use high-detail voxel templates because they need readable silhouettes during tool-use animation.
- Base character templates must have no hair and no facial hair.
- Hair, beards, mustaches, hats, helmets, outfits, backpacks, and accessories become separate swappable voxel parts later.
- `.vox` is a first-class asset source format.
- All voxel editor tabs must be child panels only and must not render the full editor shell inside themselves.

## Density profiles

| Profile | Use | Default budget |
| --- | --- | --- |
| PixelVoxel16 | tiny/background/simple props | 4k voxels |
| PixelVoxel32 | general props/world objects | 16k voxels |
| PixelVoxel64 | buildings, terrain features, important props | 64k voxels |
| HeroDetail | characters, NPCs, animals, tools, interactables | 32k-65k voxels |

## Immediate content added

This patch adds starter `.vox` templates:

- `content/voxels/characters/base_templates/character_base_template_adult_a_bald_clean.vox`
- `content/voxels/characters/base_templates/character_base_template_adult_b_bald_clean.vox`
- `content/voxels/tools/base_templates/tool_hoe_high_detail_template.vox`
- `content/voxels/tools/base_templates/tool_axe_high_detail_template.vox`
- `content/voxels/tools/base_templates/tool_pickaxe_high_detail_template.vox`
- `content/voxels/tools/base_templates/tool_watering_can_high_detail_template.vox`
- `content/voxels/tools/base_templates/tool_sword_high_detail_template.vox`

These are base starter assets, not final art. They exist to lock the pipeline shape, density scale, pivot expectations, and asset registry structure.
