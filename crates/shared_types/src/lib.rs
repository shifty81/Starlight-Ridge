use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StableId(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("content validation failed: {0}")]
    Validation(String),
}

// -----------------------------------------------------------------------------
// Phase 51 world graph, scene, sub-scene, and layer-stack contracts
// -----------------------------------------------------------------------------
//
// These types are intentionally engine/editor neutral. They describe the shared
// contract that the runtime, egui editor, web editor, and procedural generation
// pipeline should all speak. Actual tile painting, sprite rendering, and save
// systems can evolve without changing the high-level world topology.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldManifest {
    pub id: String,
    pub display_name: String,
    pub version: String,

    pub default_world_seed: u64,
    pub default_start_scene: String,
    pub calendar: WorldCalendarConfig,
    pub regions: Vec<RegionNode>,
    pub scenes: Vec<SceneNode>,
    pub notes: Vec<String>,
}

pub type WorldGraph = WorldManifest;

impl WorldManifest {
    pub fn scene(&self, scene_id: &str) -> Option<&SceneNode> {
        self.scenes.iter().find(|scene| scene.id == scene_id)
    }

    pub fn total_sub_scene_count(&self) -> usize {
        self.scenes.iter().map(|scene| scene.sub_scenes.len()).sum()
    }

    pub fn total_layer_count(&self) -> usize {
        self.scenes
            .iter()
            .map(|scene| {
                scene.layer_stack.layers.len()
                    + scene
                        .sub_scenes
                        .iter()
                        .map(|sub_scene| sub_scene.layer_stack.layers.len())
                        .sum::<usize>()
            })
            .sum()
    }

