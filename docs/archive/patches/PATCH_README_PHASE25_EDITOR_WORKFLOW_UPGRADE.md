# Starlight Ridge Phase 25 — Editor Workflow Upgrade

This patch upgrades the browser-based editor foundation from Phase 24 into three clearer editor pages:

1. **Asset Lab** for atlas-safe pixel editing.
2. **Animation Lab** for static idle/world-tile animation strips.
3. **Character Lab** for mannequin-based layered character design.

## Main corrections

The old right-side 3x3 preview made a repeated tile look like a fake world-placement preview. Phase 25 replaces that with:

- selected tile preview
- full atlas output preview
- seam diagnostic preview

The seam view still repeats the tile, but it is explicitly labeled as a diagnostic tool rather than a world preview.

## Important atlas safety fix

Phase 25 keeps the atlas display grid separate from the atlas pixel data. The blue grid/highlight overlay is never saved into the atlas PNG. This is important when manually cleaning seam pixels.

## Asset Lab additions

- marquee select
- copy, cut, paste
- Ctrl+C, Ctrl+X, Ctrl+V
- Ctrl+Z, Ctrl+Y
- Ctrl+D deselect
- Ctrl-drag selected area to move it
- undo/redo
- eyedropper/color picker
- display-only pixel grid toggle
- display-only atlas grid toggle
- selected tile preview
- full atlas preview
- seam diagnostic with edge-delta readout

## Animation Lab additions

- dedicated animation page
- add/update frame from current Asset Lab tile
- duplicate/delete/reorder frames
- per-frame duration in milliseconds
- loop mode field
- strip preview
- playback preview
- save animation PNG strip and matching RON metadata

## Character Lab v1 additions

- base male underwear mannequin
- base female underwear mannequin
- non-destructive layer stack:
  - body
  - hair
  - clothing_top
  - clothing_bottom
  - shoes
  - equipment
- composite preview
- save composite PNG and RON metadata

## Files changed

Replaces:

- `tools/asset_lab.html`
- `tools/asset_lab_server.py`

Adds:

- `assets/textures/characters/base_male_underwear.png`
- `assets/textures/characters/base_female_underwear.png`
- `content/metadata/character_mannequins_phase25.ron`
- `docs/editor/EDITOR_LABS_PHASE25.md`

## Notes

This is still a local browser-based editor foundation. The later native editor should absorb this workflow into proper panels for map editing, scene editing, prefab editing, collision painting, animation metadata editing, and character equipment preview.
