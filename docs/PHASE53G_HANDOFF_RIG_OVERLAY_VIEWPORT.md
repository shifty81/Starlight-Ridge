# Phase 53g Handoff — Rig Overlay Viewport

Phase 53g should build on Phase 53f and add the real viewport overlay.

## Focus

- Load selected base `.vox`.
- Draw simple 3D preview bounds or projected preview.
- Draw skeleton/bone line overlay.
- Draw attachment markers.
- Draw selected tool asset at right-hand/left-hand anchor.
- Preview neutral pose and first tool pose.
- Keep all rendering inside the child tab only.

## Do not do yet

- full animation playback,
- runtime skeletal voxel characters,
- spring-bone simulation,
- final art polish.

## Regression guard

No nested editor shell. Exactly one top bar, side panel set, central content area, and bottom/status stack per frame.
