Starlight Ridge Phase 20 - Render Signature Compile Hotfix

Purpose:
- Fixes the compile errors after Phase 19:
  - RenderBootstrap::new now expects 4 arguments.
  - RenderBootstrap::render_frame now expects 3 arguments.

Changed file:
- crates/app/src/lib.rs

What changed:
- App bootstrap now calls:
  RenderBootstrap::new(window_bootstrap, self.tile_map.clone(), None, None)
- The render loop now calls:
  renderer.render_frame(stats.frame_index, &[], &[])

Expected result:
- The app crate should compile past the render API mismatch.
- Static props are still not fully wired from app-side runtime data in this hotfix; this only restores build compatibility with the two-channel sprite renderer signature.

Apply:
- Extract this zip over the Starlight Ridge project root and overwrite files.
- Rerun the build menu option that generated latest.log.
