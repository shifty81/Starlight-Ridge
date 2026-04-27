//! Phase 52A core world/content contracts.
//!
//! These definitions are intentionally data-first and editor-friendly. They are
//! not wired into gameplay generation yet; they establish the schema that future
//! worldgen, simulation, `.vox` baking, web editor, and egui editor panels should
//! share.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn load_phase52_contract<T: DeserializeOwned>(path: impl AsRef<Path>) -> anyhow::Result<T> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)?;
    let value = ron::from_str(&text)?;
    Ok(value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase52ContractManifest {
    pub id: String,
    pub display_name: String,
    pub version: u32,
    pub biomes_path: String,
    pub materials_path: String,
    pub liquids_path: String,
    pub seasons_path: String,
    pub weather_path: String,
    pub map_layers_path: String,
    pub worldgen_presets_path: String,
    pub vox_assets_path: String,
    pub editor_help_path: String,
    pub validation_profile_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeCatalogDef {
    pub id: String,
    pub display_name: String,
    pub biomes: Vec<BiomeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDef {
    pub id: String,
    pub display_name: String,
    pub temperature_band: String,
    pub moisture_band: String,
    pub elevation_band: String,
    pub ocean_influence: f32,
    pub default_grass: Option<String>,
    pub default_sand: Option<String>,
    pub default_water: Option<String>,
    pub allowed_material_families: Vec<String>,
    pub allowed_liquids: Vec<String>,
    pub feature_tables: Vec<String>,
    pub weather_profile: String,
    pub season_profile: String,
    pub worldgen_weight: f32,
    pub editor_color: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialCatalogDef {
    pub id: String,
    pub display_name: String,
    pub tile_size: u32,
    pub materials: Vec<MaterialDef>,
    pub transition_sets: Vec<MaterialTransitionSetDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDef {
    pub id: String,
    pub display_name: String,
    pub family: String,
    pub kind: String,
    pub atlas_role: String,
    pub base_tile_hint: String,
    pub seasonal: bool,
    pub wet_variant: Option<String>,
    pub dry_variant: Option<String>,
    pub snow_cover_allowed: bool,
    pub walk_speed_multiplier: f32,
    pub fertility: f32,
    pub erosion_resistance: f32,
    pub absorbs_liquid: bool,
    pub editor_tooltip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialTransitionSetDef {
    pub id: String,
    pub from_family: String,
    pub to_family: String,
    pub rule_kind: String,
    pub output_layer: String,
    pub required_tiles: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidCatalogDef {
    pub id: String,
    pub display_name: String,
    pub liquids: Vec<LiquidDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidDef {
    pub id: String,
    pub display_name: String,
    pub family: String,
    pub density: f32,
    pub viscosity: f32,
    pub max_depth: f32,
    pub flow_rate: f32,
    pub evaporation_rate: f32,
    pub seep_rate: f32,
    pub freezes_below_c: Option<f32>,
    pub boils_above_c: Option<f32>,
    pub ignites: bool,
    pub damages_entities: bool,
    pub extinguishes_fire: bool,
    pub render_animation: String,
    pub editor_tooltip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherCatalogDef {
    pub id: String,
    pub display_name: String,
    pub profiles: Vec<WeatherProfileDef>,
    pub events: Vec<WeatherEventDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherProfileDef {
    pub id: String,
    pub display_name: String,
    pub biome_ids: Vec<String>,
    pub event_weights: Vec<WeatherEventWeightDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEventWeightDef {
    pub event_id: String,
    pub spring: f32,
    pub summer: f32,
    pub fall: f32,
    pub winter: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEventDef {
    pub id: String,
    pub display_name: String,
    pub precipitation_kind: String,
    pub intensity_min: f32,
    pub intensity_max: f32,
    pub wetness_per_hour: f32,
    pub snow_depth_per_hour: f32,
    pub wind_strength: f32,
    pub puddle_enabled: bool,
    pub accumulation_layer: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonCatalogDef {
    pub id: String,
    pub display_name: String,
    pub default_days_per_season: u32,
    pub seasons: Vec<SeasonDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonDef {
    pub id: String,
    pub display_name: String,
    pub average_temperature_c: f32,
    pub daylight_start_hour: f32,
    pub daylight_end_hour: f32,
    pub material_variant_suffix: String,
    pub supports_snow_baseline: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayerSchemaDef {
    pub id: String,
    pub display_name: String,
    pub visible_layers: Vec<MapLayerDef>,
    pub derived_layers: Vec<MapLayerDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLayerDef {
    pub id: String,
    pub display_name: String,
    pub order: i32,
    pub category: String,
    pub editable: bool,
    pub saved_to_map: bool,
    pub supports_autotile: bool,
    pub supports_simulation: bool,
    pub default_visible: bool,
    pub tooltip: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldgenPresetCatalogDef {
    pub id: String,
    pub display_name: String,
    pub presets: Vec<WorldgenPresetDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldgenPresetDef {
    pub id: String,
    pub display_name: String,
    pub scene_kind: String,
    pub width: u32,
    pub height: u32,
    pub seed_policy: String,
    pub biome_table: Vec<String>,
    pub passes: Vec<WorldgenPassDef>,
    pub validation_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldgenPassDef {
    pub id: String,
    pub display_name: String,
    pub pass_kind: String,
    pub order: i32,
    pub enabled: bool,
    pub writes_layers: Vec<String>,
    pub reads_layers: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxAssetCatalogDef {
    pub id: String,
    pub display_name: String,
    pub bake_profiles: Vec<VoxBakeProfileDef>,
    pub assets: Vec<VoxAssetDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxBakeProfileDef {
    pub id: String,
    pub display_name: String,
    pub output_tile_size: u32,
    pub directions: Vec<String>,
    pub orthographic_pitch_degrees: f32,
    pub render_scale: f32,
    pub emit_collision_mask: bool,
    pub emit_shadow_mask: bool,
    pub emit_footprint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxAssetDef {
    pub id: String,
    pub display_name: String,
    pub source_path: String,
    pub category: String,
    pub bake_profile: String,
    pub footprint_width: u32,
    pub footprint_height: u32,
    pub anchor_x: i32,
    pub anchor_y: i32,
    pub collision_kind: String,
    pub seasonal: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorHelpCatalogDef {
    pub id: String,
    pub display_name: String,
    pub entries: Vec<EditorHelpEntryDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorHelpEntryDef {
    pub id: String,
    pub surface: String,
    pub field: String,
    pub tooltip_short: String,
    pub wiki_path: String,
    pub validation_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase52ValidationProfileDef {
    pub id: String,
    pub display_name: String,
    pub checks: Vec<Phase52ValidationCheckDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase52ValidationCheckDef {
    pub id: String,
    pub severity: String,
    pub target: String,
    pub enabled: bool,
    pub description: String,
    pub help_entry_id: String,
}
