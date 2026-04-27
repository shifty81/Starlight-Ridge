# Content Authoring

All game content is defined in [RON](https://github.com/ron-rs/ron) files under `content/`. The `game_data` crate loads everything at startup into a `ContentRegistry`. You never need to touch Rust code to add most content types.

---

## Adding a new map

1. Create a directory under `content/maps/<your_map_id>/`.
2. Add the following files (copy from `content/maps/starter_farm/` as a template):

### `map.ron`
```ron
(
    id: "your_map_id",
    display_name: "Your Map Name",
    width: 40,
    height: 28,
    tileset: "base_tiles",
    music: Some("farm_day"),   // or None
    ambient_light: 0.88,
)
```

### `layers.ron`
Defines tile layers. Each layer has a `legend` mapping symbols to tile role names, and a `rows` grid of symbols.

```ron
(
    layers: [
        (
            id: "ground",
            label: "Ground",
            legend: {
                ".": "grass_base",
                "w": "shallow_water_base",
                "d": "dirt_base",
            },
            rows: [
                "........................................",
                // ... one string per row, length must equal map width
            ],
        ),
    ],
)
```

Tile role names must exist in the active tileset (see `content/tiles/base_tileset.ron` for the full list of 191 named roles).

### `props.ron`
```ron
[
    ( id: "farmhouse",  tile: "prop_farmhouse", x: 10, y: 6,  collidable: true  ),
    ( id: "shipping_bin", tile: "prop_shipping_bin", x: 14, y: 8, collidable: true ),
]
```

### `spawns.ron`
```ron
[
    ( id: "player_start", x: 20, y: 14 ),
]
```

### `triggers.ron`
```ron
[
    ( id: "to_town", x: 20, y: 0, width: 2, height: 1, target_map: "town", target_x: 15, target_y: 26 ),
]
```

3. Reload content in the editor (`Project > Reload Content`) or restart the game. The map appears in the map switcher automatically.

---

## Adding a new tileset

Tilesets live in `content/tiles/`. Each file is a `TilesetDef`:

```ron
(
    id: "base_tiles",
    texture: "terrain_atlas_phase15_contract.png",
    tile_width: 32,
    tile_height: 32,
    tiles: {
        "grass_base":           ( col: 0,  row: 0  ),
        "grass_edge_n":         ( col: 1,  row: 0  ),
        "water_edge_n":         ( col: 0,  row: 4  ),
        // ... one entry per named role
    },
)
```

The texture path is relative to `assets/textures/`. Tile coordinates are zero-based column/row indices into the atlas grid.

---

## Adding a new item

Items live in `content/items/`. Add a `.ron` file (one file per item, or one file with multiple items in a list — both are supported by the loader):

```ron
(
    id: "turnip_seed",
    display_name: "Turnip Seed",
    description: "Plant in tilled soil.",
    category: Seed,
    stack_max: 99,
    sell_price: Some(10),
    sprite: "item_turnip_seed",
)
```

---

## Adding a new crop

Crops live in `content/crops/`:

```ron
(
    id: "turnip",
    display_name: "Turnip",
    seed_item: "turnip_seed",
    harvest_item: "turnip",
    growth_days: 4,
    watered_stages: 4,
    seasons: [Spring, Summer],
)
```

---

## Adding a new NPC

### NPC definition — `content/npc/<id>.ron`
```ron
(
    id: "mira",
    display_name: "Mira",
    sprite_sheet: "npc_mira",
    home_map: "town",
    schedule: "mira_daily",
    dialogue: "mira_intro",
)
```

### Schedule — `content/schedules/<id>.ron`
```ron
(
    id: "mira_daily",
    entries: [
        ( time: "08:00", map: "town",       x: 12, y: 8  ),
        ( time: "12:00", map: "town_shop",  x: 4,  y: 6  ),
        ( time: "18:00", map: "town",       x: 12, y: 8  ),
    ],
)
```

---

## Adding dialogue

Dialogue trees live in `content/dialogue/`:

```ron
(
    id: "mira_intro",
    nodes: [
        (
            id: "start",
            speaker: "Mira",
            text: "Oh! A new face around here. Welcome to Starlight Ridge.",
            choices: [
                ( text: "Thanks!", next: "thanks" ),
                ( text: "Who are you?", next: "who" ),
            ],
        ),
        (
            id: "thanks",
            speaker: "Mira",
            text: "Any time. Come find me if you need anything.",
            choices: [],
        ),
    ],
)
```

---

## Adding a quest

Quests live in `content/quests/`:

```ron
(
    id: "first_harvest",
    display_name: "First Harvest",
    description: "Grow and harvest your first crop.",
    giver_npc: "mira",
    objectives: [
        ( id: "harvest_turnip", description: "Harvest 1 turnip.", item: "turnip", count: 1 ),
    ],
    reward_items: [
        ( item: "gold_coin", count: 100 ),
    ],
)
```

---

## Adding a shop

Shops live in `content/shops/`:

```ron
(
    id: "general_store",
    display_name: "General Store",
    keeper_npc: "shopkeeper_tom",
    stock: [
        ( item: "turnip_seed",  buy_price: 20,  sell_multiplier: 0.5 ),
        ( item: "watering_can", buy_price: 200, sell_multiplier: 0.4 ),
    ],
)
```

---

## Validation

Run `cargo check` after adding content to catch RON parse errors at load time. The editor also has a **Validation** panel (bottom bar) that reports missing tile refs, mismatched layer widths, and other content errors when content is reloaded.

---

## Content directory reference

```
content/
├── biomes/          # BiomePackDef — 12 planned biomes
├── contracts/       # Phase 52 manifest and schema contracts
├── crops/           # CropDef
├── dialogue/        # DialogueDef (tree nodes)
├── items/           # ItemDef
├── liquids/         # Liquid type contracts (Phase 52+)
├── maps/            # One folder per map: map.ron layers.ron props.ron spawns.ron triggers.ron
├── materials/       # Material family contracts (Phase 52+)
├── metadata/        # SpriteSheetDef, EditorAtlasPipelineDef, etc.
├── npc/             # NpcDef
├── quests/          # QuestDef
├── schedules/       # ScheduleDef
├── seasons/         # Season definitions
├── shops/           # ShopDef
├── terrain/         # TerrainTypeDef, TransitionSetDef, TerrainRulesetDef
├── tiles/           # TilesetDef (base + seasonal variants)
├── voxels/          # .vox source asset registry (Phase 52c+)
├── weather/         # Weather type contracts (Phase 52+)
└── worldgen/        # WorldGen preset contracts (Phase 52e+)
```
