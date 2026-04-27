# Starlight Ridge Phase 27 Native Editor Shell Foundation

## Apply

Extract this ZIP over the project root and overwrite files.

## Rebuild required

Yes. This patch touches Rust code.

Recommended first command:

```bash
cargo check
```

Then build normally with your menu.

## What this patch changes

### Runtime diagnostics

- Adds visible runtime failure dialogs on Windows.
- Adds a panic hook that writes `logs/latest_runtime_failure.log`.
- Converts early renderer/window startup failures into real returned errors instead of silent close behavior.
- Adds `RUN_EDITOR_DIAGNOSTIC.bat` for launching the built editor and printing the latest failure log before the terminal closes.

### Native editor direction

- Adds a first native editor shell overlay to the Rust editor window.
- Center remains the live game viewport.
- Adds left/right/bottom dock scaffolding.
- Adds icon-first toolbar scaffolding.
- Keeps the web Asset Lab as a fallback/prototype only.

### Editor core model

- Adds native shell descriptors.
- Adds Asset Studio descriptors.
- Adds Character Lab descriptors.

### Character assets

- Adds better male/female mannequin base PNGs using the current player sheet scale.
- Adds character base metadata for non-destructive layered editing.

## Files touched

- `crates/app/src/lib.rs`
- `crates/engine_render_gl/src/lib.rs`
- `crates/editor_core/src/lib.rs`
- `crates/editor_core/src/native_shell.rs`
- `crates/editor_core/src/asset_studio.rs`
- `crates/editor_core/src/character_lab.rs`
- `assets/editor/icons/toolbar_icons_phase27.png`
- `assets/textures/characters/base_male_underwear_phase27.png`
- `assets/textures/characters/base_female_underwear_phase27.png`
- `content/editor/native_editor_layout_phase27.ron`
- `content/editor/asset_studio_phase27.ron`
- `content/metadata/character_bases_phase27.ron`
- `RUN_EDITOR_DIAGNOSTIC.bat`

## Important boundary

This phase does not finish the native pixel editor yet. It moves the project onto the right native editor foundation and fixes the silent-close diagnostics problem first. Phase 28 should wire actual native Asset Studio interactions.
