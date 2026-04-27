//! Scene-based procedural generation scaffold for Starlight Ridge.
//!
//! This crate intentionally generates semantic scene data instead of atlas cell
//! coordinates. The autotile resolver remains responsible for converting these
//! semantic roles into renderable tile IDs.

pub mod phase52_contracts;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::Path;

pub const DEFAULT_SCENE_TILE_SIZE: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SceneKind {
    StarterFarm,
    CoastalFarm,
    FarmPlot,
    Town,
    Forest,
    Beach,
    Cave,
    Dungeon,
    Interior,
    EventMap,
}

impl fmt::Display for SceneKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SemanticTerrainId {
    Void,
    Grass,
    GrassDark,
    ForestFloor,
    FarmableSoil,
    PathDirt,
    PathStone,
    Sand,
    ShallowWater,
    DeepWater,
    CoastFoam,
    Cliff,
    Rock,
    TreeSpawn,
    WeedSpawn,
    BuildingZone,
    ExitZone,
    Protected,
}

impl SemanticTerrainId {
    pub fn is_water(self) -> bool {
        matches!(self, Self::ShallowWater | Self::DeepWater | Self::CoastFoam)
    }

    pub fn is_natural_object_marker(self) -> bool {
        matches!(self, Self::TreeSpawn | Self::WeedSpawn | Self::Rock)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneGenRequest {
    pub scene_id: String,
    pub kind: SceneKind,
    pub seed: u64,
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub template_id: String,
    pub allow_coast: bool,
    pub allow_forest: bool,
    pub allow_rocks: bool,
    pub allow_farmable_clearings: bool,
}

impl SceneGenRequest {
    pub fn starter_farm(seed: u64) -> Self {
        Self {
            scene_id: "starter_farm_generated_draft".to_string(),
            kind: SceneKind::StarterFarm,
            seed,
            width: 96,
            height: 96,
            tile_size: DEFAULT_SCENE_TILE_SIZE,
            template_id: "starter_farm_coastal_peninsula_v1".to_string(),
            allow_coast: true,
            allow_forest: true,
            allow_rocks: true,
            allow_farmable_clearings: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneTemplate {
    pub id: String,
    pub display_name: String,
    pub kind: SceneKind,
    pub default_width: u32,
    pub default_height: u32,
    pub required_zones: Vec<SceneZoneRule>,
    pub optional_zones: Vec<SceneZoneRule>,
    pub protected_layers: ProtectedLayerRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneZoneRule {
    pub id: String,
    pub semantic: SemanticTerrainId,
    pub min_width: u32,
    pub min_height: u32,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedLayerRules {
    pub regeneratable: Vec<String>,
    pub protected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedScene {
    pub scene_id: String,
    pub kind: SceneKind,
    pub seed: u64,
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub template_id: String,
    pub semantic_tiles: Vec<SemanticTerrainId>,
    pub object_spawns: Vec<SceneObjectSpawn>,
    pub exits: Vec<SceneExit>,
    pub protected_layers: ProtectedLayerRules,
    pub generator_notes: Vec<String>,
}

impl GeneratedScene {
    pub fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }

    pub fn terrain_at(&self, x: u32, y: u32) -> Option<SemanticTerrainId> {
        self.index(x, y).and_then(|idx| self.semantic_tiles.get(idx).copied())
    }

    pub fn set_terrain(&mut self, x: u32, y: u32, terrain: SemanticTerrainId) {
        if let Some(idx) = self.index(x, y) {
            self.semantic_tiles[idx] = terrain;
        }
    }

    pub fn validate(&self) -> SceneValidationReport {
        let mut warnings = Vec::new();
        let mut counts: BTreeMap<SemanticTerrainId, usize> = BTreeMap::new();

        if self.semantic_tiles.len() != (self.width * self.height) as usize {
            warnings.push(format!(
                "semantic tile count {} does not match scene dimensions {}x{}",
                self.semantic_tiles.len(), self.width, self.height
            ));
        }

        for tile in &self.semantic_tiles {
            *counts.entry(*tile).or_insert(0) += 1;
        }

        if self.exits.is_empty() {
            warnings.push("scene has no exits".to_string());
        }

        if !counts.contains_key(&SemanticTerrainId::Grass)
            && !counts.contains_key(&SemanticTerrainId::FarmableSoil)
        {
            warnings.push("scene has no playable grass or farmable soil".to_string());
        }

        let mut seen_ids = BTreeSet::new();
        for spawn in &self.object_spawns {
            if !seen_ids.insert(spawn.instance_id.clone()) {
                warnings.push(format!("duplicate object spawn id {}", spawn.instance_id));
            }
        }

        SceneValidationReport {
            scene_id: self.scene_id.clone(),
            warnings,
            semantic_counts: counts,
            object_count: self.object_spawns.len(),
            exit_count: self.exits.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneObjectSpawn {
    pub instance_id: String,
    pub object_id: String,
    pub x: u32,
    pub y: u32,
    pub layer: String,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneExit {
    pub id: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub target_scene: String,
    pub target_spawn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneValidationReport {
    pub scene_id: String,
    pub warnings: Vec<String>,
    pub semantic_counts: BTreeMap<SemanticTerrainId, usize>,
    pub object_count: usize,
    pub exit_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutotileHandoff {
    pub scene_id: String,
    pub width: u32,
    pub height: u32,
    pub base_semantic: Vec<SemanticTerrainId>,
    pub transition_masks_dirty: bool,
    pub notes: Vec<String>,
}

impl From<&GeneratedScene> for AutotileHandoff {
    fn from(scene: &GeneratedScene) -> Self {
        Self {
            scene_id: scene.scene_id.clone(),
            width: scene.width,
            height: scene.height,
            base_semantic: scene.semantic_tiles.clone(),
            transition_masks_dirty: true,
            notes: vec![
                "WorldGen emitted semantic terrain only; run the autotile resolver before rendering."
                    .to_string(),
            ],
        }
    }
}

pub trait SceneGenerator {
    fn generate(&self, request: &SceneGenRequest) -> anyhow::Result<GeneratedScene>;
}

#[derive(Debug, Default)]
pub struct StarterFarmGenerator;

impl SceneGenerator for StarterFarmGenerator {
    fn generate(&self, request: &SceneGenRequest) -> anyhow::Result<GeneratedScene> {
        generate_coastal_farm_like_scene(request)
    }
}

#[derive(Debug, Default)]
pub struct CoastalPlotGenerator;

impl SceneGenerator for CoastalPlotGenerator {
    fn generate(&self, request: &SceneGenRequest) -> anyhow::Result<GeneratedScene> {
        generate_coastal_farm_like_scene(request)
    }
}

pub fn generate_scene(request: &SceneGenRequest) -> anyhow::Result<GeneratedScene> {
    match request.kind {
        SceneKind::StarterFarm | SceneKind::CoastalFarm | SceneKind::FarmPlot => {
            StarterFarmGenerator.generate(request)
        }
        _ => generate_basic_scene(request),
    }
}

pub fn save_generated_scene_ron(scene: &GeneratedScene, path: impl AsRef<Path>) -> anyhow::Result<()> {
    let ron = ron::ser::to_string_pretty(scene, ron::ser::PrettyConfig::default())?;
    std::fs::write(path, ron)?;
    Ok(())
}

fn generate_basic_scene(request: &SceneGenRequest) -> anyhow::Result<GeneratedScene> {
    ensure_valid_request(request)?;
    let mut scene = blank_scene(request, SemanticTerrainId::Grass);
    carve_border(&mut scene, SemanticTerrainId::Protected);
    add_standard_exits(&mut scene);
    scene.generator_notes.push(format!(
        "Generated {:?} as a basic editable scene scaffold.",
        request.kind
    ));
    Ok(scene)
}

fn generate_coastal_farm_like_scene(request: &SceneGenRequest) -> anyhow::Result<GeneratedScene> {
    ensure_valid_request(request)?;
    let mut scene = blank_scene(request, SemanticTerrainId::Grass);

    let center_x = request.width as f32 * 0.50;
    let center_y = request.height as f32 * 0.50;
    let farm_radius_x = request.width as f32 * 0.24;
    let farm_radius_y = request.height as f32 * 0.20;

    for y in 0..request.height {
        for x in 0..request.width {
            let nx = x as f32 / request.width.max(1) as f32;
            let ny = y as f32 / request.height.max(1) as f32;
            let coast_noise = value_noise(request.seed, x as i32, y as i32, 21);
            let forest_noise = value_noise(request.seed, x as i32, y as i32, 77);
            let rock_noise = value_noise(request.seed, x as i32, y as i32, 133);
            let dx = (x as f32 - center_x) / farm_radius_x.max(1.0);
            let dy = (y as f32 - center_y) / farm_radius_y.max(1.0);
            let in_farm_clearing = dx * dx + dy * dy < 1.0;

            let terrain = if request.allow_coast && ny > 0.78 + coast_noise * 0.05 {
                SemanticTerrainId::DeepWater
            } else if request.allow_coast && ny > 0.71 + coast_noise * 0.06 {
                SemanticTerrainId::ShallowWater
            } else if request.allow_coast && ny > 0.64 + coast_noise * 0.04 {
                SemanticTerrainId::Sand
            } else if request.allow_farmable_clearings && in_farm_clearing {
                SemanticTerrainId::FarmableSoil
            } else if request.allow_forest && (nx < 0.16 || nx > 0.84 || forest_noise > 0.34) {
                SemanticTerrainId::ForestFloor
            } else if request.allow_rocks && rock_noise > 0.43 {
                SemanticTerrainId::Rock
            } else if forest_noise > 0.22 {
                SemanticTerrainId::GrassDark
            } else {
                SemanticTerrainId::Grass
            };

            scene.set_terrain(x, y, terrain);
        }
    }

    carve_path_to_town(&mut scene);
    place_natural_objects(&mut scene, request.seed);
    add_standard_exits(&mut scene);
    scene.generator_notes.push(
        "Generated editable coastal farm draft: semantic terrain first, object markers second, autotile handoff last."
            .to_string(),
    );
    Ok(scene)
}

fn blank_scene(request: &SceneGenRequest, fill: SemanticTerrainId) -> GeneratedScene {
    GeneratedScene {
        scene_id: request.scene_id.clone(),
        kind: request.kind,
        seed: request.seed,
        width: request.width,
        height: request.height,
        tile_size: request.tile_size,
        template_id: request.template_id.clone(),
        semantic_tiles: vec![fill; (request.width * request.height) as usize],
        object_spawns: Vec::new(),
        exits: Vec::new(),
        protected_layers: ProtectedLayerRules {
            regeneratable: vec![
                "base_terrain".to_string(),
                "terrain_variation".to_string(),
                "natural_objects".to_string(),
            ],
            protected: vec![
                "buildings".to_string(),
                "npc_spawns".to_string(),
                "quest_triggers".to_string(),
                "doors".to_string(),
                "scene_exits".to_string(),
                "logic_bindings".to_string(),
            ],
        },
        generator_notes: Vec::new(),
    }
}

fn ensure_valid_request(request: &SceneGenRequest) -> anyhow::Result<()> {
    anyhow::ensure!(request.width > 8, "scene width must be greater than 8 tiles");
    anyhow::ensure!(request.height > 8, "scene height must be greater than 8 tiles");
    anyhow::ensure!(request.tile_size > 0, "scene tile size must be non-zero");
    Ok(())
}

fn carve_border(scene: &mut GeneratedScene, terrain: SemanticTerrainId) {
    let width = scene.width;
    let height = scene.height;
    for x in 0..width {
        scene.set_terrain(x, 0, terrain);
        scene.set_terrain(x, height - 1, terrain);
    }
    for y in 0..height {
        scene.set_terrain(0, y, terrain);
        scene.set_terrain(width - 1, y, terrain);
    }
}

fn carve_path_to_town(scene: &mut GeneratedScene) {
    let mid_x = scene.width / 2;
    let path_half_width = 2;
    for y in 0..scene.height {
        for dx in 0..=path_half_width * 2 {
            let x = mid_x.saturating_sub(path_half_width) + dx;
            if x < scene.width {
                scene.set_terrain(x, y, SemanticTerrainId::PathDirt);
            }
        }
    }
}

fn add_standard_exits(scene: &mut GeneratedScene) {
    let north_x = scene.width.saturating_div(2).saturating_sub(2);
    scene.exits.push(SceneExit {
        id: "north_to_town".to_string(),
        x: north_x,
        y: 0,
        width: 5,
        height: 1,
        target_scene: "town".to_string(),
        target_spawn: "south_gate".to_string(),
    });
}

fn place_natural_objects(scene: &mut GeneratedScene, seed: u64) {
    let mut tree_count = 0usize;
    let mut rock_count = 0usize;
    let width = scene.width;
    let height = scene.height;

    for y in 2..height.saturating_sub(2) {
        for x in 2..width.saturating_sub(2) {
            let Some(terrain) = scene.terrain_at(x, y) else { continue };
            if matches!(terrain, SemanticTerrainId::PathDirt | SemanticTerrainId::FarmableSoil)
                || terrain.is_water()
            {
                continue;
            }

            let object_noise = value_noise(seed, x as i32, y as i32, 401);
            let spacing_ok = ((x + y * 3) % 5) != 0;
            if terrain == SemanticTerrainId::ForestFloor && object_noise > 0.39 && spacing_ok {
                tree_count += 1;
                scene.object_spawns.push(SceneObjectSpawn {
                    instance_id: format!("tree_{tree_count:04}"),
                    object_id: "weak_tree_full".to_string(),
                    x,
                    y,
                    layer: "natural_objects".to_string(),
                    protected: false,
                });
            } else if terrain == SemanticTerrainId::Rock && object_noise > 0.18 {
                rock_count += 1;
                scene.object_spawns.push(SceneObjectSpawn {
                    instance_id: format!("rock_{rock_count:04}"),
                    object_id: "big_stone_small".to_string(),
                    x,
                    y,
                    layer: "natural_objects".to_string(),
                    protected: false,
                });
            }
        }
    }
}

fn value_noise(seed: u64, x: i32, y: i32, salt: u64) -> f32 {
    let mut n = seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    n ^= (x as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    n = n.rotate_left(17);
    n ^= (y as i64 as u64).wrapping_mul(0x94D0_49BB_1331_11EB);
    n ^= n >> 30;
    n = n.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    n ^= n >> 27;
    n = n.wrapping_mul(0x94D0_49BB_1331_11EB);
    n ^= n >> 31;
    let unit = (n as f64 / u64::MAX as f64) as f32;
    unit * 2.0 - 1.0
}
