# Starlight Ridge Phase 51 — World Graph / Scene Layer Architecture

## Purpose

Phase 51 stabilizes the project around one shared world structure instead of letting the runtime maps, editor tabs, and worldgen prototypes drift independently.

The chosen architecture is:

```text
WorldManifest
  RegionNode
    SceneNode
      SubSceneNode
        LayerStack
```

The world generator should create and validate the world graph first, then generate individual scene drafts from stable scene seeds/templates. The project should not generate one monolithic world tilemap and slice it afterward.

## Implemented in this patch

- Added shared world/scene/layer contracts in `crates/shared_types`.
- Added `WorldManifest`, `RegionNode`, `SceneNode`, `SubSceneNode`, `LayerStack`, layer generation/protection policies, `GeneratedSceneDraft`, `SceneBakeContract`, and validation issue structs.
- Added phase51 worldgen/content registry loading in `game_data`.
- Added phase51 registry validation for world manifests, workflows, drafts, and bake contracts.
- Added `content/worldgen/world_manifest_phase51.ron`.
- Added `content/worldgen/protected_layer_policies/default_scene_layer_policy_phase51.ron`.
- Added initial `starter_farm_draft.ron` and `starter_farm_bake_contract.ron` so the registry validates the draft/bake pipeline immediately.
- Added `content/editor_worldgen/worldgen_editor_workflow_phase51.ron`.
- Added a native egui `World > WorldGen` panel.
- Added `Generate draft` actions in the egui WorldGen panel, which write:
  - semantic generated scene RON to `content/worldgen/generated_semantic_scenes/`
  - generated draft records to `content/worldgen/generated_drafts/`
  - bake contracts to `content/worldgen/bake_contracts/`
- Fixed `content/maps/starter_farm/layers.ron` so the `ground` layer is visible by default.
- Added worldgen contract counts to the Project Overview panel.

## Important behavior

Scene generation is now treated as draft-first:

1. Select scene in `World > WorldGen`.
2. Generate semantic draft.
3. Store draft metadata.
4. Store bake contract.
5. Later phase runs autotile resolver.
6. Bake generated layers only while preserving authored/protected layers.

## Next patch target

Recommended next step:

```text
Phase 52 — Scene Draft Preview + Autotile Bake Handoff
```

Focus:

- preview generated semantic scene inside egui
- map semantic terrain to terrain catalog IDs
- hand draft to autotile resolver
- bake base/transition/water layers into editable `layers.ron`
- preserve authored/protected layers
- add validation issue click-to-focus behavior
