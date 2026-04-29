# Dynamic Voxelizer Performance Limits

## Principle

Pixel-voxel detail is the source-of-truth art style, but runtime must be budgeted carefully.

High-density voxel assets should be authored and previewed in the editor. Runtime should use optimized meshes, baked sprites, impostors, or limited-resolution VFX where appropriate.

## Offline budgets

Offline voxelizer can afford higher resolutions because it runs during authoring.

Suggested limits:

```txt
Character template bake:
  target: 64 x 40 x 112
  max:    96 x 64 x 144

Hero character bake:
  target: 72 x 48 x 128
  max:    112 x 72 x 160

Tool bake:
  target: 96 x 32 x 32
  max:    128 x 48 x 48

Prop bake:
  target: 64 x 64 x 64
  max:    128 x 128 x 128

Building blockout:
  target: chunked
  max per chunk: 64 x 64 x 64 initially
```

## Runtime VFX budgets

Runtime voxelizer should use much smaller grids.

Suggested initial limits:

```txt
Small impact burst:
  8 x 8 x 8 to 16 x 16 x 16

Medium object dissolve:
  16 x 16 x 16 to 32 x 32 x 32

Large object effect:
  32 x 32 x 32 max unless explicitly approved

Character shell VFX:
  24 x 24 x 48 max for prototype
```

## Simultaneous runtime effect limits

Initial target:

```txt
Low-end target:
  8 small active effects
  2 medium active effects
  0 large active effects unless scripted

Mid target:
  16 small active effects
  4 medium active effects
  1 large active effect

Editor preview:
  configurable, but warning above safe budgets
```

## Required optimization path

- Greedy meshing for static voxel preview.
- Instanced cube rendering for quick editor preview.
- Particle/point rendering for runtime VFX.
- Baked directional sprites for many character animations initially.
- LOD/impostor fallback for distant objects.
- Cache offline voxelizer output.
- Never resample high-resolution mesh voxelization every frame unless it is a deliberate editor-only debug mode.

## Validation warnings

The editor should warn when:

- voxel grid exceeds profile max,
- runtime VFX profile exceeds safe budget,
- too many simultaneous active effects are requested,
- spring-bone chains are too long,
- character rig has excessive modular parts,
- offline bake output would exceed configured cache limits.

