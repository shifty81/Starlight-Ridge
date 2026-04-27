use game_world::{MapMetadata, PropPlacement, SpawnPoint, TriggerZone};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: String,
    pub display_name: String,
    pub max_stack: u32,
    pub sell_price: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropDef {
    pub id: String,
    pub display_name: String,
    pub growth_days: u32,
    pub regrow_days: Option<u32>,
    pub harvest_item_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDef {
    pub id: String,
    pub display_name: String,
    pub home_map: String,
    pub schedule_id: String,
    pub dialogue_id: String,
    pub shop_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub time: String,
    pub action: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDef {
    pub id: String,
    pub entries: Vec<ScheduleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueResponseDef {
    pub text: String,
    pub next: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNodeDef {
    pub id: String,
    pub text: String,
    pub responses: Vec<DialogueResponseDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueDef {
    pub id: String,
    pub start: String,
    pub nodes: Vec<DialogueNodeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDef {
    pub id: String,
    pub display_name: String,
    pub objectives: Vec<String>,
    pub rewards: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopStockEntry {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopDef {
    pub id: String,
    pub stock: Vec<ShopStockEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasEntryDef {
    pub id: String,
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TilesetDef {
    pub id: String,
    pub display_name: String,
    pub texture_path: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub named_tiles: Vec<AtlasEntryDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteSheetDef {
    pub id: String,
    pub display_name: String,
    pub texture_path: String,
    pub columns: u32,
    pub rows: u32,
    pub entries: Vec<AtlasEntryDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerLegendEntry {
    pub symbol: String,
    pub tile_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileLayerDef {
    pub id: String,
    pub visible: bool,
    pub legend: Vec<LayerLegendEntry>,
    pub rows: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayersDef {
    pub map_id: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub layers: Vec<TileLayerDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapBundle {
    pub metadata: MapMetadata,
    pub props: Vec<PropPlacement>,
    pub spawns: Vec<SpawnPoint>,
    pub triggers: Vec<TriggerZone>,
}


// -----------------------------------------------------------------------------
// Phase 17 terrain contract definitions
// -----------------------------------------------------------------------------
//
// These data definitions are loaded into ContentRegistry but are not yet required
// by the current renderer. They let the repo begin using semantic terrain,
// biome packs, transition sets, and PCG/editor rules without breaking existing
// map layer rendering.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainCatalogDef {
    pub terrain_types: Vec<TerrainTypeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainTypeDef {
    pub id: String,
    pub display_name: String,
    pub family: String,
    pub layer_role: String,
    pub walkable: bool,
    pub blocks_movement: bool,
    pub farmable: bool,
    pub water: bool,
    pub visual_priority: i32,
    pub pcg_tags: Vec<String>,
    pub base_variants: String,
    pub fallback_tile_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomePackDef {
    pub id: String,
    pub display_name: String,
    pub atlas_id: String,
    pub ruleset: String,
    pub terrain_variant_sets: Vec<TerrainVariantSetDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainVariantSetDef {
    pub id: String,
    pub terrain_id: String,
    pub min_variants: u32,
    pub fallback_tile_id: String,
    pub variants: Vec<TerrainVariantDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainVariantDef {
    pub tile_id: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionSetDef {
    pub id: String,
    pub display_name: String,
    pub transitions: Vec<TransitionRuleDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRuleDef {
    pub id: String,
    pub from: String,
    pub to: String,
    pub mode: String,
    pub render_layer: u8,
    pub fallback_tile_id: String,
    pub tiles: Vec<TransitionMaskTileDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionMaskTileDef {
    pub mask: u32,
    pub tile_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainRulesetDef {
    pub id: String,
    pub display_name: String,
    pub active_transition_sets: Vec<String>,
    pub terrain_priority: Vec<String>,
    pub pcg_rules: TerrainPcgRulesDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainPcgRulesDef {
    pub require_sand_band_between_grass_and_open_water: bool,
    pub minimum_pond_radius_tiles: u32,
    pub preferred_pond_edge_noise: f32,
    pub coast_smoothing_passes: u32,
    pub protect_prefab_cells: bool,
}


// -----------------------------------------------------------------------------
// Phase 19 editor/web atlas pipeline definitions
// -----------------------------------------------------------------------------
//
// These contracts describe how the editor should present atlases, seasonal
// variants, water animation frames, clipboard/paste tools, validation checks,
// and game-preview profiles. They are intentionally data-first so a web editor
// or native editor can use the same source of truth.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorAtlasPipelineCatalogDef {
    pub id: String,
    pub display_name: String,
    pub tile_size: u32,
    pub active_season: String,
    pub atlases: Vec<EditorAtlasDef>,
    pub season_sets: Vec<SeasonVariantSetDef>,
    pub water_animations: Vec<WaterAnimationDef>,
    pub clipboard_tools: Vec<ClipboardToolDef>,
    pub validation_checks: Vec<EditorValidationCheckDef>,
    pub game_preview_profiles: Vec<GamePreviewProfileDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorAtlasDef {
    pub id: String,
    pub display_name: String,
    pub asset_ref: String,
    pub asset_kind: String,
    pub render_layer: u8,
    pub editable: bool,
    pub source_role: String,
    pub allowed_categories: Vec<String>,
    pub forbidden_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasTileRefDef {
    pub atlas_id: String,
    pub tile_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonVariantSetDef {
    pub id: String,
    pub semantic_tile_id: String,
    pub spring: AtlasTileRefDef,
    pub summer: AtlasTileRefDef,
    pub fall: AtlasTileRefDef,
    pub winter: AtlasTileRefDef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterAnimationDef {
    pub id: String,
    pub terrain_id: String,
    pub frames: Vec<AtlasTileRefDef>,
    pub frame_ms: u32,
    pub loop_mode: String,
    pub render_layer: u8,
    pub random_start_offset: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardToolDef {
    pub id: String,
    pub display_name: String,
    pub snap_to_tile_grid: bool,
    pub mirror_horizontal: bool,
    pub mirror_vertical: bool,
    pub rotate_90: bool,
    pub palette_remap: bool,
    pub assign_metadata_after_paste: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorValidationCheckDef {
    pub id: String,
    pub severity: String,
    pub enabled: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePreviewProfileDef {
    pub id: String,
    pub display_name: String,
    pub season: String,
    pub show_editor_overlays: bool,
    pub show_collision: bool,
    pub show_water_animation: bool,
    pub show_props: bool,
    pub show_transitions: bool,
    pub day_time_minutes: u32,
}


// -----------------------------------------------------------------------------
// Phase 20 editor export, validation, and autotile pipeline definitions
// -----------------------------------------------------------------------------
//
// Phase 19 defines the atlas/source-art contract. Phase 20 defines the editor
// production pipeline that turns that contract into safe game content: export
// packs, validation panels, data-driven autotile rules, transition editor
// scaffolds, collision/interaction metadata, and atlas cleanup manifests.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorExportValidationPipelineDef {
    pub id: String,
    pub display_name: String,
    pub export_profiles: Vec<ExportPackProfileDef>,
    pub validation_panels: Vec<EditorValidationPanelDef>,
    pub autotile_rule_sets: Vec<EditorAutotileRuleSetDef>,
    pub transition_rule_editors: Vec<TerrainTransitionRuleEditorDef>,
    pub collision_interaction_profiles: Vec<CollisionInteractionProfileDef>,
    pub atlas_cleanup_manifests: Vec<AtlasCleanupManifestDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPackProfileDef {
    pub id: String,
    pub display_name: String,
    pub target_root: String,
    pub include_paths: Vec<String>,
    pub required_outputs: Vec<String>,
    pub dry_run_default: bool,
    pub write_manifest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorValidationPanelDef {
    pub id: String,
    pub display_name: String,
    pub issue_filters: Vec<EditorValidationIssueFilterDef>,
    pub jump_targets: Vec<EditorValidationJumpTargetDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorValidationIssueFilterDef {
    pub severity: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorValidationJumpTargetDef {
    pub kind: String,
    pub can_open: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorAutotileRuleSetDef {
    pub id: String,
    pub display_name: String,
    pub terrain_family: String,
    pub mode: String,
    pub output_layer: u8,
    pub rules: Vec<EditorAutotileRuleDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorAutotileRuleDef {
    pub id: String,
    pub from: String,
    pub to: String,
    pub priority: i32,
    pub neighbor_mask: u32,
    pub output_atlas_id: String,
    pub output_tile_id: String,
    pub tool_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainTransitionRuleEditorDef {
    pub id: String,
    pub display_name: String,
    pub rule_set_id: String,
    pub editable_fields: Vec<String>,
    pub preview_pairs: Vec<TerrainPreviewPairDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainPreviewPairDef {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionInteractionProfileDef {
    pub id: String,
    pub target_kind: String,
    pub target_id: String,
    pub walkable: bool,
    pub blocks_movement: bool,
    pub water: bool,
    pub collision_bounds: CollisionBoundsDef,
    pub interaction: Option<InteractionMetadataDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionBoundsDef {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionMetadataDef {
    pub prompt: String,
    pub required_tool: Option<String>,
    pub drops: Vec<String>,
    pub season_visibility: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasCleanupManifestDef {
    pub id: String,
    pub display_name: String,
    pub source_atlas_id: String,
    pub source_role: String,
    pub target_atlas_id: String,
    pub target_role: String,
    pub actions: Vec<AtlasCleanupActionDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasCleanupActionDef {
    pub id: String,
    pub action: String,
    pub source_category: String,
    pub target_category: String,
    pub reason: String,
}


// -----------------------------------------------------------------------------
// Phase 21 animation editor timeline, events, sockets, and hitbox definitions
// -----------------------------------------------------------------------------
//
// Phase 21 turns animations into gameplay-aware editor data. It describes
// timeline schemas, frame events, directional groups, per-frame tool sockets,
// hitboxes/interaction boxes, water animation previews, seasonal animation
// variants, and validation reports that can be consumed by both the web editor
// scaffold and future native editor tools.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorAnimationPipelineDef {
    pub id: String,
    pub display_name: String,
    pub default_frame_ms: u32,
    pub timeline_schemas: Vec<AnimationTimelineSchemaDef>,
    pub animation_clips: Vec<AnimationClipDef>,
    pub directional_groups: Vec<DirectionalAnimationGroupDef>,
    pub socket_profiles: Vec<AnimationSocketProfileDef>,
    pub hitbox_profiles: Vec<AnimationHitboxProfileDef>,
    pub water_preview_profiles: Vec<WaterAnimationPreviewProfileDef>,
    pub seasonal_animation_sets: Vec<SeasonalAnimationSetDef>,
    pub validation_reports: Vec<AnimationValidationReportDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationTimelineSchemaDef {
    pub id: String,
    pub display_name: String,
    pub supported_loop_modes: Vec<String>,
    pub required_tracks: Vec<String>,
    pub marker_kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationClipDef {
    pub id: String,
    pub display_name: String,
    pub target_kind: String,
    pub target_id: String,
    pub direction: String,
    pub loop_mode: String,
    pub frames: Vec<AnimationFrameDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrameDef {
    pub index: u32,
    pub sprite_sheet_id: String,
    pub sprite_id: String,
    pub duration_ms: u32,
    pub events: Vec<AnimationFrameEventDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrameEventDef {
    pub id: String,
    pub frame_index: u32,
    pub event_kind: String,
    pub payload: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalAnimationGroupDef {
    pub id: String,
    pub display_name: String,
    pub directions: Vec<DirectionalAnimationRefDef>,
    pub mirror_left_from_right: bool,
    pub fallback_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalAnimationRefDef {
    pub direction: String,
    pub clip_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSocketProfileDef {
    pub id: String,
    pub clip_id: String,
    pub sockets: Vec<AnimationSocketFrameDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSocketFrameDef {
    pub frame_index: u32,
    pub socket_id: String,
    pub x: i32,
    pub y: i32,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationHitboxProfileDef {
    pub id: String,
    pub clip_id: String,
    pub boxes: Vec<AnimationHitboxFrameDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationHitboxFrameDef {
    pub frame_index: u32,
    pub box_kind: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterAnimationPreviewProfileDef {
    pub id: String,
    pub animation_id: String,
    pub tilemap_width: u32,
    pub tilemap_height: u32,
    pub random_start_offset: bool,
    pub show_shore_overlay: bool,
    pub season: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalAnimationSetDef {
    pub id: String,
    pub semantic_animation_id: String,
    pub spring_clip_id: String,
    pub summer_clip_id: String,
    pub fall_clip_id: String,
    pub winter_clip_id: String,
    pub fallback_clip_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationValidationReportDef {
    pub id: String,
    pub severity: String,
    pub enabled: bool,
    pub checks: Vec<String>,
}

// -----------------------------------------------------------------------------
// -----------------------------------------------------------------------------
// Phase 51i hybrid 2D/3D world, presentation, and render pipeline definitions
// -----------------------------------------------------------------------------
//
// These definitions keep Starlight Ridge authored as a readable gameplay grid
// while allowing the same scene to carry height/elevation, 3D object placement,
// presentation camera profiles, and render/bake metadata. The current runtime can
// continue using the 2D tile layers while the editor grows 2.5D/3D preview and
// baking workflows around the same map id.

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SceneRenderModeDef {
    Tile2D,
    Hybrid2_5D,
    Scene3D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeightMapDef {
    pub map_id: String,
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub default_height: i16,
    pub min_height: i16,
    pub max_height: i16,
    pub values: Vec<i16>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene3DDef {
    pub map_id: String,
    pub coordinate_space: String,
    pub units_per_tile: f32,
    pub objects: Vec<SceneObject3DDef>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneObject3DDef {
    pub id: String,
    pub asset_id: String,
    pub source_kind: SceneAssetSourceKindDef,
    pub visual_mode: SceneObjectVisualModeDef,
    pub cell_x: u32,
    pub cell_y: u32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    pub rotation_degrees: f32,
    pub scale: f32,
    pub collision_cells: Vec<GridCellDef>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SceneAssetSourceKindDef {
    Sprite2D,
    Vox,
    Blockbench,
    Blender,
    Gltf,
    GeneratedBake,
    Placeholder,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SceneObjectVisualModeDef {
    SpriteBillboard,
    BakedSprite,
    Live3D,
    HybridProxy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCellDef {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationDef {
    pub map_id: String,
    pub default_mode: SceneRenderModeDef,
    pub depth_sorting: bool,
    pub sprite_billboarding: bool,
    pub pixel_snap: bool,
    pub active_camera_profile: String,
    pub camera_profiles: Vec<CameraProfileDef>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraProfileDef {
    pub id: String,
    pub display_name: String,
    pub mode: SceneRenderModeDef,
    pub pitch_degrees: f32,
    pub yaw_degrees: f32,
    pub orthographic_scale: f32,
    pub perspective_fov_degrees: f32,
    pub near_clip: f32,
    pub far_clip: f32,
    pub pixel_snap: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingProfileDef {
    pub map_id: String,
    pub active_profile: String,
    pub profiles: Vec<LightingProfileEntryDef>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingProfileEntryDef {
    pub id: String,
    pub display_name: String,
    pub time_of_day: String,
    pub ambient_strength: f32,
    pub sun_yaw_degrees: f32,
    pub sun_pitch_degrees: f32,
    pub shadow_strength: f32,
    pub weather_modifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridWorldEditorPipelineDef {
    pub id: String,
    pub display_name: String,
    pub default_render_mode: SceneRenderModeDef,
    pub world_subtabs: Vec<String>,
    pub asset_subtabs: Vec<String>,
    pub render_subtabs: Vec<String>,
    pub external_tool_targets: Vec<String>,
    pub automation_goals: Vec<String>,
    pub notes: Vec<String>,
}

// Phase 51 world graph / scene-layer registry definitions
// -----------------------------------------------------------------------------

pub type WorldManifestDef = shared_types::WorldManifest;
pub type WorldgenEditorWorkflowDef = shared_types::WorldgenEditorWorkflow;
pub type ProtectedLayerPolicyDef = shared_types::ProtectedLayerPolicy;
pub type GeneratedSceneDraftDef = shared_types::GeneratedSceneDraft;
pub type SceneBakeContractDef = shared_types::SceneBakeContract;
