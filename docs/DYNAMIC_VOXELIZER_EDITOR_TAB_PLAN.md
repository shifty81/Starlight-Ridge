# Dynamic Voxelizer Editor Tab Plan

## Proposed tab location

```txt
Assets → Voxel Generator → Dynamic Voxelizer
```

or as a subtab inside the existing Phase 53c Voxel Generator tab.

## Required subtabs

```txt
Profiles
Offline Voxelizer
Runtime VFX Preview
Rig Preview
Spring Bones
Reference Compare
External Tools
Validation
```

## Profiles panel

Shows:

- voxelizer profile ID,
- mode,
- source kind,
- output kind,
- grid size,
- max voxel budget,
- palette/material profile,
- target output path.

Actions:

- Create profile later,
- Duplicate profile later,
- Validate profile,
- Run selected profile,
- Run all safe offline profiles.

## Offline Voxelizer panel

Inputs:

- source path,
- source type,
- target output path,
- grid resolution,
- pivot/origin mode,
- palette profile,
- fill mode.

Actions:

- Generate `.vox`,
- Write bake cache,
- Open output in MagicaVoxel,
- Open source in Blender,
- Register generated asset,
- Compare silhouette.

## Runtime VFX Preview panel

Inputs:

- preview source,
- voxel grid resolution,
- particle lifetime,
- gravity,
- scatter force,
- dissolve direction,
- material override.

Actions:

- Preview impact burst,
- Preview dissolve,
- Preview harvest scatter,
- Preview teleport shell,
- Save VFX preset.

## Rig Preview panel

Shows:

- bone hierarchy,
- voxel part bindings,
- attachment points,
- selected pose,
- tool-in-hand preview,
- missing part warnings.

Actions:

- Load rig profile,
- Preview idle,
- Preview walk pose,
- Preview tool pose,
- Bake 8-direction pose set later.

## Spring Bones panel

Shows:

- spring profiles,
- chains,
- stiffness,
- damping,
- gravity,
- collision settings.

Actions:

- Preview chain,
- Reset simulation,
- Toggle runtime budget,
- Bake secondary motion later.

## Reference Compare panel

Shows third-party and internal reference assets.

Actions:

- Open reference in Blender,
- Validate attribution,
- Compare proportions,
- Compare bounding box,
- Compare generated template scale.

## External Tools panel

Buttons:

- Open source in Blender,
- Open output in MagicaVoxel,
- Open Blockbench model,
- Run Blender bake script,
- Open output folder.

## Nested editor regression guard

This tab must render as a child tab only.

It must never call the full editor shell render function.

There must be exactly one top bar, one left panel, one right panel, one central content area, and one bottom/status area per frame.

