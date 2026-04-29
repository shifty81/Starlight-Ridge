use crate::{GeneratedScene, SemanticTerrainId};
use game_data::defs::{WorldgenBiomeRulePackDef, WorldgenBiomeTerrainRuleDef, WorldgenSeasonTileDef};
use game_data::registry::ContentRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldgenSeason {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl WorldgenSeason {
    pub const ALL: [WorldgenSeason; 4] = [
        WorldgenSeason::Spring,
        WorldgenSeason::Summer,
        WorldgenSeason::Fall,
        WorldgenSeason::Winter,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            WorldgenSeason::Spring => "spring",
            WorldgenSeason::Summer => "summer",
            WorldgenSeason::Fall => "fall",
            WorldgenSeason::Winter => "winter",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            WorldgenSeason::Spring => "Spring",
            WorldgenSeason::Summer => "Summer",
            WorldgenSeason::Fall => "Fall",
            WorldgenSeason::Winter => "Winter",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "summer" => WorldgenSeason::Summer,
            "fall" | "autumn" => WorldgenSeason::Fall,
            "winter" => WorldgenSeason::Winter,
            _ => WorldgenSeason::Spring,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainResolveRequest {
    pub biome_pack_id: String,
    pub season: WorldgenSeason,
    pub semantic_terrain_id: String,
    pub x: u32,
    pub y: u32,
    pub seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTerrainTile {
    pub biome_pack_id: String,
    pub season: String,
    pub semantic_terrain_id: String,
    pub family: String,
    pub layer_role: String,
    pub tile_id: String,
    pub overlay_tile_id: String,
    pub water_animation_id: String,
    pub walkable: bool,
    pub blocks_movement: bool,
    pub liquid: bool,
    pub liquid_behavior: String,
    pub snow_accumulates: bool,
    pub weather_accumulation: String,
    pub used_fallback: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedSeasonalScene {
    pub scene_id: String,
    pub biome_pack_id: String,
    pub season: String,
    pub width: u32,
    pub height: u32,
    pub base_tiles: Vec<String>,
    pub overlay_tiles: Vec<String>,
    pub water_animation_ids: Vec<String>,
    pub warnings: Vec<String>,
    pub notes: Vec<String>,
}

pub fn semantic_terrain_to_rule_id(terrain: SemanticTerrainId) -> &'static str {
    match terrain {
        SemanticTerrainId::Void => "void",
        SemanticTerrainId::Grass => "grass",
        SemanticTerrainId::GrassDark => "grass_dark",
        SemanticTerrainId::ForestFloor => "forest_floor",
        SemanticTerrainId::FarmableSoil => "farmable_soil",
        SemanticTerrainId::PathDirt => "path_dirt",
        SemanticTerrainId::PathStone => "path_stone",
        SemanticTerrainId::Sand => "sand",
        SemanticTerrainId::ShallowWater => "shallow_water",
        SemanticTerrainId::DeepWater => "deep_water",
        SemanticTerrainId::CoastFoam => "coast_foam",
        SemanticTerrainId::Cliff => "cliff",
        SemanticTerrainId::Rock => "rock",
        SemanticTerrainId::TreeSpawn => "forest_floor",
        SemanticTerrainId::WeedSpawn => "grass",
        SemanticTerrainId::BuildingZone => "path_stone",
        SemanticTerrainId::ExitZone => "path_dirt",
        SemanticTerrainId::Protected => "protected",
    }
}

pub fn resolve_terrain_tile(
    registry: &ContentRegistry,
    request: &TerrainResolveRequest,
) -> ResolvedTerrainTile {
    let Some(pack) = resolve_pack(registry, &request.biome_pack_id) else {
        return unresolved_tile(request, "missing biome rule pack");
    };

    let semantic_id = request.semantic_terrain_id.as_str();
    let default_rule = pack
        .terrain_rules
        .iter()
        .find(|rule| rule.semantic_terrain_id == "grass")
        .or_else(|| pack.terrain_rules.first());
    let Some(rule) = pack
        .terrain_rules
        .iter()
        .find(|rule| rule.semantic_terrain_id == semantic_id)
        .or(default_rule)
    else {
        return unresolved_tile(request, "biome rule pack has no terrain rules");
    };

    let matched_exact_rule = rule.semantic_terrain_id == semantic_id;
    let season = request.season.as_str();
    let season_tile = resolve_season_tile(pack, rule, season);
    let mut used_fallback = season_tile.is_none() || !matched_exact_rule;
    let mut notes = Vec::new();
    if !matched_exact_rule {
        notes.push(format!(
            "No explicit rule for semantic terrain '{}'; used '{}'.",
            semantic_id, rule.semantic_terrain_id
        ));
    }

    let base_tile = season_tile
        .map(|tile| tile.tile_id.as_str())
        .filter(|tile_id| !tile_id.trim().is_empty())
        .unwrap_or_else(|| {
            used_fallback = true;
            rule.fallback_tile_id.as_str()
        });

    let selected_tile = choose_variant_tile(rule, request.seed, request.x, request.y)
        .filter(|_| variation_roll(rule, request.seed, request.x, request.y))
        .unwrap_or(base_tile)
        .to_string();

    let overlay_tile_id = season_tile
        .map(|tile| tile.overlay_tile_id.clone())
        .unwrap_or_default();
    let water_animation_id = season_tile
        .map(|tile| tile.water_animation_id.clone())
        .unwrap_or_default();

    ResolvedTerrainTile {
        biome_pack_id: pack.id.clone(),
        season: season.to_string(),
        semantic_terrain_id: semantic_id.to_string(),
        family: rule.family.clone(),
        layer_role: rule.layer_role.clone(),
        tile_id: selected_tile,
        overlay_tile_id,
        water_animation_id,
        walkable: rule.walkable,
        blocks_movement: rule.blocks_movement,
        liquid: rule.liquid,
        liquid_behavior: rule.liquid_behavior.clone(),
        snow_accumulates: rule.snow_accumulates,
        weather_accumulation: rule.weather_accumulation.clone(),
        used_fallback,
        notes,
    }
}

pub fn resolve_generated_scene_to_tile_ids(
    registry: &ContentRegistry,
    scene: &GeneratedScene,
    biome_pack_id: &str,
    season: WorldgenSeason,
) -> ResolvedSeasonalScene {
    let mut base_tiles = Vec::with_capacity(scene.semantic_tiles.len());
    let mut overlay_tiles = Vec::with_capacity(scene.semantic_tiles.len());
    let mut water_animation_ids = Vec::with_capacity(scene.semantic_tiles.len());
    let mut warnings = Vec::new();

    for y in 0..scene.height {
        for x in 0..scene.width {
            let terrain = scene
                .terrain_at(x, y)
                .unwrap_or(SemanticTerrainId::Void);
            let request = TerrainResolveRequest {
                biome_pack_id: biome_pack_id.to_string(),
                season,
                semantic_terrain_id: semantic_terrain_to_rule_id(terrain).to_string(),
                x,
                y,
                seed: scene.seed,
            };
            let resolved = resolve_terrain_tile(registry, &request);
            if resolved.used_fallback && warnings.len() < 32 {
                warnings.push(format!(
                    "{},{} '{}' resolved through fallback tile '{}'.",
                    x, y, resolved.semantic_terrain_id, resolved.tile_id
                ));
            }
            base_tiles.push(resolved.tile_id);
            overlay_tiles.push(resolved.overlay_tile_id);
            water_animation_ids.push(resolved.water_animation_id);
        }
    }

    ResolvedSeasonalScene {
        scene_id: scene.scene_id.clone(),
        biome_pack_id: biome_pack_id.to_string(),
        season: season.as_str().to_string(),
        width: scene.width,
        height: scene.height,
        base_tiles,
        overlay_tiles,
        water_animation_ids,
        warnings,
        notes: vec![
            "Phase 52c resolved semantic terrain through a seasonal biome rule pack.".to_string(),
            "Autotile baking is still responsible for final transition/edge layers.".to_string(),
        ],
    }
}

fn resolve_pack<'a>(
    registry: &'a ContentRegistry,
    biome_pack_id: &str,
) -> Option<&'a WorldgenBiomeRulePackDef> {
    registry
        .worldgen_biome_rule_packs
        .get(biome_pack_id)
        .or_else(|| registry.active_worldgen_biome_rule_pack())
}

fn resolve_season_tile<'a>(
    pack: &'a WorldgenBiomeRulePackDef,
    rule: &'a WorldgenBiomeTerrainRuleDef,
    season: &str,
) -> Option<&'a WorldgenSeasonTileDef> {
    rule.seasonal_tiles
        .iter()
        .find(|tile| tile.season == season)
        .or_else(|| {
            rule.seasonal_tiles
                .iter()
                .find(|tile| tile.season == pack.default_season)
        })
        .or_else(|| rule.seasonal_tiles.first())
}

