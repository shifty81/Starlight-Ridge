# Phase 53f Handoff — Voxel Rig Preview + Tool Attachment Preview

Phase 53f should use the Phase 53e character templates as preview targets.

## Focus

- Load high-density character base `.vox`.
- Display humanoid voxel rig overlay.
- Display skeleton/bone hierarchy.
- Display attachment anchors.
- Preview right-hand and left-hand tool grips.
- Preview simple neutral pose and tool pose.
- Validate base template has no hair/facial hair.
- Validate required markers/anchors.

## Do not do yet

- Full runtime bone animation.
- Full spring-bone simulation.
- Final character art.
- Final gameplay animation set.

## Target editor location

```txt
Assets → Voxel Generator → Rig Preview
```

or:

```txt
Assets → Voxel Rig Preview
```

## Regression guard

The rig preview tab must render as a child panel only and must never call the full editor shell render function.
