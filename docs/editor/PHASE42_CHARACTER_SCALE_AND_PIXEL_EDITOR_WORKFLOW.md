# Phase 42 — Character Scale + Shared Pixel Editor Workflow

## Immediate compile blocker

The latest cargo check reached `app` and failed inside `crates/app/src/egui_editor.rs` because a duplicate, empty `draw_top_bar` function header left the `impl StarlightRidgeEguiEditor` block with an unclosed delimiter. This patch removes that duplicate header and keeps the real `draw_top_bar` implementation.

## Character scale decision

The current Starlight Ridge terrain contract uses 32x32 world tiles. Character scale should therefore use tile-relative authoring rather than arbitrary sprite sizes.

Recommended scale ladder:

| Scale class | World height | Frame target | Use |
|---|---:|---:|---|
| Small child | 1.0 tile | 32x32 | young child / tiny NPC |
| Tall child / teen | 1.5 tiles | 32x48 | older child / teen / short villager |
| Standard adult | 2.0 tiles | 32x64 | player and most adult NPCs |
| Tall adult / rig | 2.25 tiles | 48x72 | large NPC, heavy clothing, backpack, later rig/exosuit |

The important rule is that collision and rendering are separate:

- Render sprite can be 64 pixels tall.
- Foot anchor stays bottom-center on the tile grid.
- Collision should stay smaller than the art so the player can walk naturally around objects.
- Sorting should use feet Y, not sprite top Y.

## Why 2 blocks tall for adults is the cleanest baseline

A 2-block adult gives enough vertical room for readable faces, hair, shirts, pants, hats, tools, and future backpacks/rig parts. It also makes children easy to represent without special-case scaling: children use 1.0 or 1.5 tile bases, adults use 2.0 tile bases, and large/equipped silhouettes can use 2.25 tile bases.

Do not scale one body up and down at runtime as the primary art workflow. Author separate bases for each scale class. Shared clothing can exist, but it needs per-scale offsets.

## Character Studio workflow

The Character workspace should use the same pixel editor core as tiles and props, but with character-specific panels:

- Mannequin
- Paperdoll Layers
- Equipment / Overlays
- Animation Preview
- Scale Validator
- Export / Metadata

The mannequin view should show:

- male / female / child / teen / adult scale selection
- base body layer
- overlay stack
- facing direction
- animation playback
- foot anchor
- collision capsule
- sprite bounds
- per-frame overlay offsets

## Shared pixel editor workflow

Build one full-featured pixel editor core and reuse it across:

- tile editing
- atlas editing
- atlas compare/import
- props/objects
- character mannequin editing
- clothing, beard, hat, hair, and equipment overlays
- animation frame editing

The canvas viewport must remain static. Tool drawers expand inside the left tool region and push the RGBA/display block sideways, but the central viewport anchor should not jump.

Recommended left-side pattern:

```text
[ Tool Rail ][ Optional Tool Drawer ][ RGBA / Display ][ Static Canvas ][ Inspector ]
```

Clicking Pencil once opens the brush drawer. Clicking Pencil again collapses it.

## Brush drawer requirements

Pencil drawer:

- 1px pencil
- square brush
- circle brush
- line brush
- dither brush
- replace-color brush
- mirror-aware brush
- stamp brush
- pattern brush

Eraser drawer:

- 1px eraser
- square eraser
- alpha eraser
- color-target eraser

Fill drawer:

- contiguous fill
- global replace fill
- threshold fill
- pattern fill

## Zoom requirements

The asset editor and character editor both need:

- mouse wheel zoom
- middle mouse drag pan
- space + drag pan
- Fit / 100% / 200% / 400% / 800%
- pixel grid toggle
- tile/sprite bounds toggle
- center guide toggle
- onion skin toggle for animation contexts

## Runtime contract

Character data should be metadata-driven:

- frame size by scale class
- foot anchor by direction/frame
- collision capsule by scale class
- paperdoll layer order
- overlay offsets by scale/direction/frame
- animation event markers
- tool socket anchors

This keeps adult, child, clothing, beards, hats, and later rig/exosuit parts from becoming hardcoded one-offs.
