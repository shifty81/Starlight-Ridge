# Starlight Ridge Phase 15 Art Contract Reset

Phase 15 stops trying to force partially matched art into guessed renderer rules.

## Terrain atlas contract

File:

```text
assets/textures/terrain_atlas_phase15_contract.png
```

Cell size:

```text
32x32
```

Atlas size:

```text
16 columns x 14 rows
```

Rules:

- Every terrain role has a named metadata entry in `content/tiles/base_tileset.ron`.
- Gameplay/render code resolves by tile id, not by guessed columns.
- Terrain variants are full-cell, opaque-safe sprites so renderer scaling does not expose transparent gutters.
- Path, soil, water, sand, cliff, stone, and wood variants are explicit named roles.

## Player sprite contract

File:

```text
assets/textures/player_walk.png
```

Frame size:

```text
32x48
```

Layout:

```text
4 columns x 5 rows
row 0 = front/down
row 1 = left
row 2 = right
row 3 = back/up
row 4 = interaction prompt
```

Rules:

- Frames are not packed or trimmed.
- Every animation frame uses the same 32x48 bounds.
- The player foot position is consistent across all frames.
- The interaction prompt is rendered from the same sheet as a temporary UI-safe marker.

## Map layer contract

Starter farm now uses:

```text
ground  = semantic terrain
objects = visual props/objects/foliage
props.ron = interaction/collision authority for named interactables
triggers.ron = transition/inspection zones
spawns.ron = player/NPC spawn data
```

## Gameplay path included

- WASD / arrow movement.
- Escape exits.
- E / Space logs the nearest interaction.
- Water, cliffs, trees, bushes, fences, farmhouse objects, well, and shipping bin block movement.
- A visible `!` prompt appears above the player when an interaction target is nearby.

## Known remaining work

This is a contract reset and stabilization pass, not a final art pass. Remaining work:

- Replace generated placeholder art with final hand-authored art.
- Add true roof/wall depth and y-sorted object rendering.
- Add real UI text prompts instead of the temporary prompt sprite.
- Add map transitions that actually load the target map.
- Add editor tools for painting semantic terrain and object layers.
