# Character Modular Slot Standard

## Purpose

Character identity, outfit, and accessories should be modular. Base bodies stay neutral.

## Required slots

```txt
body_base
head_base
hair
facial_hair
hat
face_overlay
shirt
jacket
pants
boots
gloves
right_hand_tool
left_hand_tool
two_hand_tool
back_item
front_accessory
side_accessory
```

## Base template restrictions

The following slots must be empty on the base template:

```txt
hair
facial_hair
hat
shirt
jacket
pants
boots
gloves
right_hand_tool
left_hand_tool
two_hand_tool
back_item
front_accessory
side_accessory
```

The base body may contain template/material guide colors and neutral anatomical guide markers only.

## Attachment anchors

Required future attachment points:

```txt
feet_center_pivot
right_hand_grip
left_hand_grip
two_hand_tool_anchor
head_hat_anchor
hair_anchor
face_overlay_anchor
back_item_anchor
front_accessory_anchor
```

## Runtime/bake behavior

Bake profiles should be able to:

- hide guide markers,
- bake base + outfit + hair into directional sprites,
- keep runtime voxel parts separate for future real-time voxel rigging,
- validate missing required attachment points.

