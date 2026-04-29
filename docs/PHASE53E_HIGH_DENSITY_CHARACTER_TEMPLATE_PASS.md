# Phase 53e — First Real High-Density Character Template Pass

## Purpose

Phase 53e replaces the rough proof-of-concept character bases with a stronger first-pass high-density pixel-voxel character template set.

This is still not final art. It is a better foundation for the generator, rig preview, tool attachments, and future hand-refinement in MagicaVoxel / Blockbench / Blender.

## Files added

```txt
content/voxels/characters/templates/character_base_body_a_neutral_bald_clean.vox
content/voxels/characters/templates/character_base_body_b_neutral_bald_clean.vox
content/voxels/characters/templates/character_base_player_neutral_bald_clean.vox
content/voxels/characters/templates/character_base_npc_average_neutral_bald_clean.vox
```

## Generated template stats

| File | Dimensions | Voxels |
|---|---:|---:|
| `character_base_body_a_neutral_bald_clean.vox` | 56 × 36 × 96 | 28,314 |
| `character_base_body_b_neutral_bald_clean.vox` | 62 × 38 × 104 | 35,588 |
| `character_base_player_neutral_bald_clean.vox` | 60 × 38 × 104 | 32,325 |
| `character_base_npc_average_neutral_bald_clean.vox` | 56 × 36 × 96 | 30,151 |

## Character rules

- No hair.
- No facial hair.
- No baked identity traits.
- Neutral base face only.
- Neutral body only.
- Feet-center pivot.
- Rig-ready proportions.
- Tool attachment hand markers.
- Modular overlay anchors for hair, hats, face overlays, front accessories, and back items.

## Marker colors

The templates include visible guide markers so editor and rigging work can line up consistently.

```txt
Green  = feet-center pivot
Yellow = hand/tool grip markers
Blue   = front/back accessory anchors
Magenta = hair/hat/face overlay anchors
Muted blue/gray = anatomical guide landmarks
```

These markers are part of the template authoring workflow and can be hidden/stripped later in bake profiles.

## Quality target

This pass improves:

- front/side/back silhouette,
- head and jaw form,
- neutral face plane,
- shoulders,
- torso/waist/hip rhythm,
- arms,
- elbows,
- hands,
- legs,
- knees,
- feet,
- rig/attachment readiness.

## Not included

This phase intentionally does not add:

- final clothing,
- hair,
- facial hair,
- final animation,
- runtime voxelizer implementation,
- spring-bone simulation,
- gameplay character controller changes.

## Next phase

Phase 53f should add:

```txt
Voxel Rig Preview + Tool Attachment Preview
```

That phase should load these templates, display bones/attachments, and preview tool grip positions.
