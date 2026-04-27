# Phase 14 - Release-All Compile Fix

## Symptom

Option 17 / release-all app binaries failed while compiling `app`:

```text
error[E0061]: this function takes 6 arguments but 7 arguments were supplied
```

The error pointed at `crates/app/src/lib.rs` and the call to `push_terrain_layer_tiles(...)`.

## Cause

The safe-tile render cleanup removed the need to pass `tileset.columns.max(1)` into `push_terrain_layer_tiles(...)`, because guessed atlas-column transition overlays were disabled. The helper signature was updated, but the call site still passed the old atlas-column argument.

## Fix

The call site now matches the helper signature:

```rust
push_terrain_layer_tiles(
    &mut tiles,
    &tile_grid,
    &atlas_lookup,
    &mut stats,
    map_id,
    &layer.id,
);
```

## Next checks

Run these from `BUILD_MENU.bat`:

- Option 2: cargo check
- Option 17: build all release app binaries
- Option 5: run game debug
- Option 6: run editor debug

If rendering still looks wrong, the next work item is not this compile fix. It is the terrain atlas contract: add explicit tile roles for water/path/soil/grass edge and corner cases, then re-enable transition overlays by role name instead of guessed atlas columns.
