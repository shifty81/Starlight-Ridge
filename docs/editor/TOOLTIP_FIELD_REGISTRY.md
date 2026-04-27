# Tooltip Field Registry — Initial Required Fields

This file lists the first tooltip/help ids that should exist before the Phase 52 editor UI is considered complete.

| Field ID | Short tooltip |
|---|---|
| `worldgen.seed` | Controls deterministic generation. Same seed and preset should produce the same draft. |
| `worldgen.scene_template` | Chooses the base scene shape and required authored layers. |
| `biome.temperature_range` | Preferred normalized temperature range for biome placement. |
| `biome.moisture_range` | Preferred normalized moisture range for biome placement. |
| `biome.elevation_range` | Preferred normalized elevation range for biome placement. |
| `biome.allowed_liquids` | Liquids that may naturally appear in this biome. |
| `material.family` | Semantic material group used by rendering and simulation. |
| `material.puddle_capacity` | Maximum shallow water this material can hold before runoff. |
| `material.snow_retention` | How easily snow accumulates and remains on this material. |
| `material.transition_profile` | Autotile/overlay rules used when this material borders another. |
| `liquid.viscosity` | Controls how slowly this liquid moves between cells. |
| `liquid.flow_rate` | Maximum amount this liquid can transfer per update. |
| `liquid.seep_rate` | How quickly this liquid disappears into absorbent ground. |
| `weather.wetness_delta` | How much wetness this weather adds to exposed cells. |
| `weather.puddle_delta` | How much puddle depth this weather can add to low exposed cells. |
| `weather.snow_delta` | How much snow depth this weather adds. |
| `vox.source_vox` | Source MagicaVoxel file used for baking runtime sprites. |
| `vox.required_facings` | Directional sprites required by the camera/rendering system. |
| `vox.footprint` | Logical placement area occupied by this object. |
| `vox.collision_mask` | Blocking cells used by movement and placement validation. |
| `layer.protected` | Prevents generation tools from overwriting this cell or region. |
| `layer.visibility` | Shows or hides this layer in the editor viewport. |
| `layer.locked` | Prevents accidental editing of this layer. |
