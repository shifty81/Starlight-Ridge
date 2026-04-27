# Starlight Ridge Phase 21 - Transition Autotiler Activation

This patch wires the previously unused local transition helpers into the active legacy terrain rendering path.

## Runtime behavior

- `starter_farm` is again the preferred launch map when present.
- `autotile_test_coast` remains available as a fallback diagnostic map.
- Contract semantic layers still use `game_world::autotile::AutotileResolver`.
- Legacy terrain layers use a conservative second pass that adds transition overlays after base terrain cells.
- Layers with `object`, `prop`, or `decor` in the id remain outside the terrain auto-tiler.

## Transition pairs activated

- Grass/dirt to path
- Grass/dirt to tilled dry soil
- Tilled dry soil to tilled watered soil
- Grass to sand
- Sand to shallow water
- Shallow water to deep water

## Not included

This patch does not spawn static props from `props.ron`. That remains a separate renderer/data hookup.
