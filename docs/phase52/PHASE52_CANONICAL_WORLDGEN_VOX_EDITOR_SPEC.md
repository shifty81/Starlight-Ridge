# Phase 52 Canonical Spec — Worldgen, Pixel Voxels, Simulation Layers, and Editor Help

## Intent

Phase 52 formalizes the architecture required for Starlight Ridge to support:

- richly varied procedural maps;
- multi-biome terrain generation;
- modular terrain/material/liquid definitions;
- puddling rain, flowing liquids, and snow buildup;
- `.vox` source assets baked into pixel-art runtime sprites;
- an orthographic orbit-style 2D camera;
- mobile LAN editor support;
- in-editor help/wiki documentation and field-level tooltips.

This is **not** a single atlas expansion. It is a content/runtime/editor contract reset.

## Core rule

Every visible thing in the world should be driven by a content contract, not hardcoded guesses.

That means:

- biome behavior comes from biome definitions;
- tile visuals come from material definitions;
- fluid behavior comes from liquid definitions;
- weather behavior comes from weather definitions;
- object visuals come from `.vox` or sprite asset manifests;
- editor UI fields are generated from schema/help metadata where possible;
- validation reports tell the user what is missing and why.

## Major subsystems

1. Material system
2. Biome system
3. World generation pipeline
4. Layered map/simulation model
5. Liquid/weather/snow simulation
6. `.vox` asset pipeline
7. Directional orthographic camera/rendering pipeline
8. Editor help/wiki/tooltip/validation system

## Non-goals for the first implementation pass

- Full CFD fluid simulation.
- Free-angle 3D orbit camera.
- Replacing the 2D runtime with a 3D voxel engine.
- Hand-authoring every tile variant before the contracts exist.
- A separate Android APK project before the LAN/web editor mode is stable.

## Recommended implementation model

Keep the game 2D and pixel-art rendered, but use `.vox` as a source-authoring format for props, structures, terrain features, and directional objects.

The runtime consumes baked 2D outputs:

```text
.vox source -> bake tool -> directional sprites + masks + metadata -> runtime/editor
```

The editor consumes both:

- source `.vox` files for inspection and rebake;
- baked sprites and metadata for placement, rendering, collision, and validation.
