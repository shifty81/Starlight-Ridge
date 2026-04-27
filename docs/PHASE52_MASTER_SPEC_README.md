# Starlight Ridge Phase 52 — Master Worldgen + VOX + Editor Spec Pack

Patch/spec name: `Starlight_Ridge_phase52_master_worldgen_vox_editor_spec.zip`

This is a canonical planning/specification pack for the next major Starlight Ridge direction:

- 12-biome procedural world generation.
- Modular material families: 5 grass, 5 sand/soil, 5 water, lava, crude oil.
- Layered editor/world model.
- Liquid/weather/snow simulation layers.
- `.vox` source-asset pipeline for pixel-voxel props and directional sprites.
- Orthographic orbit-style 2D camera with directional render support.
- In-editor help wiki, hover tooltips, validation rules, and README coverage.
- Phased implementation roadmap.

This pack is documentation-first. It is safe to extract over the project root because it only adds documentation, schema examples, and planning files under `docs/` and `content/contracts/`.

Recommended next implementation patch after this spec:

```text
Starlight_Ridge_phase52a_core_contracts_docs_foundation.zip
```

Phase 52a should turn these specs into compile-checked Rust data structs, RON loading, validation, and editor help metadata plumbing.
