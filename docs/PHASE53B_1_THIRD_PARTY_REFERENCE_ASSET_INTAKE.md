# Phase 53b.1 — Third-Party Reference Asset Intake

This patch adds a safe third-party reference intake path for the uploaded `Staxel Voxel Female` model.

## Why this exists

The project needs a way to bring in outside voxel models for:
- style comparison,
- generator calibration,
- external tool testing,
- GLB import validation,
- Blender/Blockbench/MagicaVoxel workflow testing.

This does not mean third-party assets automatically become production assets.

## Asset added

`content/third_party/sketchfab/staxel_voxel_female/`

Contains:
- `staxel_voxel_female.glb`
- `staxel-voxel-female.zip`
- `ATTRIBUTION.md`
- `README.md`

## Generator policy

The Phase 53b generator may use this asset as a reference target for proportions and silhouette comparison, but Starlight Ridge character bases should remain original generated/refined project assets.

## Phase 53c editor implication

The future egui Voxel Generator tab should include:

- Reference Assets panel
- Open GLB in Blender
- Open archive folder
- Validate attribution
- Compare generated template dimensions against reference
- Promote to production reference only after explicit approval

## Attribution

See:

`content/third_party/sketchfab/staxel_voxel_female/ATTRIBUTION.md`
