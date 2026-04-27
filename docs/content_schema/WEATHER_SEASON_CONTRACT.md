# Weather and Season Contract

## Weather profile purpose

Weather profiles describe probabilities, intensity, and simulation writes for rain, snow, fog, wind, ashfall, heat waves, and storms.

## Season profile purpose

Season profiles define long-term visuals and environment rules: temperature ranges, snow chance, crop rules, daylight balance, and material variant selection.

## Weather fields

| Field | Type | Purpose |
|---|---|---|
| `id` | string | Stable weather id. |
| `display_name` | string | User-facing name. |
| `kind` | enum | clear, rain, storm, snow, blizzard, fog, ashfall, heatwave. |
| `intensity_range` | range | Min/max intensity. |
| `duration_minutes_range` | range | Runtime duration. |
| `wetness_delta` | number | How much wetness is added. |
| `puddle_delta` | number | How much liquid depth can be added. |
| `snow_delta` | number | How much snow depth is added. |
| `wind_strength` | number | Used later for particles/leaves/snow drift. |
| `temperature_delta` | number | Local temperature adjustment. |
| `visibility_modifier` | number | Fog/storm visibility. |
| `ambience_id` | string | Audio/visual ambience. |
| `help_doc_id` | string | Editor wiki link. |

## Season fields

| Field | Type | Purpose |
|---|---|---|
| `id` | string | spring/summer/fall/winter or custom. |
| `temperature_modifier` | number | Baseline temperature offset. |
| `daylight_scale` | number | Later day/night tuning. |
| `snow_enabled` | bool | Whether snow can accumulate. |
| `freeze_enabled` | bool | Whether water can freeze. |
| `material_variant_key` | string | Variant key used by material resolver. |
| `weather_weights` | object | Weighted weather table. |
| `crop_rule_profile` | string | Farming integration later. |
| `help_doc_id` | string | Editor wiki link. |

## Validation rules

- Snow weather can only appear if season/biome permits it, unless manually overridden.
- Rain must write wetness and optionally puddle depth.
- Storms must have stronger wetness/wind than rain.
- Blizzard must have snow and visibility effects.
- Ashfall should be restricted to volcanic/ash biomes unless overridden.
