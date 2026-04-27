use crate::defs::{
    BiomePackDef, CropDef, DialogueDef, EditorAtlasPipelineCatalogDef, EditorAnimationPipelineDef, EditorExportValidationPipelineDef, ItemDef, MapBundle,
    MapLayersDef, NpcDef, QuestDef, ScheduleDef, ShopDef, SpriteSheetDef, TerrainRulesetDef,
    TerrainTypeDef, TilesetDef, TransitionSetDef,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct ContentRegistry {
    pub root: PathBuf,
    pub items: HashMap<String, ItemDef>,
    pub crops: HashMap<String, CropDef>,
    pub npcs: HashMap<String, NpcDef>,
    pub schedules: HashMap<String, ScheduleDef>,
    pub dialogues: HashMap<String, DialogueDef>,
    pub quests: HashMap<String, QuestDef>,
    pub shops: HashMap<String, ShopDef>,
    pub maps: HashMap<String, MapBundle>,
    pub tilesets: HashMap<String, TilesetDef>,
    pub sprite_sheets: HashMap<String, SpriteSheetDef>,
    pub map_layers: HashMap<String, MapLayersDef>,
    pub terrain_types: HashMap<String, TerrainTypeDef>,
    pub biome_packs: HashMap<String, BiomePackDef>,
    pub transition_sets: HashMap<String, TransitionSetDef>,
    pub terrain_rulesets: HashMap<String, TerrainRulesetDef>,
    pub editor_atlas_pipelines: HashMap<String, EditorAtlasPipelineCatalogDef>,
    pub editor_export_pipelines: HashMap<String, EditorExportValidationPipelineDef>,
    pub editor_animation_pipelines: HashMap<String, EditorAnimationPipelineDef>,
}

impl ContentRegistry {
    pub fn summary(&self) -> String {
        format!(
            "items={} crops={} npcs={} schedules={} dialogues={} quests={} shops={} maps={} tilesets={} sprite_sheets={} map_layers={} terrain_types={} biome_packs={} transition_sets={} terrain_rulesets={} editor_atlas_pipelines={} editor_export_pipelines={} editor_animation_pipelines={}",
            self.items.len(),
            self.crops.len(),
            self.npcs.len(),
            self.schedules.len(),
            self.dialogues.len(),
            self.quests.len(),
            self.shops.len(),
            self.maps.len(),
            self.tilesets.len(),
            self.sprite_sheets.len(),
            self.map_layers.len(),
            self.terrain_types.len(),
            self.biome_packs.len(),
            self.transition_sets.len(),
            self.terrain_rulesets.len(),
            self.editor_atlas_pipelines.len(),
            self.editor_export_pipelines.len(),
            self.editor_animation_pipelines.len(),
        )
    }

    pub fn has_phase19_editor_pipeline(&self) -> bool {
        !self.editor_atlas_pipelines.is_empty()
    }

    pub fn has_phase20_editor_export_pipeline(&self) -> bool {
        !self.editor_export_pipelines.is_empty()
    }

    pub fn has_phase21_editor_animation_pipeline(&self) -> bool {
        !self.editor_animation_pipelines.is_empty()
    }

    pub fn has_phase17_terrain_contracts(&self) -> bool {
        !self.terrain_types.is_empty()
            || !self.biome_packs.is_empty()
            || !self.transition_sets.is_empty()
            || !self.terrain_rulesets.is_empty()
    }
}
