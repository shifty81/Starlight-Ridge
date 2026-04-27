# Phase 51i — Hybrid 2D/3D World Authoring Scaffold

This patch pivots Starlight Ridge toward a hybrid world model:

- gameplay remains grid-readable and compatible with the existing 2D map editor;
- maps can now carry optional height/elevation data;
- maps can now carry optional 3D scene/object placement data;
- maps can now carry optional presentation/camera profiles for 2D, 2.5D, and 3D modes;
- maps can now carry optional lighting profiles for day/evening/night/weather previews;
- the egui editor gains World subtabs for Height / Elevation, 3D Scene Layout, Camera / Presentation, Lighting / Time, Weather, and WorldGen;
- the egui editor gains a Render main tab scaffold for 3D Viewport, Scene Preview, Sprite Bake, Icon Bake, Lighting Studio, Camera Presets, and Render Jobs.

The current stable 2D renderer is not replaced. This patch adds the data contracts and editor surfaces required to evolve toward Animal Crossing/Core Keeper style 2.5D/3D presentation while preserving simple farming-game logic.

## New content files

- `content/maps/starter_farm/height.ron`
- `content/maps/starter_farm/scene3d.ron`
- `content/maps/starter_farm/presentation.ron`
- `content/maps/starter_farm/lighting.ron`
- `content/editor_hybrid_world/hybrid_world_pipeline_phase51i.ron`

## New registry contracts

- `HeightMapDef`
- `Scene3DDef`
- `SceneObject3DDef`
- `PresentationDef`
- `CameraProfileDef`
- `LightingProfileDef`
- `HybridWorldEditorPipelineDef`

## What this enables next

Recommended follow-up patches:

1. Command Runner + Bash / External Tool Settings.
2. Internal Render → 3D Viewport scaffold with grid/orbit camera.
3. MagicaVoxel / Blockbench / Blender bridge workflows.
4. Sprite and icon bake job automation.
5. Actual 2.5D scene preview rendering from `layers.ron + height.ron + scene3d.ron + presentation.ron`.
