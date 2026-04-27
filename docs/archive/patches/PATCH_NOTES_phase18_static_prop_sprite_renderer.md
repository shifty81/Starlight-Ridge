# Starlight Ridge Phase 18 Static Prop Sprite Renderer Patch

Drop this zip into the project root and overwrite files when prompted.

## What this patch changes

1. Preserves the Phase 17 generated runtime textures and metadata.
2. Adds a second sprite-sheet render layer in `engine_render_gl`.
3. Builds static prop sprite instances from each active map's `props.ron`.
4. Uses the Phase 17 Ocean bridge sheet:
   - `assets/textures/oceans_heart_bridge_phase17.png`
   - `content/metadata/oceans_heart_bridge_phase17.ron`
5. Adds visible test props to:
   - `content/maps/autotile_test_coast/props.ron`
   - `content/maps/autotile_test_pond/props.ron`
   - `content/maps/starter_farm/props.ron`

## Expected result

- The map still renders.
- The player still renders.
- Static visual props now render from the Ocean bridge sheet.
- Existing non-bridge placeholder props are skipped instead of breaking rendering.

## Important implementation detail

The renderer now draws:

```text
tile map
static props
player
```

This keeps props visually separate from terrain and avoids forcing Ocean bridge sprites into the terrain atlas.

## Verification note

This patch was source-audited and mechanically checked for the expected symbols. Cargo is not available in this container, so run your project build script after applying it.
