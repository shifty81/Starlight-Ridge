pub mod defs;
pub mod loader;
pub mod registry;
pub mod validate;

use crate::defs::MapBundle;
use crate::loader::*;
use crate::registry::ContentRegistry;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub fn discover_content_root(root: impl AsRef<Path>) -> anyhow::Result<PathBuf> {
    let content_root = root.as_ref().join("content");
    anyhow::ensure!(
        content_root.exists(),
        "content folder not found at {}",
        content_root.display()
    );
    Ok(content_root)
}

fn is_tileset_sidecar_file(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };

    matches!(
        file_name,
        "base_tileset_roles.ron" | "tile_roles.ron" | "tileset_roles.ron" | "atlas_roles.ron"
    ) || file_name.ends_with("_roles.ron")
        || file_name.ends_with("_sidecar.ron")
        || file_name.contains("roles_")
}

fn looks_like_sprite_sheet_metadata_file(path: &Path) -> anyhow::Result<bool> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to inspect sprite metadata {}", path.display()))?;

    Ok(contents.contains("texture_path") && contents.contains("entries"))
}

pub fn load_registry(project_root: impl AsRef<Path>) -> anyhow::Result<ContentRegistry> {
    let content_root = discover_content_root(project_root)?;
    let mut registry = ContentRegistry {
        root: content_root.clone(),
        ..ContentRegistry::default()
    };

    for path in ron_files_in(&content_root.join("items"))? {
        let def = load_item_def(&path)?;
        registry.items.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&content_root.join("crops"))? {
        let def = load_crop_def(&path)?;
        registry.crops.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&content_root.join("npc"))? {
        let def = load_npc_def(&path)?;
        registry.npcs.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&content_root.join("schedules"))? {
        let def = load_schedule_def(&path)?;
        registry.schedules.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&content_root.join("dialogue"))? {
        let def = load_dialogue_def(&path)?;
        registry.dialogues.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&content_root.join("quests"))? {
        let def = load_quest_def(&path)?;
        registry.quests.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&content_root.join("shops"))? {
        let def = load_shop_def(&path)?;
        registry.shops.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&content_root.join("tiles"))? {
        if is_tileset_sidecar_file(&path) {
            log::debug!("skipping editor-only tileset sidecar {}", path.display());
            continue;
        }

        let def = load_tileset_def(&path)?;
        registry.tilesets.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&content_root.join("metadata"))? {
        if !looks_like_sprite_sheet_metadata_file(&path)? {
            log::debug!(
                "skipping editor-only/non-sprite metadata file {}",
                path.display()
            );
            continue;
        }

        let def = load_sprite_sheet_def(&path)?;
        registry.sprite_sheets.insert(def.id.clone(), def);
    }

    let terrain_root = content_root.join("terrain");
    let terrain_catalog_path = terrain_root.join("terrain_types.ron");
    if terrain_catalog_path.exists() {
        let catalog = load_terrain_catalog(&terrain_catalog_path)?;
        for def in catalog.terrain_types {
            registry.terrain_types.insert(def.id.clone(), def);
        }
    }
    for path in ron_files_in(&terrain_root.join("biome_packs"))? {
        let def = load_biome_pack_def(&path)?;
        registry.biome_packs.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&terrain_root.join("transition_sets"))? {
        let def = load_transition_set_def(&path)?;
        registry.transition_sets.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&terrain_root.join("rulesets"))? {
        let def = load_terrain_ruleset_def(&path)?;
        registry.terrain_rulesets.insert(def.id.clone(), def);
    }

    for path in ron_files_in(&content_root.join("editor_pipeline"))? {
        let def = load_editor_atlas_pipeline_catalog(&path)?;
        registry.editor_atlas_pipelines.insert(def.id.clone(), def);
    }

    for path in ron_files_in(&content_root.join("editor_export"))? {
        let def = load_editor_export_validation_pipeline(&path)?;
        registry.editor_export_pipelines.insert(def.id.clone(), def);
    }

    for path in ron_files_in(&content_root.join("editor_animation"))? {
        let def = load_editor_animation_pipeline(&path)?;
        registry
            .editor_animation_pipelines
            .insert(def.id.clone(), def);
    }

    // Phase 51: load the canonical world/scene/layer architecture contracts.
    // These are intentionally loaded by stable filenames/directories so older
    // phase48 exploratory files can remain in content/worldgen without breaking
    // the runtime registry.
    let worldgen_root = content_root.join("worldgen");
    let world_manifest_path = worldgen_root.join("world_manifest_phase51.ron");
    if world_manifest_path.exists() {
        let def = load_world_manifest(&world_manifest_path)?;
        registry.world_manifests.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&worldgen_root.join("protected_layer_policies"))? {
        let def = load_protected_layer_policy(&path)?;
        registry
            .protected_layer_policies
            .insert(def.id.clone(), def);
    }
    for path in ron_files_in(&worldgen_root.join("generated_drafts"))? {
        let def = load_generated_scene_draft(&path)?;
        registry.generated_scene_drafts.insert(def.id.clone(), def);
    }
    for path in ron_files_in(&worldgen_root.join("bake_contracts"))? {
        let def = load_scene_bake_contract(&path)?;
        registry.scene_bake_contracts.insert(def.id.clone(), def);
    }

    let editor_worldgen_workflow_path = content_root
        .join("editor_worldgen")
        .join("worldgen_editor_workflow_phase51.ron");
    if editor_worldgen_workflow_path.exists() {
        let def = load_worldgen_editor_workflow(&editor_worldgen_workflow_path)?;
        registry
            .worldgen_editor_workflows
            .insert(def.id.clone(), def);
    }

    let maps_root = content_root.join("maps");
    if maps_root.exists() {
        for entry in std::fs::read_dir(&maps_root)
            .with_context(|| format!("failed to read map root {}", maps_root.display()))?
        {
            let path = entry?.path();
            if !path.is_dir() {
                continue;
            }
            let metadata = load_map_metadata(&path.join("map.ron"))?;
            let map_id = metadata.id.clone();
            let bundle = MapBundle {
                metadata,
                props: load_prop_list(&path.join("props.ron"))?,
                spawns: load_spawn_list(&path.join("spawns.ron"))?,
                triggers: load_trigger_list(&path.join("triggers.ron"))?,
            };
            registry.maps.insert(map_id.clone(), bundle);

            let layers_path = path.join("layers.ron");
            if layers_path.exists() {
                let layers = load_map_layers(&layers_path)?;
                registry.map_layers.insert(map_id, layers);
            }
        }
    }

    validate::validate_registry(&registry)?;
    log::info!(
        "loaded content registry from {} -> {}",
        registry.root.display(),
        registry.summary()
    );
    Ok(registry)
}
