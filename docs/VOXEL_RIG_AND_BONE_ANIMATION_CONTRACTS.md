# Voxel Rig and Bone Animation Contracts

## Purpose

Voxel characters should support both:

1. Modular voxel parts attached to bones.
2. Pose/animation baking into sprite sheets, voxel pose frames, or preview caches.

The initial runtime can remain baked/2.5D-friendly, while the editor supports rig preview and future real-time voxel rigging.

## VoxelRigDef

```rust
pub struct VoxelRigDef {
    pub id: String,
    pub display_name: String,
    pub skeleton: VoxelSkeletonDef,
    pub parts: Vec<VoxelRigPartDef>,
    pub attachment_points: Vec<VoxelAttachmentPointDef>,
    pub animation_profiles: Vec<String>,
    pub bake_profiles: Vec<String>,
}
```

## VoxelSkeletonDef

```rust
pub struct VoxelSkeletonDef {
    pub id: String,
    pub root_bone: String,
    pub bones: Vec<VoxelBoneDef>,
    pub units_per_voxel: f32,
    pub default_pose: String,
}
```

## VoxelBoneDef

```rust
pub struct VoxelBoneDef {
    pub id: String,
    pub parent: Option<String>,
    pub local_position: [f32; 3],
    pub local_rotation_degrees: [f32; 3],
    pub length: f32,
    pub mirror_of: Option<String>,
    pub tags: Vec<String>,
}
```

## Recommended humanoid bone IDs

```txt
root
hips
spine
chest
neck
head

shoulder_l
upper_arm_l
forearm_l
hand_l

shoulder_r
upper_arm_r
forearm_r
hand_r

upper_leg_l
lower_leg_l
foot_l
toe_l

upper_leg_r
lower_leg_r
foot_r
toe_r
```

## VoxelRigPartDef

```rust
pub struct VoxelRigPartDef {
    pub id: String,
    pub asset_id: String,
    pub bound_bone: String,
    pub pivot: [f32; 3],
    pub local_offset: [f32; 3],
    pub local_rotation_degrees: [f32; 3],
    pub slot: VoxelRigSlot,
    pub replaceable: bool,
}
```

## VoxelRigSlot

```rust
pub enum VoxelRigSlot {
    BodyBase,
    Head,
    Torso,
    ArmUpperLeft,
    ArmLowerLeft,
    HandLeft,
    ArmUpperRight,
    ArmLowerRight,
    HandRight,
    LegUpperLeft,
    LegLowerLeft,
    FootLeft,
    LegUpperRight,
    LegLowerRight,
    FootRight,
    Hair,
    FacialHair,
    Hat,
    Shirt,
    Jacket,
    Pants,
    Boots,
    Gloves,
    ToolRightHand,
    ToolLeftHand,
    BackItem,
    Accessory,
}
```

## Base-template restriction

Base templates may use only neutral body slots. Hair/facial hair slots must remain empty by default.

## Attachment points

```rust
pub struct VoxelAttachmentPointDef {
    pub id: String,
    pub bone_id: String,
    pub local_position: [f32; 3],
    pub local_rotation_degrees: [f32; 3],
    pub tags: Vec<String>,
}
```

Required attachment points:

```txt
right_hand_grip
left_hand_grip
two_hand_tool_anchor
back_item_anchor
head_hat_anchor
hair_anchor
face_overlay_anchor
front_accessory_anchor
```

## Animation clips

```rust
pub struct VoxelAnimationClipDef {
    pub id: String,
    pub display_name: String,
    pub rig_id: String,
    pub duration_seconds: f32,
    pub looped: bool,
    pub keyframes: Vec<VoxelAnimationKeyframeDef>,
}
```

```rust
pub struct VoxelAnimationKeyframeDef {
    pub time_seconds: f32,
    pub bone_transforms: Vec<VoxelBoneTransformDef>,
}
```

```rust
pub struct VoxelBoneTransformDef {
    pub bone_id: String,
    pub translation: [f32; 3],
    pub rotation_degrees: [f32; 3],
    pub scale: [f32; 3],
}
```

## Initial animation set

```txt
idle_8dir
walk_8dir
run_8dir
hoe_use_8dir
axe_use_8dir
pickaxe_use_8dir
watering_can_use_8dir
sword_slash_8dir
carry_item_8dir
hurt
```

## Recommended implementation order

1. Static rig preview.
2. Tool attachment preview.
3. Keyframe pose preview.
4. 8-direction bake preview.
5. Runtime baked animation playback.
6. Optional real-time voxel-bone runtime.

