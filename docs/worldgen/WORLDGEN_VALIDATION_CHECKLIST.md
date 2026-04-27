# Worldgen Validation Checklist

## Seed determinism

- Same seed produces identical generated draft.
- Different scene seed changes only the target scene.
- Pass seeds are recorded in generation metadata.
- Debug overlays can be regenerated from metadata.

## Biome validation

- Every cell has a biome id.
- Every biome id exists in the biome registry.
- Biome transitions are allowed by the biome adjacency rules.
- No isolated single-cell biome islands unless allowed.
- Special biomes like volcanic/ashlands require proper material and hazard definitions.

## Material validation

- Every terrain cell has a material id.
- Every material id exists in the material registry.
- Every material has required seasonal variants or a declared seasonal fallback.
- Every material has editor thumbnail metadata.
- Every material has a tooltip/help doc id.

## Liquid validation

- Every liquid cell has a valid liquid id.
- Liquid/material combinations are allowed.
- Lava/oil do not appear in normal farm maps unless intentionally placed.
- Puddle capacity is valid for each material.
- Flow/depth values are within configured bounds.

## Layer validation

- Required layers exist.
- Layer dimensions match map dimensions.
- Protected authored cells are not overwritten by generation.
- Collision layer agrees with structure/prop footprints.
- Interaction ids resolve to known actions.

## VOX validation

- `.vox` files parse successfully.
- Each source has a manifest entry.
- Required baked facings exist: N, E, S, W.
- Footprint mask exists for placeable objects.
- Collision mask exists if object is blocking.
- Anchor point exists.
- Missing variants have documented fallback behavior.

## Camera/directional validation

- Every rotatable visible prop has directional render data.
- Camera yaw maps to valid facing set.
- Selection bounds rotate with object footprints.
- Directional shadows exist or have fallback.

## Editor help validation

- Every exposed field has a short tooltip.
- Every exposed field has a full help doc id or inherited group help.
- Every validation error has a help link.
- Wiki pages referenced by fields exist.
