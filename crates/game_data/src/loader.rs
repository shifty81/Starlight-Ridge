use crate::defs::{
    BiomePackDef, CropDef, DialogueDef, EditorAtlasPipelineCatalogDef, EditorAnimationPipelineDef,
    EditorExportValidationPipelineDef, GeneratedSceneDraftDef, HeightMapDef, HybridWorldEditorPipelineDef, ItemDef, LightingProfileDef, MapLayersDef, NpcDef,
    PresentationDef, ProtectedLayerPolicyDef, QuestDef, Scene3DDef, SceneBakeContractDef, ScheduleDef, ShopDef,
    SpriteSheetDef, TerrainCatalogDef, TerrainRulesetDef, TilesetDef, TransitionSetDef,
    WorldManifestDef, WorldgenEditorWorkflowDef,
};
use anyhow::Context;
use game_world::{MapMetadata, PropPlacement, SpawnPoint, TriggerZone};
use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn load_ron_file<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("failed to read content file {}", path.display()))?;
    ron::from_str(&source)
        .with_context(|| format!("failed to parse RON content {}", path.display()))
}

pub fn load_item_def(path: &Path) -> anyhow::Result<ItemDef> { load_ron_file(path) }
pub fn load_crop_def(path: &Path) -> anyhow::Result<CropDef> { load_ron_file(path) }
pub fn load_npc_def(path: &Path) -> anyhow::Result<NpcDef> { load_ron_file(path) }
pub fn load_schedule_def(path: &Path) -> anyhow::Result<ScheduleDef> { load_ron_file(path) }
pub fn load_dialogue_def(path: &Path) -> anyhow::Result<DialogueDef> { load_ron_file(path) }
pub fn load_quest_def(path: &Path) -> anyhow::Result<QuestDef> { load_ron_file(path) }
pub fn load_shop_def(path: &Path) -> anyhow::Result<ShopDef> { load_ron_file(path) }
pub fn load_map_metadata(path: &Path) -> anyhow::Result<MapMetadata> { load_ron_file(path) }
pub fn load_prop_list(path: &Path) -> anyhow::Result<Vec<PropPlacement>> { load_ron_file(path) }
pub fn load_spawn_list(path: &Path) -> anyhow::Result<Vec<SpawnPoint>> { load_ron_file(path) }
pub fn load_trigger_list(path: &Path) -> anyhow::Result<Vec<TriggerZone>> { load_ron_file(path) }
pub fn load_tileset_def(path: &Path) -> anyhow::Result<TilesetDef> { load_ron_file(path) }
pub fn load_sprite_sheet_def(path: &Path) -> anyhow::Result<SpriteSheetDef> { load_ron_file(path) }
pub fn load_map_layers(path: &Path) -> anyhow::Result<MapLayersDef> { load_ron_file(path) }

pub fn save_ron_file<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create content directory {}", parent.display()))?;
    }

    let pretty = ron::ser::PrettyConfig::new();
    let body = ron::ser::to_string_pretty(value, pretty)
        .with_context(|| format!("failed to serialize RON content {}", path.display()))?;
    fs::write(path, body)
        .with_context(|| format!("failed to write content file {}", path.display()))
}

pub fn save_map_layers_with_backup(path: &Path, layers: &MapLayersDef) -> anyhow::Result<Option<PathBuf>> {
    let backup_path = if path.exists() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let backup_path = path.with_file_name(format!(
            "{}.phase51f.{}.bak.ron",
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("layers"),
            timestamp
        ));
        fs::copy(path, &backup_path)
            .with_context(|| format!("failed to create map layer backup {}", backup_path.display()))?;
        Some(backup_path)
    } else {
        None
    };

    save_ron_file(path, layers)?;
    Ok(backup_path)
}

pub fn load_terrain_catalog(path: &Path) -> anyhow::Result<TerrainCatalogDef> { load_ron_file(path) }
pub fn load_biome_pack_def(path: &Path) -> anyhow::Result<BiomePackDef> { load_ron_file(path) }
pub fn load_transition_set_def(path: &Path) -> anyhow::Result<TransitionSetDef> { load_ron_file(path) }
pub fn load_terrain_ruleset_def(path: &Path) -> anyhow::Result<TerrainRulesetDef> { load_ron_file(path) }
pub fn load_editor_atlas_pipeline_catalog(path: &Path) -> anyhow::Result<EditorAtlasPipelineCatalogDef> { load_ron_file(path) }
pub fn load_editor_export_validation_pipeline(path: &Path) -> anyhow::Result<EditorExportValidationPipelineDef> { load_ron_file(path) }
pub fn load_editor_animation_pipeline(path: &Path) -> anyhow::Result<EditorAnimationPipelineDef> { load_ron_file(path) }
pub fn load_world_manifest(path: &Path) -> anyhow::Result<WorldManifestDef> { load_ron_file(path) }
pub fn load_worldgen_editor_workflow(path: &Path) -> anyhow::Result<WorldgenEditorWorkflowDef> { load_ron_file(path) }
pub fn load_protected_layer_policy(path: &Path) -> anyhow::Result<ProtectedLayerPolicyDef> { load_ron_file(path) }
pub fn load_generated_scene_draft(path: &Path) -> anyhow::Result<GeneratedSceneDraftDef> { load_ron_file(path) }
pub fn load_scene_bake_contract(path: &Path) -> anyhow::Result<SceneBakeContractDef> { load_ron_file(path) }

pub fn ron_files_in(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = fs::read_dir(dir)
        .with_context(|| format!("failed to read content directory {}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("ron"))
        .collect::<Vec<_>>();

    files.sort();
    Ok(files)
}

pub fn load_height_map(path: &Path) -> anyhow::Result<HeightMapDef> { load_ron_file(path) }
pub fn load_scene3d(path: &Path) -> anyhow::Result<Scene3DDef> { load_ron_file(path) }
pub fn load_presentation(path: &Path) -> anyhow::Result<PresentationDef> { load_ron_file(path) }
pub fn load_lighting_profile(path: &Path) -> anyhow::Result<LightingProfileDef> { load_ron_file(path) }
pub fn load_hybrid_world_editor_pipeline(path: &Path) -> anyhow::Result<HybridWorldEditorPipelineDef> { load_ron_file(path) }
