Starlight Ridge Art Pass v2

This pass replaces the first placeholder/debug-looking texture sheets with the visual preview assets converted into exact runtime sheets:

- assets/textures/terrain_atlas_phase5.png
- assets/textures/entity_sprite_sheet_phase5.png
- content/tiles/base_tileset.ron
- content/maps/town/layers.ron

It keeps the same OpenGL/glow renderer and editor overlay code. Re-run:

cargo check
cargo run -p app
cargo run -p app --bin editor

If the map opens but still looks wrong, delete target/ and rerun so stale cached build artifacts cannot mask updated assets.
