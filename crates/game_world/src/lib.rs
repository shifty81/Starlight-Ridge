pub mod autotile;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMetadata {
    pub id: String,
    pub display_name: String,
    pub width: u32,
    pub height: u32,
    pub tileset: String,
    pub music: Option<String>,
    pub ambient_light: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropPlacement {
    pub id: String,
    pub kind: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnPoint {
    pub id: String,
    pub kind: String,
    pub ref_id: Option<String>,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerZone {
    pub id: String,
    pub kind: String,
    pub target_map: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug, Clone)]
pub struct WorldBootstrap {
    pub active_map_id: String,
}

impl WorldBootstrap {
    pub fn new(active_map_id: impl Into<String>) -> Self {
        let bootstrap = Self {
            active_map_id: active_map_id.into(),
        };
        log::info!("world bootstrap active map: {}", bootstrap.active_map_id);
        bootstrap
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DebugColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl DebugColor {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone)]
pub struct DebugRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: DebugColor,
}

#[derive(Debug, Clone)]
pub struct DebugMapView {
    pub map_id: String,
    pub map_width: u32,
    pub map_height: u32,
    pub tile_size: f32,
    pub tiles: Vec<DebugRect>,
    pub overlays: Vec<DebugRect>,
}

impl DebugMapView {
    pub fn from_map_parts(
        metadata: &MapMetadata,
        props: &[PropPlacement],
        spawns: &[SpawnPoint],
        triggers: &[TriggerZone],
    ) -> Self {
        let tile_size = 32.0;
        let mut tiles = Vec::with_capacity((metadata.width * metadata.height) as usize);
        let origin_x = -(metadata.width as f32 * tile_size) * 0.5;
        let origin_y = -(metadata.height as f32 * tile_size) * 0.5;

        for y in 0..metadata.height {
            for x in 0..metadata.width {
                let band = ((x / 8) + (y / 8)) % 2;
                let color = if band == 0 {
                    DebugColor::new(0.16, 0.22, 0.18, 1.0)
                } else {
                    DebugColor::new(0.19, 0.26, 0.21, 1.0)
                };
                tiles.push(DebugRect {
                    x: origin_x + x as f32 * tile_size,
                    y: origin_y + y as f32 * tile_size,
                    w: tile_size,
                    h: tile_size,
                    color,
                });
            }
        }

        let mut overlays = Vec::new();
        for prop in props {
            overlays.push(DebugRect {
                x: origin_x + prop.x as f32 * tile_size,
                y: origin_y + prop.y as f32 * tile_size,
                w: tile_size,
                h: tile_size,
                color: DebugColor::new(0.80, 0.62, 0.18, 0.95),
            });
        }
        for spawn in spawns {
            overlays.push(DebugRect {
                x: origin_x + spawn.x as f32 * tile_size + tile_size * 0.2,
                y: origin_y + spawn.y as f32 * tile_size + tile_size * 0.2,
                w: tile_size * 0.6,
                h: tile_size * 0.6,
                color: DebugColor::new(0.20, 0.72, 0.96, 0.98),
            });
        }
        for trigger in triggers {
            overlays.push(DebugRect {
                x: origin_x + trigger.x as f32 * tile_size,
                y: origin_y + trigger.y as f32 * tile_size,
                w: trigger.w.max(1) as f32 * tile_size,
                h: trigger.h.max(1) as f32 * tile_size,
                color: DebugColor::new(0.92, 0.25, 0.30, 0.28),
            });
        }

        Self {
            map_id: metadata.id.clone(),
            map_width: metadata.width,
            map_height: metadata.height,
            tile_size,
            tiles,
            overlays,
        }
    }

    pub fn world_center(&self) -> glam::Vec2 {
        glam::Vec2::ZERO
    }
}

/// Canonical terrain roles used by the renderer contract.
///
/// The map layer can use short symbols, but every symbol must resolve to one of
/// these roles through an explicit tile id. This prevents atlas-coordinate drift
/// where a water symbol can accidentally render as dirt, cliff, flowers, or a
/// random transition tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileRole {
    GrassBright,
    GrassDark,
    Dirt,
    PathSand,
    TilledDry,
    TilledWatered,
    StoneFloor,
    WoodFloor,
    Sand,
    WaterShallow,
    WaterDeep,
    Cliff,
    FlowersYellow,
    FlowersPink,
    FlowersPurple,
    Bush,
    TreeRound,
    TreePine,
    TreeLeafy,
    Stump,
    BranchRock,
    FenceH,
    FenceV,
    StoneWall,
    Unknown,
}

impl TileRole {
    pub fn from_tile_id(tile_id: &str) -> Self {
        match tile_id {
            "grass_bright"
            | "grass_bright_flower_1"
            | "grass_bright_sprout_1"
            | "grass_bright_alt" => Self::GrassBright,
            "grass_dark" | "grass_dark_flower_1" | "grass_dark_sprout_1" | "grass_dark_alt" => {
                Self::GrassDark
            }
            "dirt" | "dirt_pebbles_1" | "dirt_sprouts_1" | "dirt_alt" => Self::Dirt,
            "path_sand" | "path_sand_pebbles_1" | "path_sand_sprouts_1" | "path_sand_alt" => {
                Self::PathSand
            }
            "tilled_dry" | "tilled_dry_pebbles_1" | "tilled_dry_sprouts_1" | "tilled_dry_edge" => {
                Self::TilledDry
            }
            "tilled_watered"
            | "tilled_watered_sprouts_1"
            | "tilled_watered_sprouts_2"
            | "tilled_watered_edge" => Self::TilledWatered,
            "stone_floor" | "stone_floor_moss_1" | "stone_floor_moss_2" | "stone_floor_alt" => {
                Self::StoneFloor
            }
            "wood_floor" | "wood_floor_scuffed_1" | "wood_floor_sprouts_1" | "wood_floor_alt" => {
                Self::WoodFloor
            }
            "sand" | "sand_pebbles_1" | "sand_sprouts_1" | "sand_alt" => Self::Sand,
            "water_shallow"
            | "water_shallow_rocks_1"
            | "water_shallow_rocks_2"
            | "water_shallow_bank" => Self::WaterShallow,
            "water_deep" | "water_deep_rocks_1" | "water_deep_rocks_2" | "water_deep_bank" => {
                Self::WaterDeep
            }
            "cliff" | "cliff_grass_cap" | "cliff_rock_face" | "cliff_alt" => Self::Cliff,
            "flowers_yellow" => Self::FlowersYellow,
            "flowers_pink" => Self::FlowersPink,
            "flowers_purple" => Self::FlowersPurple,
            "bush" => Self::Bush,
            "tree_round" => Self::TreeRound,
            "tree_pine" => Self::TreePine,
            "tree_leafy" => Self::TreeLeafy,
            "stump" => Self::Stump,
            "branch_rock" => Self::BranchRock,
            "fence_h" => Self::FenceH,
            "fence_v" => Self::FenceV,
            "stone_wall" => Self::StoneWall,
            _ => Self::Unknown,
        }
    }

    pub fn base_tile_id(self) -> &'static str {
        match self {
            Self::GrassBright => "grass_bright",
            Self::GrassDark => "grass_dark",
            Self::Dirt => "dirt",
            Self::PathSand => "path_sand",
            Self::TilledDry => "tilled_dry",
            Self::TilledWatered => "tilled_watered",
            Self::StoneFloor => "stone_floor",
            Self::WoodFloor => "wood_floor",
            Self::Sand => "sand",
            Self::WaterShallow => "water_shallow",
            Self::WaterDeep => "water_deep",
            Self::Cliff => "cliff",
            Self::FlowersYellow => "flowers_yellow",
            Self::FlowersPink => "flowers_pink",
            Self::FlowersPurple => "flowers_purple",
            Self::Bush => "bush",
            Self::TreeRound => "tree_round",
            Self::TreePine => "tree_pine",
            Self::TreeLeafy => "tree_leafy",
            Self::Stump => "stump",
            Self::BranchRock => "branch_rock",
            Self::FenceH => "fence_h",
            Self::FenceV => "fence_v",
            Self::StoneWall => "stone_wall",
            Self::Unknown => "grass_bright",
        }
    }

    fn same_transition_group(self, other: Self) -> bool {
        match (self, other) {
            (Self::WaterShallow | Self::WaterDeep, Self::WaterShallow | Self::WaterDeep) => true,
            (Self::TilledDry | Self::TilledWatered, Self::TilledDry | Self::TilledWatered) => true,
            _ => self == other,
        }
    }

    fn resolved_tile_id(self, mask: NeighborMask, x: u32, y: u32) -> &'static str {
        let variant = terrain_hash(x, y) % 5;
        match self {
            Self::GrassBright => match variant {
                0 => "grass_bright_flower_1",
                1 => "grass_bright_sprout_1",
                2 => "grass_bright_alt",
                _ => "grass_bright",
            },
            Self::GrassDark => match variant {
                0 => "grass_dark_flower_1",
                1 => "grass_dark_sprout_1",
                2 => "grass_dark_alt",
                _ => "grass_dark",
            },
            Self::Dirt => match variant {
                0 => "dirt_pebbles_1",
                1 => "dirt_sprouts_1",
                2 => "dirt_alt",
                _ => "dirt",
            },
            Self::PathSand => match variant {
                0 => "path_sand_pebbles_1",
                1 => "path_sand_sprouts_1",
                2 => "path_sand_alt",
                _ => "path_sand",
            },
            Self::TilledDry => {
                if !mask.all_same() {
                    "tilled_dry_edge"
                } else if variant == 0 {
                    "tilled_dry_pebbles_1"
                } else if variant == 1 {
                    "tilled_dry_sprouts_1"
                } else {
                    "tilled_dry"
                }
            }
            Self::TilledWatered => {
                if !mask.all_same() {
                    "tilled_watered_edge"
                } else if variant == 0 {
                    "tilled_watered_sprouts_1"
                } else if variant == 1 {
                    "tilled_watered_sprouts_2"
                } else {
                    "tilled_watered"
                }
            }
            Self::StoneFloor => match variant {
                0 => "stone_floor_moss_1",
                1 => "stone_floor_moss_2",
                2 => "stone_floor_alt",
                _ => "stone_floor",
            },
            Self::WoodFloor => match variant {
                0 => "wood_floor_scuffed_1",
                1 => "wood_floor_sprouts_1",
                2 => "wood_floor_alt",
                _ => "wood_floor",
            },
            Self::Sand => match variant {
                0 => "sand_pebbles_1",
                1 => "sand_sprouts_1",
                2 => "sand_alt",
                _ => "sand",
            },
            Self::WaterShallow => {
                if !mask.all_same() {
                    "water_shallow_bank"
                } else if variant == 0 {
                    "water_shallow_rocks_1"
                } else if variant == 1 {
                    "water_shallow_rocks_2"
                } else {
                    "water_shallow"
                }
            }
            Self::WaterDeep => {
                if !mask.all_same() {
                    "water_deep_bank"
                } else if variant == 0 {
                    "water_deep_rocks_1"
                } else if variant == 1 {
                    "water_deep_rocks_2"
                } else {
                    "water_deep"
                }
            }
            Self::Cliff => match variant {
                0 => "cliff_grass_cap",
                1 => "cliff_rock_face",
                2 => "cliff_alt",
                _ => "cliff",
            },
            Self::Unknown => self.base_tile_id(),
            _ => self.base_tile_id(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NeighborMask {
    pub north: bool,
    pub east: bool,
    pub south: bool,
    pub west: bool,
}

impl NeighborMask {
    pub fn all_same(self) -> bool {
        self.north && self.east && self.south && self.west
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedSemanticTile {
    pub x: u32,
    pub y: u32,
    pub source_tile_id: String,
    pub resolved_tile_id: String,
    pub role: TileRole,
    pub neighbors: NeighborMask,
}

pub struct SemanticTerrainResolver;

impl SemanticTerrainResolver {
    pub fn resolve_layer(
        width: u32,
        height: u32,
        rows: &[String],
        legend: &[(String, String)],
    ) -> anyhow::Result<Vec<ResolvedSemanticTile>> {
        let width_usize = width as usize;
        let height_usize = height as usize;
        anyhow::ensure!(
            rows.len() == height_usize,
            "layer row count mismatch: expected {} rows, found {}",
            height,
            rows.len()
        );

        let legend_map = legend
            .iter()
            .map(|(symbol, tile_id)| (symbol.clone(), tile_id.clone()))
            .collect::<HashMap<_, _>>();

        let mut source_ids = Vec::with_capacity(width_usize * height_usize);
        let mut roles = Vec::with_capacity(width_usize * height_usize);

        for (row_index, row) in rows.iter().enumerate() {
            let symbols = row.chars().map(|ch| ch.to_string()).collect::<Vec<_>>();
            anyhow::ensure!(
                symbols.len() == width_usize,
                "layer row {} width mismatch: expected {} symbols, found {}",
                row_index,
                width,
                symbols.len()
            );

            for symbol in symbols {
                let tile_id = legend_map
                    .get(&symbol)
                    .ok_or_else(|| anyhow::anyhow!("layer uses unmapped tile symbol '{}'", symbol))?
                    .clone();
                let role = TileRole::from_tile_id(&tile_id);
                anyhow::ensure!(
                    role != TileRole::Unknown,
                    "tile id '{}' has no strict TileRole mapping",
                    tile_id
                );
                source_ids.push(tile_id);
                roles.push(role);
            }
        }

        let mut resolved = Vec::with_capacity(width_usize * height_usize);
        for y in 0..height_usize {
            for x in 0..width_usize {
                let idx = y * width_usize + x;
                let role = roles[idx];
                let neighbors = NeighborMask {
                    north: y > 0 && role.same_transition_group(roles[(y - 1) * width_usize + x]),
                    east: x + 1 < width_usize
                        && role.same_transition_group(roles[y * width_usize + x + 1]),
                    south: y + 1 < height_usize
                        && role.same_transition_group(roles[(y + 1) * width_usize + x]),
                    west: x > 0 && role.same_transition_group(roles[y * width_usize + x - 1]),
                };

                resolved.push(ResolvedSemanticTile {
                    x: x as u32,
                    y: y as u32,
                    source_tile_id: source_ids[idx].clone(),
                    resolved_tile_id: role
                        .resolved_tile_id(neighbors, x as u32, y as u32)
                        .to_string(),
                    role,
                    neighbors,
                });
            }
        }

        Ok(resolved)
    }
}

fn terrain_hash(x: u32, y: u32) -> u32 {
    let mut value = x
        .wrapping_mul(0x45d9f3b)
        .wrapping_add(y.wrapping_mul(0x119de1f3));
    value ^= value >> 16;
    value = value.wrapping_mul(0x45d9f3b);
    value ^= value >> 16;
    value
}

// -----------------------------------------------------------------------------
// Phase 17 terrain contract support
// -----------------------------------------------------------------------------
//
// These runtime-side terrain primitives are intentionally semantic. They do not
// reference atlas coordinates and they do not replace the current renderer path.
// The renderer can keep drawing existing MapLayersDef data while game_data loads
// the terrain contracts that the future biome/PCG/autotile resolver will consume.

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerrainId(pub String);

impl TerrainId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerrainLayerRole {
    BaseGround,
    Water,
    Soil,
    TransitionOverlay,
    ObjectOverlay,
    CollisionOnly,
}

impl TerrainLayerRole {
    pub fn from_id(id: &str) -> Self {
        match id {
            "base_ground" | "ground" => Self::BaseGround,
            "water" => Self::Water,
            "soil" | "farm_soil" => Self::Soil,
            "transition_overlay" | "transition" => Self::TransitionOverlay,
            "object_overlay" | "object" | "overlay" => Self::ObjectOverlay,
            "collision_only" | "collision" => Self::CollisionOnly,
            _ => Self::BaseGround,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTerrainCell {
    pub terrain_id: String,
    pub biome_id: Option<String>,
    pub pcg_locked: bool,
}

impl SemanticTerrainCell {
    pub fn new(terrain_id: impl Into<String>) -> Self {
        Self {
            terrain_id: terrain_id.into(),
            biome_id: None,
            pcg_locked: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticTerrainGrid {
    pub scene_id: String,
    pub width: u32,
    pub height: u32,
    pub cells: Vec<SemanticTerrainCell>,
}

impl SemanticTerrainGrid {
    pub fn new(
        scene_id: impl Into<String>,
        width: u32,
        height: u32,
        cells: Vec<SemanticTerrainCell>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            cells.len() == (width as usize).saturating_mul(height as usize),
            "semantic terrain grid cell count mismatch: expected {} cells for {}x{}, found {}",
            (width as usize).saturating_mul(height as usize),
            width,
            height,
            cells.len()
        );

        Ok(Self {
            scene_id: scene_id.into(),
            width,
            height,
            cells,
        })
    }

    pub fn index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }

    pub fn cell(&self, x: i32, y: i32) -> Option<&SemanticTerrainCell> {
        if x < 0 || y < 0 {
            return None;
        }
        self.index(x as u32, y as u32)
            .and_then(|index| self.cells.get(index))
    }

    pub fn terrain_at(&self, x: i32, y: i32) -> Option<&str> {
        self.cell(x, y).map(|cell| cell.terrain_id.as_str())
    }

    pub fn set_terrain(
        &mut self,
        x: u32,
        y: u32,
        terrain_id: impl Into<String>,
    ) -> anyhow::Result<()> {
        let index = self.index(x, y).ok_or_else(|| {
            anyhow::anyhow!("semantic terrain write out of bounds at {},{}", x, y)
        })?;
        if let Some(cell) = self.cells.get_mut(index) {
            cell.terrain_id = terrain_id.into();
        }
        Ok(())
    }
}

/// Cardinal terrain neighbor mask used by the first transition pass.
///
/// Bits:
/// N=1, E=2, S=4, W=8
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerrainCardinalMask {
    pub bits: u8,
}

impl TerrainCardinalMask {
    pub const NORTH: u8 = 1;
    pub const EAST: u8 = 2;
    pub const SOUTH: u8 = 4;
    pub const WEST: u8 = 8;

    pub fn for_target(grid: &SemanticTerrainGrid, x: i32, y: i32, target_terrain: &str) -> Self {
        let mut bits = 0;
        if grid.terrain_at(x, y - 1) == Some(target_terrain) {
            bits |= Self::NORTH;
        }
        if grid.terrain_at(x + 1, y) == Some(target_terrain) {
            bits |= Self::EAST;
        }
        if grid.terrain_at(x, y + 1) == Some(target_terrain) {
            bits |= Self::SOUTH;
        }
        if grid.terrain_at(x - 1, y) == Some(target_terrain) {
            bits |= Self::WEST;
        }
        Self { bits }
    }

    pub fn is_empty(self) -> bool {
        self.bits == 0
    }
}

/// 8-bit blob mask used by future 47-tile/quarter-tile shoreline solving.
///
/// Bits:
/// N=1, NE=2, E=4, SE=8, S=16, SW=32, W=64, NW=128.
/// Diagonal bits are only set when both adjacent cardinal cells also match.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TerrainBlobMask {
    pub bits: u8,
}

impl TerrainBlobMask {
    pub fn for_same(grid: &SemanticTerrainGrid, x: i32, y: i32, terrain: &str) -> Self {
        let n = grid.terrain_at(x, y - 1) == Some(terrain);
        let e = grid.terrain_at(x + 1, y) == Some(terrain);
        let s = grid.terrain_at(x, y + 1) == Some(terrain);
        let w = grid.terrain_at(x - 1, y) == Some(terrain);

        let ne = n && e && grid.terrain_at(x + 1, y - 1) == Some(terrain);
        let se = s && e && grid.terrain_at(x + 1, y + 1) == Some(terrain);
        let sw = s && w && grid.terrain_at(x - 1, y + 1) == Some(terrain);
        let nw = n && w && grid.terrain_at(x - 1, y - 1) == Some(terrain);

        let mut bits = 0;
        if n {
            bits |= 1;
        }
        if ne {
            bits |= 2;
        }
        if e {
            bits |= 4;
        }
        if se {
            bits |= 8;
        }
        if s {
            bits |= 16;
        }
        if sw {
            bits |= 32;
        }
        if w {
            bits |= 64;
        }
        if nw {
            bits |= 128;
        }

        Self { bits }
    }
}

/// Seed-stable terrain hash for deterministic base-variant selection.
///
/// This is deliberately local and dependency-free so PCG, editor preview, and
/// runtime resolving can share the same result for the same seed/scene/cell.
pub fn stable_terrain_hash(seed: u64, scene_id: &str, terrain_id: &str, x: u32, y: u32) -> u64 {
    let mut hash = 1469598103934665603u64 ^ seed;

    for byte in scene_id.bytes().chain(terrain_id.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }

    hash ^= x as u64;
    hash = hash.wrapping_mul(1099511628211);
    hash ^= y as u64;
    hash = hash.wrapping_mul(1099511628211);

    hash
}