    pub fn validate_basic(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if self.id.trim().is_empty() {
            issues.push(ValidationIssue::error(
                "world_manifest",
                "World manifest id is empty.",
            ));
        }

        if self.scene(&self.default_start_scene).is_none() {
            issues.push(ValidationIssue::error(
                "world_manifest.default_start_scene",
                format!(
                    "Default start scene '{}' does not exist in the manifest.",
                    self.default_start_scene
                ),
            ));
        }

        let mut scene_ids = std::collections::BTreeSet::new();
        for scene in &self.scenes {
            if !scene_ids.insert(scene.id.as_str()) {
                issues.push(ValidationIssue::error(
                    format!("scene.{}", scene.id),
                    format!("Duplicate scene id '{}'.", scene.id),
                ));
            }
            issues.extend(scene.validate_basic());
        }

        for scene in &self.scenes {
            for connection in &scene.connections {
                if self.scene(&connection.target_scene).is_none() {
                    issues.push(ValidationIssue::error(
                        format!("scene.{}.connection.{}", scene.id, connection.id),
                        format!(
                            "Connection '{}' targets missing scene '{}'.",
                            connection.id, connection.target_scene
                        ),
                    ));
                }
            }
            for exit in &scene.exits {
                if self.scene(&exit.target_scene).is_none() {
                    issues.push(ValidationIssue::warning(
                        format!("scene.{}.exit.{}", scene.id, exit.id),
                        format!(
                            "Exit '{}' targets scene '{}' that is not currently in the manifest.",
                            exit.id, exit.target_scene
                        ),
                    ));
                }
            }
        }

        issues
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldCalendarConfig {
    pub day_length_real_minutes: u32,
    pub default_season_length_days: u32,
    pub sleep_required_to_advance_day: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionNode {
    pub id: String,
    pub display_name: String,
    pub kind: RegionKind,
    pub scene_ids: Vec<String>,
    pub unlock_rule: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RegionKind {
    Farmstead,
    Town,
    Coast,
    Forest,
    CaveNetwork,
    InteriorSet,
    EventSpace,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneNode {
    pub id: String,
    pub display_name: String,
    pub kind: SceneKind,
    pub region_id: String,
    pub seed: u64,
    pub template_id: String,
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub bake_target: String,
    pub generation: SceneGenerationConfig,
    pub layer_stack: LayerStack,
    pub exits: Vec<SceneExit>,
    pub connections: Vec<SceneConnection>,
    pub sub_scenes: Vec<SubSceneNode>,
    pub notes: Vec<String>,
}

impl SceneNode {
    pub fn validate_basic(&self) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        if self.id.trim().is_empty() {
            issues.push(ValidationIssue::error("scene", "Scene id is empty."));
        }
        if self.width == 0 || self.height == 0 {
            issues.push(ValidationIssue::error(
                format!("scene.{}", self.id),
                format!(
                    "Scene dimensions are invalid: {}x{}.",
                    self.width, self.height
                ),
            ));
        }
        if self.tile_size == 0 {
            issues.push(ValidationIssue::error(
                format!("scene.{}", self.id),
                "Scene tile size is zero.",
            ));
        }
        if self.layer_stack.layers.is_empty() {
            issues.push(ValidationIssue::error(
                format!("scene.{}.layers", self.id),
                "Scene has no layers.",
            ));
        }
        issues.extend(self.layer_stack.validate_basic(&self.id));

        for sub_scene in &self.sub_scenes {
            issues.extend(sub_scene.validate_basic(&self.id));
        }

        issues
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubSceneNode {
    pub id: String,
    pub display_name: String,
    pub kind: SceneKind,
    pub parent_scene_id: String,
    pub seed: u64,
    pub template_id: String,
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub bake_target: String,
    pub generation: SceneGenerationConfig,
    pub layer_stack: LayerStack,
    pub exits: Vec<SceneExit>,
    pub notes: Vec<String>,
}

impl SubSceneNode {
    pub fn validate_basic(&self, parent_scene_id: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        if self.parent_scene_id != parent_scene_id {
            issues.push(ValidationIssue::warning(
                format!("sub_scene.{}", self.id),
                format!(
                    "Sub-scene parent_scene_id '{}' does not match containing scene '{}'.",
                    self.parent_scene_id, parent_scene_id
                ),
            ));
        }
        if self.width == 0 || self.height == 0 {
            issues.push(ValidationIssue::error(
                format!("sub_scene.{}", self.id),
                format!(
                    "Sub-scene dimensions are invalid: {}x{}.",
                    self.width, self.height
                ),
            ));
        }
        issues.extend(self.layer_stack.validate_basic(&self.id));
        issues
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneGenerationConfig {
    pub source: GenerationSource,
    pub generated_revision: u32,
    pub editable_after_generation: bool,
    pub protected_policy_id: String,
    pub draft_output: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GenerationSource {
    Authored,
    ProceduralTemplate,
    HybridDraftBake,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneConnection {
    pub id: String,
    pub target_scene: String,
    pub description: String,
    pub unlock_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneExit {
    pub id: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub target_scene: String,
    pub target_spawn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerStack {
    pub id: String,
    pub display_name: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub layers: Vec<LayerDefinition>,
}

impl LayerStack {
    pub fn validate_basic(&self, owner_id: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        let mut layer_ids = std::collections::BTreeSet::new();
        for layer in &self.layers {
            if layer.id.trim().is_empty() {
                issues.push(ValidationIssue::error(
                    format!("{}.layers", owner_id),
                    "Layer id is empty.",
                ));
            }
            if !layer_ids.insert(layer.id.as_str()) {
                issues.push(ValidationIssue::error(
                    format!("{}.layers.{}", owner_id, layer.id),
                    format!("Duplicate layer id '{}'.", layer.id),
                ));
            }
            if layer.kind == LayerKind::Collision
                && layer.generation_policy == LayerGenerationPolicy::Authored
            {
                issues.push(ValidationIssue::info(
                    format!("{}.layers.{}", owner_id, layer.id),
                    "Collision layer is authored; consider deriving it from terrain/objects after the gameplay rules settle.",
                ));
            }
        }
        issues
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayerDefinition {
    pub id: String,
    pub display_name: String,
    pub kind: LayerKind,
    pub render_order: i32,
    pub visible_by_default: bool,
    pub editable: bool,
    pub generation_policy: LayerGenerationPolicy,
    pub protection: LayerProtection,
    pub storage_hint: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerKind {
    SemanticTerrain,
    BaseTerrain,
    TerrainVariation,
    TerrainTransitions,
    Water,
    WaterAnimation,
    ShorelineFoam,
    Paths,
    TilledSoil,
    WateredSoil,
    NaturalObjects,
    PlacedObjects,
    Buildings,
    Collision,
    Interactions,
    Spawns,
    Triggers,
    Exits,
    Lighting,
    AudioZones,
    LogicBindings,
    Decals,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayerGenerationPolicy {
    Generated,
    Authored,
    Protected,
    Derived,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LayerProtection {
    Regeneratable,
    PreserveOnRegenerate,
    DerivedRebuildable,
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtectedLayerPolicy {
    pub id: String,
    pub regeneratable_layers: Vec<String>,
    pub protected_layers: Vec<String>,
    pub derived_layers: Vec<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneratedSceneDraft {
    pub id: String,
    pub source_world_manifest: String,
    pub scene_id: String,
    pub scene_kind: SceneKind,
    pub seed: u64,
    pub template_id: String,
    pub width: u32,
    pub height: u32,
    pub tile_size: u32,
    pub layer_stack_id: String,
    pub semantic_layer_id: String,
    pub bake_contract_id: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SceneBakeContract {
    pub id: String,
    pub scene_id: String,
    pub source_draft: String,
    pub target_map_dir: String,
    pub overwrite_policy: BakeOverwritePolicy,
    pub generated_layers: Vec<String>,
    pub protected_layers: Vec<String>,
    pub derived_layers: Vec<String>,
    pub required_validation: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BakeOverwritePolicy {
    DraftOnly,
    GeneratedLayersOnly,
    FullSceneRequiresConfirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldgenEditorWorkflow {
    pub id: String,
    pub display_name: String,
    pub active_world_manifest: String,
    pub preview_panels: Vec<String>,
    pub bake_steps: Vec<String>,
    pub validation_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub target: String,
    pub message: String,
}

impl ValidationIssue {
    pub fn error(target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Error,
            target: target.into(),
            message: message.into(),
        }
    }

    pub fn warning(target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Warning,
            target: target.into(),
            message: message.into(),
        }
    }

    pub fn info(target: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: ValidationSeverity::Info,
            target: target.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}
