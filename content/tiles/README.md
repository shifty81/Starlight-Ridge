# Tileset Content Folder

`base_tileset.ron` is the active runtime tileset for Starlight Ridge.

Phase 17 active generated atlas:

```text
assets/textures/terrain_atlas_phase17_generated.png
```

Runtime contract:

- tile size: `32x32`
- atlas grid: `11 columns x 13 rows`
- maps and terrain rules should use named tile IDs from `base_tileset.ron`
- do not infer tile meaning from raw atlas positions unless the role is documented in `base_tileset.ron`

The Phase 17 generated atlas was normalized from a larger art-source image into the exact runtime sheet dimensions. The generated master image is preserved under:

```text
assets/art_source/generated_phase17/
```

Seasonal tileset files are retained as future art references and should not be selected by maps until they are rebuilt against this named-role contract.
