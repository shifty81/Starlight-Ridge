# Open-Source Audit for Phase 52

This audit identifies projects/libraries worth studying or using. Do not blindly copy large outside systems. Use them as references or dependencies only when they fit the project architecture and license requirements.

## `.vox` parsing

### `dot_vox`

- Purpose: Rust parser for MagicaVoxel `.vox` files.
- Useful for: loading `.vox` source assets, dimensions, palettes, voxel positions, materials.
- Notes: current maintained versions focus on newer MagicaVoxel files and do not support older `MATT` chunks in v3+; old assets may need to be reopened and resaved in current MagicaVoxel.
- Sources:
  - https://github.com/dust-engine/dot_vox
  - https://crates.io/crates/dot_vox

Recommendation: use or keep compatible with `dot_vox` unless the custom parser must remain minimal.

## Noise generation

### FastNoiseLite

- Purpose: portable noise library with many algorithms.
- Useful for: elevation, moisture, temperature, fertility, resource distribution, biome masks.
- Sources:
  - https://github.com/Auburn/FastNoiseLite
  - https://github.com/Auburn/FastNoiseLite/wiki

Recommendation: use the design patterns and consider direct Rust usage/port if dependency fit is clean. Noise should be wrapped behind the project's own `NoiseSource` trait so it can be replaced later.

## Wave Function Collapse / constraint generation

### mxgmn/WaveFunctionCollapse

- Purpose: canonical bitmap/tilemap WFC reference implementation.
- Useful for: local microstructure, village blocks, ruin patterns, shoreline detail, tile adjacency learning.
- Source:
  - https://github.com/mxgmn/WaveFunctionCollapse

### Rust crates/projects to inspect

- `wave-function-collapse`: https://crates.io/crates/wave-function-collapse
- `procedural_tilemaps_core`: https://docs.rs/procedural_tilemaps_core
- `kahuna`: https://github.com/OutOfTheVoid/kahuna

Recommendation: WFC should be used for local detail passes, not as the whole world generator.

## Tilemap architecture references

### `bevy_ecs_tilemap`

- Purpose: chunked tilemap rendering with sparse maps, layers, animations, isometric/hex support.
- Useful for: architecture inspiration around chunking, layer separation, animated tile rendering.
- Source:
  - https://github.com/StarArawn/bevy_ecs_tilemap

Recommendation: study for tilemap architecture. Do not migrate engines just to use it.

## Cellular material and weather simulation references

### Sandspiel

- Purpose: open-source Rust/WASM/WebGL falling-sand cellular automata toy/game.
- Useful for: understanding simple grid material reactions and browser-side simulation architecture.
- Source:
  - https://github.com/maxbittker/sandspiel

Recommendation: use as inspiration for water/lava/oil/snow cell rules. Do not import as-is.

## Hydrology/erosion references

### terrain-erosion-3-ways

- Purpose: educational hydraulic erosion implementations.
- Useful for: understanding sediment/water/terrain interactions during offline worldgen.
- Source:
  - https://github.com/dandrino/terrain-erosion-3-ways

### SimpleHydrology

- Purpose: procedural hydrology and streams/pools concepts.
- Useful for: river routing and basin formation concepts.
- Source:
  - https://github.com/weigert/SimpleHydrology

Recommendation: use hydrology ideas for generator passes, not necessarily runtime.

## PWA/mobile web references

- MDN PWA installability: https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Making_PWAs_installable
- web.dev web app manifest: https://web.dev/learn/pwa/web-app-manifest
- Chrome DevTools PWA debugging: https://developer.chrome.com/docs/devtools/progressive-web-apps

Recommendation: keep mobile editor integrated with LAN server first. Add installable PWA behavior after the mobile LAN editor is stable.
