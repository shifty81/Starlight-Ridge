# Starlight Ridge Phase 52A — Core Contracts + Docs Foundation

Phase 52A turns the Phase 52 master plan into concrete repo files without attempting the full world generator yet.

## Included

- Rust contract structs in `crates/game_worldgen/src/phase52_contracts.rs`.
- A Phase 52 manifest at `content/contracts/phase52_manifest.ron`.
- Canonical starter catalogs for biomes, materials, liquids, seasons, weather, map layers, worldgen presets, `.vox` assets, editor help, and validation.
- README files beside every new content contract folder.
- Documentation/wiki scaffolds carried forward from the master spec.
- Remaining mirror implementation notes kept in the zip.

## Not included yet

- Runtime liquid simulation.
- Snow/puddle derived layer execution.
- Real `.vox` baking.
- Worldgen execution against these contracts.
- Native/web editor panels that edit every new contract.

## Next implementation phase

`Starlight_Ridge_phase52b_layered_world_model_editor_foundation.zip`

That phase should wire the 10 authored layers and 4 derived layers into editor data, validation panels, and map serialization.
