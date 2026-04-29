# Phase 53c — Egui Voxel Generator Tab + Command Runner Hookup

Phase 53c wires the Phase 53b pixel-voxel generator foundation into the native egui editor.

## Scope

This phase is workflow only.

It does not try to create final-grade character art. It gives the editor a usable path for:

```text
Generate → Preview → Open in MagicaVoxel → Clean up → Save → Refresh → Validate → Register
```

## Editor changes

Adds:

- `Assets → Voxel Generator`
- generator profile list
- selected profile details
- Generate Selected button
- Generate All Phase 53b Templates button
- Open in MagicaVoxel
- Open in Blockbench
- Open in Blender
- Open Folder
- Reference Assets panel
- third-party attribution validation
- generator workflow validation

## Files

- `crates/app/src/egui_editor.rs`
- `tools/scripts/voxel/phase53c_run_voxel_generator.py`
- `tools/scripts/voxel/phase53c_run_voxel_generator.bat`
- `content/editor_tools/voxel_generator_profiles.ron`
- `content/editor_tools/voxel_generator_commands.ron`
- `content/editor_tools/external_tools.ron`

## Important regression guard

The Voxel Generator tab is a child workspace panel only.

It must never call the full editor shell renderer. It must not create a second top bar, second side panel, second central panel, or second bottom status/console stack.

The project has repeatedly hit a nested-editor regression, so this rule is blocker-class.

## Third-party reference behavior

The Reference Assets panel is reference-only. Imported CC BY assets such as the Staxel Voxel Female reference can be used for:

- proportion comparison,
- external tool round-trip testing,
- GLB import testing,
- generator QA.

They should not become direct Starlight Ridge character bases unless explicitly promoted and attribution is carried into production credits.

## Next phase

Phase 53d should add a real reference compare loop:

- compare generated profile dimensions against reference assets,
- report missing generated outputs,
- preview side-by-side metadata,
- prep the Phase 53e high-density character template rebuild.
