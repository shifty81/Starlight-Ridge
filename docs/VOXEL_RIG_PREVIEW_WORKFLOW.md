# Voxel Rig Preview Workflow

## Workflow

```txt
Select rig profile
→ verify base `.vox`
→ refresh discovered tool `.vox` assets
→ select a tool
→ open base/tool in MagicaVoxel if needed
→ validate anchors
→ run placeholder command
→ later render full 3D overlay
```

## Required anchors

```txt
feet_center_pivot
right_hand_grip
left_hand_grip
two_hand_tool_anchor
hair_anchor
hat_anchor
face_overlay_anchor
front_accessory_anchor
back_item_anchor
```

## Tool compatibility checks

Initial Phase 53f checks are metadata/path/list based. The next viewport phase should add visual checks:

- right-hand alignment,
- left-hand alignment,
- two-hand grip pose,
- shaft/blade orientation,
- pivot sanity,
- tool scale sanity,
- sprite bake readiness.

## External tool bridge

Use MagicaVoxel for `.vox` refinement. Use Blender for future bake/render workflows. Use Blockbench later for pose/rig exploration where useful.
