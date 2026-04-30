use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneratorKind {
    CharacterBase,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxelGeneratorProfile {
    pub id: &'static str,
    pub generator_kind: GeneratorKind,
    pub output_path: &'static str,
    pub dimensions: [u8; 3],
    pub no_hair: bool,
    pub no_facial_hair: bool,
}

pub fn default_profiles() -> Vec<VoxelGeneratorProfile> {
    vec![
        VoxelGeneratorProfile {
            id: "character_body_a_high_density_clean",
            generator_kind: GeneratorKind::CharacterBase,
            output_path: "content/voxels/generated_templates/characters/character_base_body_a_phase53b_bald_clean.vox",
            dimensions: [56, 36, 96],
            no_hair: true,
            no_facial_hair: true,
        },
        VoxelGeneratorProfile {
            id: "character_body_b_high_density_clean",
            generator_kind: GeneratorKind::CharacterBase,
            output_path: "content/voxels/generated_templates/characters/character_base_body_b_phase53b_bald_clean.vox",
            dimensions: [62, 38, 104],
            no_hair: true,
            no_facial_hair: true,
        },
        VoxelGeneratorProfile {
            id: "tool_hoe_hero_detail",
            generator_kind: GeneratorKind::Tool,
            output_path: "content/voxels/generated_templates/tools/tool_hoe_phase53b_generated.vox",
            dimensions: [96, 26, 18],
            no_hair: false,
            no_facial_hair: false,
        },
        VoxelGeneratorProfile {
            id: "tool_pickaxe_hero_detail",
            generator_kind: GeneratorKind::Tool,
            output_path: "content/voxels/generated_templates/tools/tool_pickaxe_phase53b_generated.vox",
            dimensions: [92, 38, 22],
            no_hair: false,
            no_facial_hair: false,
        },
        VoxelGeneratorProfile {
            id: "tool_watering_can_hero_detail",
            generator_kind: GeneratorKind::Tool,
            output_path: "content/voxels/generated_templates/tools/tool_watering_can_phase53b_generated.vox",
            dimensions: [64, 44, 36],
            no_hair: false,
            no_facial_hair: false,
        },
    ]
}

/// Profiles whose output paths match entries in `content/voxel_assets/voxel_asset_registry.ron`.
/// These must be generated so that the scene voxel preview can display real geometry.
pub fn registry_profiles() -> Vec<VoxelGeneratorProfile> {
    vec![
        VoxelGeneratorProfile {
            id: "character_base_template_adult_a_bald_clean",
            generator_kind: GeneratorKind::CharacterBase,
            output_path: "content/voxels/characters/base_templates/character_base_template_adult_a_bald_clean.vox",
            dimensions: [56, 36, 96],
            no_hair: true,
            no_facial_hair: true,
        },
        VoxelGeneratorProfile {
            id: "character_base_template_adult_b_bald_clean",
            generator_kind: GeneratorKind::CharacterBase,
            output_path: "content/voxels/characters/base_templates/character_base_template_adult_b_bald_clean.vox",
            dimensions: [62, 38, 104],
            no_hair: true,
            no_facial_hair: true,
        },
        VoxelGeneratorProfile {
            id: "tool_hoe_high_detail_template",
            generator_kind: GeneratorKind::Tool,
            output_path: "content/voxels/tools/base_templates/tool_hoe_high_detail_template.vox",
            dimensions: [96, 26, 18],
            no_hair: false,
            no_facial_hair: false,
        },
        VoxelGeneratorProfile {
            id: "tool_axe_high_detail_template",
            generator_kind: GeneratorKind::Tool,
            output_path: "content/voxels/tools/base_templates/tool_axe_high_detail_template.vox",
            dimensions: [88, 30, 20],
            no_hair: false,
            no_facial_hair: false,
        },
        VoxelGeneratorProfile {
            id: "tool_pickaxe_high_detail_template",
            generator_kind: GeneratorKind::Tool,
            output_path: "content/voxels/tools/base_templates/tool_pickaxe_high_detail_template.vox",
            dimensions: [92, 38, 22],
            no_hair: false,
            no_facial_hair: false,
        },
        VoxelGeneratorProfile {
            id: "tool_watering_can_high_detail_template",
            generator_kind: GeneratorKind::Tool,
            output_path: "content/voxels/tools/base_templates/tool_watering_can_high_detail_template.vox",
            dimensions: [64, 44, 36],
            no_hair: false,
            no_facial_hair: false,
        },
        VoxelGeneratorProfile {
            id: "tool_sword_high_detail_template",
            generator_kind: GeneratorKind::Tool,
            output_path: "content/voxels/tools/base_templates/tool_sword_high_detail_template.vox",
            dimensions: [80, 20, 16],
            no_hair: false,
            no_facial_hair: false,
        },
    ]
}
