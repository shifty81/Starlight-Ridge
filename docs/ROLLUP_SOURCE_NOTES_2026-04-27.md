# Starlight Ridge Source Rollup — 2026-04-27

This source rollup consolidates the uploaded `Starlight-Ridge-main.zip` tree with any missing files from `Starlight_Ridge_source_2026-04-27_07-31-41.zip` without overwriting newer files from the main tree.

Included focus areas from the current project line:

- egui editor shell and world/map editor foundation
- web editor/LAN helper scripts and asset lab tooling
- Phase 51 editor map/layer/pixel workflow documentation where present
- Phase 52 worldgen/material/biome/liquid/weather/season/voxel contract content
- current starter farm, town, autotile test maps, tile metadata, atlas assets, and generated assets

Validation note:

- I could not run `cargo check` in this container because Rust/Cargo is not installed here. Run `cargo check` locally after extracting.

Suggested next phase after testing:

- Phase 52f — Generated Scene Dry-Run Bake Preview
