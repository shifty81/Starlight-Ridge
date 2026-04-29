# Phase 53f — Voxel Rig Preview + Tool Attachment Preview

## Purpose

Phase 53f connects the new high-density character templates to an editor-facing rig/tool attachment workflow.

This phase does not implement full runtime bone animation. It adds the editor tab, metadata contracts, validation hooks, and command-runner bridge needed before building the real 3D rig overlay.

## Added editor workflow

```txt
Assets → Voxel Rig Preview
```

The tab shows:

- rig preview profiles,
- base template path/status,
- skeleton ID,
- rig ID,
- default pose,
- animation profile,
- required attachment anchors,
- discovered tool `.vox` assets,
- external tool buttons for MagicaVoxel/Blender,
- validation action,
- placeholder command-runner action.

## Character base rule

Base character templates remain:

- bald,
- clean-shaven,
- neutral,
- no baked identity traits,
- no baked hair,
- no baked facial hair.

Hair, facial hair, hats, clothing, gloves, boots, tools, and accessories remain modular overlays.

## Files added

```txt
content/editor_tools/phase53f_voxel_rig_preview_profiles.ron
content/editor_tools/phase53f_voxel_rig_preview_validation_rules.ron
content/editor_tools/phase53f_voxel_rig_preview_commands.ron
content/voxel_contracts/phase53f_voxel_rig_preview_contracts.ron
tools/scripts/voxel/phase53f_rig_preview_placeholder.py
```

## Rust editor changes

`crates/app/src/egui_editor.rs` now includes:

- `AssetSubTab::VoxelRigPreview`,
- rig-preview profile loading,
- `Assets → Voxel Rig Preview` tab,
- selected base profile panel,
- tool attachment preview panel,
- validation workflow,
- external open buttons.

## Nested editor regression guard

The Voxel Rig Preview tab is a child workspace tab only.

It must never call the full editor shell renderer, create a second top bar, create a second side panel, or create a second bottom/status stack.

## Next phase

Phase 53g should add the actual 3D overlay viewport pass for:

- bone hierarchy drawing,
- attachment marker drawing,
- base/tool overlay preview,
- pose selection,
- basic A-pose/idle/tool-pose visualization.
