use serde::{Deserialize, Serialize};

/// A single named bone in a voxel character rig.
///
/// Bones form a tree: each bone references its parent by name (empty string for the root).
/// The `head` and `tail` are positions in the character's local voxel space; Z is up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxRigBone {
    /// Unique identifier within the rig (e.g. `"spine"`, `"upper_arm_l"`).
    pub id: String,
    /// Display label for the editor.
    pub label: String,
    /// Parent bone id.  Empty string = root bone.
    pub parent_id: String,
    /// Bone origin (start point) in local voxel integer coordinates.
    pub head: [i32; 3],
    /// Bone tip (end point) in local voxel integer coordinates.
    pub tail: [i32; 3],
    /// Optional attachment anchor used for held items, equipment, or IK targets.
    pub attachment_anchor: Option<[i32; 3]>,
}

/// A full rig definition shared by all character templates of the same base body type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxRigDef {
    /// Unique rig id (e.g. `"character_base_adult_a"`).
    pub id: String,
    /// Human-readable name.
    pub display_name: String,
    /// Bones in depth-first order (parents before children).
    pub bones: Vec<VoxRigBone>,
}

/// A simple key-frame pose: a per-bone rotation offset (Euler degrees, XYZ order).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxRigBonePose {
    /// The bone this pose entry applies to.
    pub bone_id: String,
    /// Rotation offset in degrees (Euler XYZ).
    pub rotation_degrees: [f32; 3],
    /// Optional translation offset in voxel units.
    pub translation: [f32; 3],
}

/// A named pose combining bone overrides for animation or editor preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxRigPose {
    /// Unique pose id (e.g. `"idle"`, `"walk_0"`).
    pub id: String,
    /// Human-readable name.
    pub label: String,
    /// Per-bone overrides.  Bones not listed use their default (bind) pose.
    pub bone_poses: Vec<VoxRigBonePose>,
}

impl VoxRigDef {
    /// Returns the placeholder rig for a standard adult character body.
    /// This defines the canonical attach points and skeleton structure used
    /// by all character base templates.
    pub fn placeholder_adult() -> Self {
        Self {
            id: "character_base_adult_placeholder".to_string(),
            display_name: "Adult Character (placeholder rig)".to_string(),
            bones: vec![
                VoxRigBone {
                    id: "root".to_string(),
                    label: "Root".to_string(),
                    parent_id: String::new(),
                    head: [28, 18, 0],
                    tail: [28, 18, 8],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "pelvis".to_string(),
                    label: "Pelvis".to_string(),
                    parent_id: "root".to_string(),
                    head: [28, 18, 8],
                    tail: [28, 18, 24],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "spine".to_string(),
                    label: "Spine".to_string(),
                    parent_id: "pelvis".to_string(),
                    head: [28, 18, 24],
                    tail: [28, 18, 56],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "neck".to_string(),
                    label: "Neck".to_string(),
                    parent_id: "spine".to_string(),
                    head: [28, 18, 56],
                    tail: [28, 18, 68],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "head".to_string(),
                    label: "Head".to_string(),
                    parent_id: "neck".to_string(),
                    head: [28, 18, 68],
                    tail: [28, 18, 92],
                    attachment_anchor: Some([28, 12, 86]),
                },
                VoxRigBone {
                    id: "upper_arm_l".to_string(),
                    label: "Upper Arm (L)".to_string(),
                    parent_id: "spine".to_string(),
                    head: [14, 18, 54],
                    tail: [6, 18, 38],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "lower_arm_l".to_string(),
                    label: "Lower Arm (L)".to_string(),
                    parent_id: "upper_arm_l".to_string(),
                    head: [6, 18, 38],
                    tail: [4, 18, 18],
                    attachment_anchor: Some([4, 18, 14]),
                },
                VoxRigBone {
                    id: "upper_arm_r".to_string(),
                    label: "Upper Arm (R)".to_string(),
                    parent_id: "spine".to_string(),
                    head: [42, 18, 54],
                    tail: [50, 18, 38],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "lower_arm_r".to_string(),
                    label: "Lower Arm (R)".to_string(),
                    parent_id: "upper_arm_r".to_string(),
                    head: [50, 18, 38],
                    tail: [52, 18, 18],
                    attachment_anchor: Some([52, 18, 14]),
                },
                VoxRigBone {
                    id: "thigh_l".to_string(),
                    label: "Thigh (L)".to_string(),
                    parent_id: "pelvis".to_string(),
                    head: [20, 18, 8],
                    tail: [18, 18, -24],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "shin_l".to_string(),
                    label: "Shin (L)".to_string(),
                    parent_id: "thigh_l".to_string(),
                    head: [18, 18, -24],
                    tail: [18, 18, -48],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "thigh_r".to_string(),
                    label: "Thigh (R)".to_string(),
                    parent_id: "pelvis".to_string(),
                    head: [36, 18, 8],
                    tail: [38, 18, -24],
                    attachment_anchor: None,
                },
                VoxRigBone {
                    id: "shin_r".to_string(),
                    label: "Shin (R)".to_string(),
                    parent_id: "thigh_r".to_string(),
                    head: [38, 18, -24],
                    tail: [38, 18, -48],
                    attachment_anchor: None,
                },
            ],
        }
    }
}
