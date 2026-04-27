# Biome and Material Matrix

## Canonical 12 biomes

| ID | Biome | Primary purpose | Common materials | Special rules |
|---|---|---|---|---|
| `coastal_forest` | Coastal Forest | Starter-area flavor, beaches, saltwater edges | lush grass, coastal saltwater, wet shoreline sand | salt spray, driftwood, gulls, coastal shrubs |
| `temperate_woodland` | Temperate Woodland | Default forest biome | lush grass, meadow grass, freshwater | dense trees, mushrooms, shade moisture |
| `meadow_plains` | Meadow / Plains | Open buildable/farmable land | meadow grass, dry grass | flowers, high visibility, low tree density |
| `cultivated_farmland` | Farmland / Cultivated | Farms, player land, NPC farmland | tilled soil, lush grass, watered soil | crop-ready, irrigation, fertilizer support |
| `marsh` | Marsh | Wet transitional land | marsh grass, swamp water, mud | puddles, reeds, frogs/insects, slow movement |
| `swamp` | Swamp | Dense wet forest | marsh grass, swamp water, dark mud | dark water, high decay/poison potential |
| `taiga_pine` | Taiga / Pine Forest | Cold forest | alpine grass, icy water, snow overlay | snow retention, conifers |
| `tundra` | Tundra | Sparse cold land | tundra grass, icy water, snow | low vegetation, permafrost |
| `alpine_snowfield` | Alpine / Snowfield | Mountain/snow maps | snow, stone, icy water | heavy snow, steep slopes |
| `arid_scrub` | Arid Scrub | Dry semi-desert | dry grass, red sand, sparse shrubs | low moisture, dust weather |
| `desert_dune` | Desert / Dune | Hot dry maps | golden sand, pale sand | no puddle retention except basins, heat shimmer |
| `volcanic_ashlands` | Volcanic / Ashlands | High-risk biome | volcanic sand, lava, ash stone | lava, ashfall, fire hazard |

## Grass families

| ID | Display | Used by | Seasonal variants | Notes |
|---|---|---|---|---|
| `grass_lush` | Lush Grass | coastal forest, woodland, farmland | spring/summer/fall/winter | default healthy green |
| `grass_dry` | Dry Grass | arid scrub, dry plains | spring/summer/fall/winter | yellows/browns in summer |
| `grass_meadow` | Meadow Grass | plains, farmland edges | spring/summer/fall/winter | flower overlays allowed |
| `grass_marsh` | Marsh Grass | marsh, swamp | spring/summer/fall/winter | blends with mud/water |
| `grass_alpine` | Alpine/Tundra Grass | taiga, tundra, alpine | spring/summer/fall/winter | snow-cover compatible |

## Sand/soil families

| ID | Display | Used by | Notes |
|---|---|---|---|
| `sand_pale_beach` | Pale Beach Sand | coastal forest, beaches | wet variant required |
| `sand_golden_dune` | Golden Dune Sand | desert | wind/dune variants later |
| `sand_wet_shoreline` | Wet Shoreline Sand | coasts, ponds, rivers | strong water transition support |
| `sand_red_badlands` | Red/Badlands Sand | arid scrub | stone/rock blend support |
| `sand_dark_volcanic` | Dark Volcanic Sand | volcanic ashlands | lava edge support |

## Water families

| ID | Display | Used by | Animation | Physics profile |
|---|---|---|---|---|
| `water_clear_fresh` | Clear Freshwater | rivers, ponds | required | low viscosity, evaporates slowly |
| `water_coastal_salt` | Coastal Saltwater | ocean/beach | required | tide support later |
| `water_swamp` | Swamp Water | marsh/swamp | required | murky, slower, algae overlays |
| `water_deep` | Deep Water | ocean/lake depths | required | non-walkable, darkened |
| `water_icy` | Icy/Glacial Water | tundra/alpine | required | freezes easily |

## Special liquids

| ID | Display | Required behavior |
|---|---|---|
| `lava` | Lava | high viscosity, heat damage, ignition, cooling/crust rules later |
| `crude_oil` | Crude Oil | high viscosity, flammable, floats over water later, dark stain/wetness |

## Material asset requirements

Each material family must eventually provide:

- base tile set;
- transition tile/overlay set;
- season variants if exposed outdoors;
- wet/dry variants where relevant;
- animation frames where relevant;
- physics profile binding;
- editor thumbnail;
- validation rules;
- help tooltip metadata.
