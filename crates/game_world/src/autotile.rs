use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{SemanticTerrainGrid, TerrainCardinalMask, stable_terrain_hash};

#[derive(Debug, Clone, Default)]
pub struct TerrainResolveCatalog {
    pub seed: u64,
    pub variant_sets: HashMap<String, TerrainVariantSet>,
    pub transition_rules: Vec<TerrainTransitionRule>,
    pub terrain_flags: HashMap<String, TerrainFlags>,
}

#[derive(Debug, Clone, Default)]
pub struct TerrainFlags {
    pub walkable: bool,
    pub blocks_movement: bool,
    pub water: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TerrainVariantSet {
    pub terrain_id: String,
    pub fallback_tile_id: String,
    pub variants: Vec<TerrainVariantChoice>,
}

#[derive(Debug, Clone, Default)]
pub struct TerrainVariantChoice {
    pub tile_id: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Default)]
pub struct TerrainTransitionRule {
    pub id: String,
    pub from: String,
    pub to: String,
    pub render_layer: u8,
    pub fallback_tile_id: String,
    pub tiles_by_mask: HashMap<u32, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolvedTileKind {
    Base,
    Transition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTerrainTile {
    pub x: u32,
    pub y: u32,
    pub render_layer: u8,
    pub tile_id: String,
    pub source_terrain_id: String,
    pub target_terrain_id: Option<String>,
    pub mask: u32,
    pub collision_blocked: bool,
    pub kind: ResolvedTileKind,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolvedTerrainLayer {
    pub scene_id: String,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<ResolvedTerrainTile>,
}

pub struct AutotileResolver;

impl AutotileResolver {
    pub fn resolve(
        grid: &SemanticTerrainGrid,
        catalog: &TerrainResolveCatalog,
    ) -> ResolvedTerrainLayer {
        let mut tiles = Vec::with_capacity(grid.cells.len().saturating_mul(2));

        for y in 0..grid.height {
            for x in 0..grid.width {
                let Some(cell) = grid.cell(x as i32, y as i32) else {
                    continue;
                };

                let terrain_id = cell.terrain_id.as_str();
                let flags = catalog
                    .terrain_flags
                    .get(terrain_id)
                    .cloned()
                    .unwrap_or_default();
                let base_tile_id = select_base_tile_id(catalog, &grid.scene_id, terrain_id, x, y);

                tiles.push(ResolvedTerrainTile {
                    x,
                    y,
                    render_layer: 0,
                    tile_id: base_tile_id,
                    source_terrain_id: terrain_id.to_string(),
                    target_terrain_id: None,
                    mask: 0,
                    collision_blocked: flags.blocks_movement || flags.water,
                    kind: ResolvedTileKind::Base,
                });
            }
        }

        for y in 0..grid.height {
            for x in 0..grid.width {
                let Some(cell) = grid.cell(x as i32, y as i32) else {
                    continue;
                };
                let terrain_id = cell.terrain_id.as_str();

                for rule in catalog
                    .transition_rules
                    .iter()
                    .filter(|rule| rule.from.as_str() == terrain_id)
                {
                    let mask = TerrainCardinalMask::for_target(grid, x as i32, y as i32, &rule.to)
                        .bits as u32;
                    if mask == 0 {
                        continue;
                    }

                    let tile_id = transition_tile_for_mask(rule, mask);
                    tiles.push(ResolvedTerrainTile {
                        x,
                        y,
                        render_layer: rule.render_layer.max(1),
                        tile_id,
                        source_terrain_id: rule.from.clone(),
                        target_terrain_id: Some(rule.to.clone()),
                        mask,
                        collision_blocked: false,
                        kind: ResolvedTileKind::Transition,
                    });
                }
            }
        }

        tiles.sort_by_key(|tile| (tile.render_layer, tile.y, tile.x));

        ResolvedTerrainLayer {
            scene_id: grid.scene_id.clone(),
            width: grid.width,
            height: grid.height,
            tiles,
        }
    }
}

fn select_base_tile_id(
    catalog: &TerrainResolveCatalog,
    scene_id: &str,
    terrain_id: &str,
    x: u32,
    y: u32,
) -> String {
    let Some(set) = catalog.variant_sets.get(terrain_id) else {
        return terrain_id.to_string();
    };

    if set.variants.is_empty() {
        return set.fallback_tile_id.clone();
    }

    let total_weight = set
        .variants
        .iter()
        .map(|variant| variant.weight.max(1) as u64)
        .sum::<u64>()
        .max(1);
    let mut pick = stable_terrain_hash(catalog.seed, scene_id, terrain_id, x, y) % total_weight;

    for variant in &set.variants {
        let weight = variant.weight.max(1) as u64;
        if pick < weight {
            return variant.tile_id.clone();
        }
        pick -= weight;
    }

    set.fallback_tile_id.clone()
}

fn transition_tile_for_mask(rule: &TerrainTransitionRule, mask: u32) -> String {
    if let Some(tile_id) = rule.tiles_by_mask.get(&mask) {
        return tile_id.clone();
    }

    // Many early placeholder sheets only provide cardinal pieces. If a corner or
    // multi-direction mask does not exist yet, select a stable cardinal piece
    // from the mask before falling back to the rule's safest base tile.
    for bit in [1, 2, 4, 8] {
        if mask & bit != 0 {
            if let Some(tile_id) = rule.tiles_by_mask.get(&bit) {
                return tile_id.clone();
            }
        }
    }

    rule.fallback_tile_id.clone()
}