fn choose_variant_tile<'a>(
    rule: &'a WorldgenBiomeTerrainRuleDef,
    seed: u64,
    x: u32,
    y: u32,
) -> Option<&'a str> {
    let total_weight: u32 = rule.variants.iter().map(|variant| variant.weight).sum();
    if total_weight == 0 {
        return None;
    }

    let mut pick = (stable_hash(seed, x, y, 0x52C0_A57A) % total_weight as u64) as u32;
    for variant in &rule.variants {
        if pick < variant.weight {
            return Some(variant.tile_id.as_str());
        }
        pick -= variant.weight;
    }
    None
}

fn variation_roll(rule: &WorldgenBiomeTerrainRuleDef, seed: u64, x: u32, y: u32) -> bool {
    if rule.variation_chance_percent == 0 {
        return false;
    }
    let roll = (stable_hash(seed, x, y, 0xC0A5_7A11) % 100) as u32;
    roll < rule.variation_chance_percent.min(100)
}

fn unresolved_tile(request: &TerrainResolveRequest, note: &str) -> ResolvedTerrainTile {
    ResolvedTerrainTile {
        biome_pack_id: request.biome_pack_id.clone(),
        season: request.season.as_str().to_string(),
        semantic_terrain_id: request.semantic_terrain_id.clone(),
        family: "unknown".to_string(),
        layer_role: "base_terrain".to_string(),
        tile_id: "missing_tile".to_string(),
        overlay_tile_id: String::new(),
        water_animation_id: String::new(),
        walkable: false,
        blocks_movement: true,
        liquid: false,
        liquid_behavior: String::new(),
        snow_accumulates: false,
        weather_accumulation: String::new(),
        used_fallback: true,
        notes: vec![note.to_string()],
    }
}

fn stable_hash(seed: u64, x: u32, y: u32, salt: u64) -> u64 {
    let mut n = seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    n ^= (x as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    n = n.rotate_left(17);
    n ^= (y as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
    n ^= n >> 30;
    n = n.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    n ^= n >> 27;
    n = n.wrapping_mul(0x94D0_49BB_1331_11EB);
    n ^ (n >> 31)
}
