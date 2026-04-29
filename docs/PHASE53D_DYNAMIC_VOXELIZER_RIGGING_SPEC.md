# Phase 53d — Dynamic Voxelizer + Rigging Spec

## Purpose

Phase 53d defines a native Starlight Ridge dynamic voxelizer and voxel rigging architecture.

The goal is to support:

- offline mesh/GLB/skinned-pose to voxel conversion,
- runtime voxel VFX sampling,
- modular voxel character rigs,
- bone-driven voxel parts,
- animation bake workflows,
- spring-bone secondary motion,
- external tool bridges for MagicaVoxel, Blockbench, and Blender.

This phase is a contract/spec phase, not a final renderer or final character-art phase.

## Core system name

`Starlight Dynamic Voxel System`

## System modules

```txt
Dynamic Voxelizer
  Offline Voxelizer
  Runtime Voxel VFX Sampler
  Voxel Bake Cache

Voxel Rig
  Skeleton
  Bone hierarchy
  Voxel part bindings
  Attachment points
  Animation clip metadata

Voxel Motion
  Spring bones
  Secondary motion chains
  Modular overlay motion
```

## Runtime/offline separation

### Offline voxelizer

The offline voxelizer is for authoring and baking.

Input examples:

- `.glb`
- `.fbx`
- `.obj`
- Blender-exported mesh
- Blockbench-exported model
- skinned mesh sampled at a pose
- reference model

Output examples:

- `.vox`
- internal `.starlight_vox` later
- voxel bake cache
- generated thumbnails
- collision approximations
- silhouette comparison reports

Use cases:

- generator calibration,
- GLB import tests,
- rough prop conversion,
- reference silhouette comparison,
- animation pose bakes,
- baking character preview frames.

### Runtime voxelizer

The runtime voxelizer is for effects, not canonical asset generation.

Use cases:

- chopping/mining impact voxel bursts,
- object dissolve,
- teleport/magic effects,
- weather particles,
- harvest scatter,
- temporary voxel shell around skinned mesh pose,
- destruction/debris VFX.

Runtime voxelizer output is temporary unless explicitly saved through an editor bake action.

## Character-base rule

Base character templates remain:

- bald,
- clean-shaven,
- neutral,
- no baked identity traits,
- no baked hair or facial hair,
- future-ready for modular hair/beard/hat/clothing/accessory overlays.

Dynamic bones are for modular overlays such as hair, straps, tails, scarves, tool charms, and bags, not for baked-in base hair.

## Immediate implementation target after this spec

Phase 53e should rebuild high-density character base templates to a stronger pixel-voxel art standard.

Phase 53f should then add voxel rig preview.

