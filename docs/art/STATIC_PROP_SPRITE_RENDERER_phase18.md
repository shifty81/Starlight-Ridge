# Phase 18 Static Prop Sprite Renderer

This patch makes selected `props.ron` entries visible without baking them into the terrain atlas.

## Runtime path

The game now builds two sprite layers:

1. Player layer:
   - metadata: `content/metadata/entity_sprite_sheet_phase5.ron`
   - texture: `assets/textures/player_walk.png`

2. Static prop layer:
   - metadata: `content/metadata/oceans_heart_bridge_phase17.ron`
   - texture: `assets/textures/oceans_heart_bridge_phase17.png`

The renderer draws in this order:

```text
tile map
static props
player
```

## Supported prop kinds

The first Phase 18 bridge kinds are:

```text
seagull
seagull_idle
seagull_flap_up
seagull_flap_down
seagull_glide
weak_tree
weak_tree_full
weak_tree_sapling
big_stone
big_stone_large
big_stone_small
explosive_barrel
wooden_crate
crate
root_small
root_cluster
driftwood
driftwood_log
steam_puff
steam_puff_small
impact_puff
enemy_killed
red_spark_puff
fireball_red_small
coastal_shrub
shrub
```

Unknown prop kinds are skipped so existing placeholder props such as `farmhouse_placeholder`, `shipping_bin`, `well`, and `sign` do not break rendering.

## Example

```ron
[
    (id: "coast_seagull_01", kind: "seagull", x: 8, y: 6),
    (id: "coast_weak_tree_01", kind: "weak_tree_full", x: 6, y: 16),
    (id: "coast_stone_01", kind: "big_stone_large", x: 21, y: 17),
    (id: "coast_driftwood_01", kind: "driftwood_log", x: 29, y: 21),
    (id: "coast_shrub_01", kind: "coastal_shrub", x: 12, y: 18),
]
```

## Not included yet

- Prop animation.
- Collision from props.
- Interactions/tool hits.
- Per-prop custom sprite size.
- Sorting by Y/depth.

This is intentionally a small visual/runtime bridge so assets can appear in-game before deeper entity and interaction systems are added.
