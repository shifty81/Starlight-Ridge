use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use engine_assets::vox::{VoxModel, load_vox_file};
use engine_render_gl::{
    TileMapRenderData, VoxelSceneObjectRange, VoxelSceneRenderData, VoxelVertex,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
enum SceneVoxelObjectSetFile {
    VoxelObjectSetDef(SceneVoxelObjectSetDef),
}

#[derive(Debug, Clone, Deserialize)]
struct SceneVoxelObjectSetDef {
    id: String,
    scene_id: String,
    objects: Vec<SceneVoxelObjectFile>,
}

#[derive(Debug, Clone, Deserialize)]
enum SceneVoxelObjectFile {
    VoxelSceneObjectDef(SceneVoxelObjectDef),
}

#[derive(Debug, Clone, Deserialize)]
struct SceneVoxelObjectDef {
    id: String,
    asset_id: String,
    position: [f32; 3],
    rotation_degrees: [f32; 3],
    scale: f32,
    layer: String,
    tags: Vec<String>,
    collision_enabled: bool,
    interaction_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
enum SceneVoxelAssetRegistryFile {
    VoxelAssetRegistry(SceneVoxelAssetRegistryDef),
}

#[derive(Debug, Clone, Deserialize)]
struct SceneVoxelAssetRegistryDef {
    phase: String,
    default_voxels_per_tile: u32,
    assets: Vec<SceneVoxelAssetFile>,
}

#[derive(Debug, Clone, Deserialize)]
enum SceneVoxelAssetFile {
    VoxelAssetDef(SceneVoxelAssetDef),
}

#[derive(Debug, Clone, Deserialize)]
struct SceneVoxelAssetDef {
    id: String,
    source_path: String,
    voxels_per_tile: u32,
    scale: f32,
    pivot: ScenePivotFile,
}

#[derive(Debug, Clone, Deserialize)]
enum ScenePivotFile {
    PivotDef(ScenePivotDef),
}

#[derive(Debug, Clone, Deserialize)]
struct ScenePivotDef {
    mode: ScenePivotMode,
    offset: [f32; 3],
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub(crate) enum ScenePivotMode {
    FeetCenter,
    GripPoint,
}

#[derive(Debug, Clone)]
pub(crate) struct SceneVoxelPreviewState {
    pub(crate) scene_id: String,
    pub(crate) set_id: String,
    pub(crate) phase: String,
    pub(crate) entries: Vec<SceneVoxelPreviewEntry>,
}

#[derive(Debug, Clone)]
pub(crate) struct SceneVoxelPreviewEntry {
    pub(crate) object_id: String,
    pub(crate) asset_id: String,
    pub(crate) source_path: PathBuf,
    pub(crate) relative_source_path: String,
    pub(crate) position: [f32; 3],
    pub(crate) rotation_degrees: [f32; 3],
    pub(crate) scale: f32,
    pub(crate) asset_scale: f32,
    pub(crate) voxels_per_tile: u32,
    pub(crate) layer: String,
    pub(crate) tags: Vec<String>,
    pub(crate) collision_enabled: bool,
    pub(crate) interaction_id: Option<String>,
    pub(crate) source_exists: bool,
    pub(crate) pivot_mode: ScenePivotMode,
    pub(crate) pivot_offset: [f32; 3],
}

pub(crate) fn load_scene_voxel_preview_state(
    project_root: &Path,
    map_id: &str,
) -> anyhow::Result<Option<SceneVoxelPreviewState>> {
    let object_set_path = project_root
        .join("content")
        .join("scenes")
        .join(map_id)
        .join("voxel_objects.ron");
    if !object_set_path.exists() {
        return Ok(None);
    }

    let asset_registry_path = project_root
        .join("content")
        .join("voxel_assets")
        .join("voxel_asset_registry.ron");
    if !asset_registry_path.exists() {
        return Ok(None);
    }

    let SceneVoxelObjectSetFile::VoxelObjectSetDef(object_set) =
        game_data::loader::load_ron_file::<SceneVoxelObjectSetFile>(&object_set_path)
            .with_context(|| format!("failed to load {}", object_set_path.display()))?;
    let SceneVoxelAssetRegistryFile::VoxelAssetRegistry(asset_registry) =
        game_data::loader::load_ron_file::<SceneVoxelAssetRegistryFile>(&asset_registry_path)
            .with_context(|| format!("failed to load {}", asset_registry_path.display()))?;

    let asset_lookup = asset_registry
        .assets
        .into_iter()
        .map(|asset| match asset {
            SceneVoxelAssetFile::VoxelAssetDef(asset) => (asset.id.clone(), asset),
        })
        .collect::<HashMap<_, _>>();

    let entries = object_set
        .objects
        .into_iter()
        .map(|object| match object {
            SceneVoxelObjectFile::VoxelSceneObjectDef(object) => {
                let asset = asset_lookup.get(&object.asset_id);
                let relative_source_path = asset
                    .map(|asset| asset.source_path.clone())
                    .unwrap_or_default();
                let source_path = if relative_source_path.is_empty() {
                    project_root.to_path_buf()
                } else {
                    project_root.join(&relative_source_path)
                };
                let voxels_per_tile = asset
                    .map(|asset| {
                        asset
                            .voxels_per_tile
                            .max(asset_registry.default_voxels_per_tile)
                            .max(1)
                    })
                    .unwrap_or(asset_registry.default_voxels_per_tile.max(1));
                let (pivot_mode, pivot_offset) = asset
                    .map(|asset| match &asset.pivot {
                        ScenePivotFile::PivotDef(pivot) => (pivot.mode, pivot.offset),
                    })
                    .unwrap_or((ScenePivotMode::FeetCenter, [0.0, 0.0, 0.0]));
                SceneVoxelPreviewEntry {
                    object_id: object.id,
                    asset_id: object.asset_id,
                    source_exists: !relative_source_path.is_empty() && source_path.exists(),
                    source_path,
                    relative_source_path,
                    position: object.position,
                    rotation_degrees: object.rotation_degrees,
                    scale: object.scale,
                    asset_scale: asset.map(|asset| asset.scale).unwrap_or(1.0),
                    voxels_per_tile,
                    layer: object.layer,
                    tags: object.tags,
                    collision_enabled: object.collision_enabled,
                    interaction_id: object.interaction_id,
                    pivot_mode,
                    pivot_offset,
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(Some(SceneVoxelPreviewState {
        scene_id: object_set.scene_id,
        set_id: object_set.id,
        phase: asset_registry.phase,
        entries,
    }))
}

pub(crate) fn build_scene_voxel_render_data(
    project_root: &Path,
    map_id: &str,
    tile_map: Option<&TileMapRenderData>,
) -> anyhow::Result<Option<VoxelSceneRenderData>> {
    let Some(tile_map) = tile_map else {
        return Ok(None);
    };
    let Some(scene_preview) = load_scene_voxel_preview_state(project_root, map_id)? else {
        return Ok(None);
    };

    let tile_width = tile_map.tile_width.max(1) as f32;
    let tile_height = tile_map.tile_height.max(1) as f32;
    let world_origin_x = -((tile_map.map_width.max(1) * tile_map.tile_width.max(1)) as f32) * 0.5;
    let world_origin_y = -((tile_map.map_height.max(1) * tile_map.tile_height.max(1)) as f32) * 0.5;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut object_ranges = Vec::new();
    let mut bounds_min = glam::Vec3::splat(f32::INFINITY);
    let mut bounds_max = glam::Vec3::splat(f32::NEG_INFINITY);

    for entry in &scene_preview.entries {
        if !entry.source_exists {
            continue;
        }
        let model = match load_vox_file(&entry.source_path) {
            Ok(model) => model,
            Err(error) => {
                log::warn!(
                    "scene voxel object '{}' failed to load '{}': {error:#}",
                    entry.object_id,
                    entry.source_path.display()
                );
                continue;
            }
        };

        let voxels_per_tile = entry.voxels_per_tile.max(1) as f32;
        let combined_scale = entry.asset_scale.max(0.01) * entry.scale.max(0.01);
        let voxel_scale = glam::Vec3::new(
            tile_width / voxels_per_tile,
            tile_height / voxels_per_tile,
            tile_height / voxels_per_tile,
        ) * combined_scale;
        let pivot = scene_asset_pivot(entry.pivot_mode, entry.pivot_offset, &model);
        let translation = glam::Vec3::new(
            world_origin_x + entry.position[0] * tile_width,
            world_origin_y + entry.position[1] * tile_height,
            entry.position[2] * tile_height,
        );
        let yaw_radians = entry.rotation_degrees[2].to_radians();

        let index_start = indices.len() as u32;
        let object_bounds = append_scene_vox_model_mesh(
            &mut vertices,
            &mut indices,
            &model,
            voxel_scale,
            pivot,
            translation,
            yaw_radians,
        );
        let index_count = indices.len() as u32 - index_start;
        if index_count > 0 {
            object_ranges.push(VoxelSceneObjectRange {
                object_key: format!("scene:{}", entry.object_id),
                label: entry.object_id.clone(),
                index_start,
                index_count,
                bounds_min: object_bounds.0.to_array(),
                bounds_max: object_bounds.1.to_array(),
            });
        }
        bounds_min = bounds_min.min(object_bounds.0);
        bounds_max = bounds_max.max(object_bounds.1);
    }

    if vertices.is_empty() || indices.is_empty() {
        return Ok(None);
    }

    log::info!(
        "scene voxel render contract ready: scene={} id={} phase={} objects={} vertices={} indices={}",
        scene_preview.scene_id,
        scene_preview.set_id,
        scene_preview.phase,
        scene_preview.entries.len(),
        vertices.len(),
        indices.len()
    );

    Ok(Some(VoxelSceneRenderData {
        vertices,
        indices,
        bounds_min: bounds_min.to_array(),
        bounds_max: bounds_max.to_array(),
        object_ranges,
    }))
}

fn scene_asset_pivot(
    pivot_mode: ScenePivotMode,
    pivot_offset: [f32; 3],
    model: &VoxModel,
) -> glam::Vec3 {
    let base = match pivot_mode {
        ScenePivotMode::FeetCenter => {
            glam::Vec3::new(model.width as f32 * 0.5, model.height as f32 * 0.5, 0.0)
        }
        ScenePivotMode::GripPoint => glam::Vec3::ZERO,
    };
    base + glam::Vec3::from_array(pivot_offset)
}

fn shade_color(color: [f32; 4], multiplier: f32) -> [f32; 4] {
    [
        (color[0] * multiplier).clamp(0.0, 1.0),
        (color[1] * multiplier).clamp(0.0, 1.0),
        (color[2] * multiplier).clamp(0.0, 1.0),
        color[3],
    ]
}

fn append_scene_vox_model_mesh(
    vertices: &mut Vec<VoxelVertex>,
    indices: &mut Vec<u32>,
    model: &VoxModel,
    voxel_scale: glam::Vec3,
    pivot: glam::Vec3,
    translation: glam::Vec3,
    yaw_radians: f32,
) -> (glam::Vec3, glam::Vec3) {
    let rotation = glam::Quat::from_rotation_z(yaw_radians);
    let occupied = model
        .voxels
        .iter()
        .map(|voxel| (voxel.x, voxel.y, voxel.z))
        .collect::<HashSet<_>>();
    let face_directions = [
        (
            (0_i16, 0_i16, -1_i16),
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            0.64,
        ),
        (
            (0_i16, 0_i16, 1_i16),
            [
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            1.08,
        ),
        (
            (0_i16, -1_i16, 0_i16),
            [
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
            ],
            0.82,
        ),
        (
            (1_i16, 0_i16, 0_i16),
            [
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
            ],
            0.92,
        ),
        (
            (0_i16, 1_i16, 0_i16),
            [
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            0.76,
        ),
        (
            (-1_i16, 0_i16, 0_i16),
            [
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
            ],
            0.70,
        ),
    ];

    let mut bounds_min = glam::Vec3::splat(f32::INFINITY);
    let mut bounds_max = glam::Vec3::splat(f32::NEG_INFINITY);

    for voxel in &model.voxels {
        let color_index = voxel.color_index as usize;
        let palette_color =
            model
                .palette
                .get(color_index)
                .copied()
                .unwrap_or(engine_assets::vox::VoxColor {
                    r: 180,
                    g: 60,
                    b: 200,
                    a: 255,
                });
        if palette_color.a == 0 {
            continue;
        }
        let base_color = [
            palette_color.r as f32 / 255.0,
            palette_color.g as f32 / 255.0,
            palette_color.b as f32 / 255.0,
            1.0,
        ];
        let local_min =
            (glam::Vec3::new(voxel.x as f32, voxel.y as f32, voxel.z as f32) - pivot) * voxel_scale;
        let local_max = local_min + voxel_scale;
        let min = [local_min.x, local_min.y, local_min.z];
        let max = [local_max.x, local_max.y, local_max.z];
        let visible_faces = face_directions.iter().filter(|(delta, _, _)| {
            let neighbor = (
                voxel.x as i16 + delta.0,
                voxel.y as i16 + delta.1,
                voxel.z as i16 + delta.2,
            );
            neighbor.0 < 0
                || neighbor.1 < 0
                || neighbor.2 < 0
                || !occupied.contains(&(neighbor.0 as u8, neighbor.1 as u8, neighbor.2 as u8))
        });

        for (_, corners, shade) in visible_faces {
            let face_color = shade_color(base_color, *shade);
            let base = vertices.len() as u32;
            for corner in corners {
                let point = glam::Vec3::new(
                    min[0] + (max[0] - min[0]) * corner[0],
                    min[1] + (max[1] - min[1]) * corner[1],
                    min[2] + (max[2] - min[2]) * corner[2],
                );
                let world = rotation.mul_vec3(point) + translation;
                bounds_min = bounds_min.min(world);
                bounds_max = bounds_max.max(world);
                vertices.push(VoxelVertex {
                    position: world.to_array(),
                    color: face_color,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    if !bounds_min.is_finite() || !bounds_max.is_finite() {
        return (translation, translation);
    }

    (bounds_min, bounds_max)
}
