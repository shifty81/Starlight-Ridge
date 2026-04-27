# Starlight Ridge Phase 27 — Native Editor Shell Foundation

## Decision

The web Asset Lab is now treated as a prototype/fallback utility. The long-term editor direction is the native Rust editor with the game viewport in the center and editor docks around it.

## Immediate fixes in this phase

1. Runtime failures are no longer allowed to fail silently.
2. The native editor gets a visible dock/toolbar shell over the live game viewport.
3. Asset Studio, Animation Studio, and Character Studio are defined as first-class native workspaces.
4. Male and female base mannequin assets are added as the starting point for a non-destructive layered character pipeline.

## Native editor layout

### Top bar

Icon-first toolbar:

- Select
- Brush
- Eraser
- Fill
- Eyedropper
- Marquee Select
- Move Selection
- Seam Check
- Animation Studio
- Character Studio

Tool names should be shown through hover tooltips in the next UI pass, not as permanent word-heavy buttons.

### Left dock

- Project Browser
- Asset Browser
- Maps
- Prefabs later

### Center

- Live game/world viewport
- Playtest surface
- Future map/scene edit surface

### Right dock

- Inspector
- Tile Properties
- Character Layers
- Animation Properties later

### Bottom dock

- Console
- Runtime Log
- Validation
- Animation Timeline

## Why this replaces the web-first workflow

The web Asset Lab was useful for proving the idea, but the real editor needs access to the same render path, hot reload path, validation path, and viewport as the running client. Keeping the asset tools directly inside the Rust editor prevents workflow splits and makes map/scene/prefab editing much easier later.

## Phase 28 target

Phase 28 should connect real interactions to the native Asset Studio:

- selectable atlas tile grid inside the native editor
- right-side tile inspector
- role/collision editing from `base_tileset_roles.ron`
- seam preview panel
- save with hot reload
- icon hover tooltips
- panel collapse/minimize controls
