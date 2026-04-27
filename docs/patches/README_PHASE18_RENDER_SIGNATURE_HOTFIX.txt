Starlight Ridge Phase 18 Render Signature Hotfix

Drop this into the project root and overwrite.

Fixes the compile errors caused by crates/app/src/lib.rs still calling the older renderer API:

- RenderBootstrap::new(window_bootstrap, tile_map)
- renderer.render_frame(frame_index)

The current OpenGL renderer expects:

- RenderBootstrap::new(window_bootstrap, tile_map, Option<SpriteRenderData>)
- renderer.render_frame(frame_index, &[SpriteInstance])

This hotfix passes None for sprite render data and an empty sprite slice so the Phase 18 terrain/autotile test can compile without blocking on player sprite wiring.

After applying, run:

cargo check

or use the root build.sh menu.
