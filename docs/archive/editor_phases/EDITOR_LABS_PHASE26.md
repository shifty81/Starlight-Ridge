# Starlight Ridge Editor Labs Phase 26

Phase 26 turns the browser-based editor into a stronger project content workstation.

## Project Browser

The Project Browser scans `assets/` and `content/` for editable project files and shows:

- asset path
- asset type
- size
- basic usage references from content/docs/tools text
- texture preview for PNG files
- text preview for RON/JSON/TOML/Markdown/text files

PNG assets can be opened directly in Asset Lab.

## Tileset Roles

The Tileset Roles page reads `content/tiles/base_tileset.ron`, shows the atlas, and lets each named tile receive editor metadata:

- role
- collision type
- walkable
- blocks movement
- water
- interactable
- crop soil
- door/portal

The tool saves this to `content/tiles/base_tileset_roles.ron` instead of injecting new fields into `base_tileset.ron`. This keeps the current Rust content loader safe while giving the editor a real source-of-truth sidecar.

## Validation

The validation panel checks:

- missing tileset texture path
- duplicate tile IDs
- shared atlas coordinates
- tile coordinates outside declared atlas bounds
- map layer legend references that do not resolve to known tile/terrain IDs
- missing PNG references in content metadata
- tile role sidecar coverage

## Runtime-safe atlas export

The atlas export tool creates a padded/extruded runtime atlas. Each 32x32 tile gets copied into a larger padded cell and its border pixels are extruded outward. This helps prevent texture sampling bleed/grid seams once the renderer is wired to the padded atlas contract.

## Seam cleanup tools

Asset Lab now includes edge cleanup shortcuts:

- Copy Left → Right
- Copy Right → Left
- Copy Top → Bottom
- Copy Bottom → Top
- Blend L/R
- Blend T/B
- Soften Border

These are intended for manual cleanup of seamless terrain tiles.
