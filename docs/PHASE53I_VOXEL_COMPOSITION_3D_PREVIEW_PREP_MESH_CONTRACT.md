# Phase 53i — Voxel Composition 3D Preview Prep / Mesh Contract

Phase 53i adds the non-rendering data bridge between the 2D voxel-panel composition canvas and the later real 3D voxel viewport.

## Scope

This phase remains editor/data-contract only.

Added:

- composition-to-preview mesh export contracts
- baked panel instance transform records
- baked voxel records with material color and render hints
- composition bounds metadata
- socket gizmo records
- connection gizmo records
- per-composition export path metadata
- `Export 3D preview RON` action in `Assets -> Voxel Panels -> Compositions`
- validation for mesh export path and bake anchor metadata
- starter kit metadata updated for Phase 53i

Explicitly not added:

- full 3D rendering
- GPU/mesh renderer changes
- runtime gameplay spawning
- MagicaVoxel / Blockbench / Blender export
- nested editor shell rendering

## Export location

Default export folder:

```text
content/editor_voxel_panels/preview_exports/
```

The exported file name is generated from the kit id and composition id:

```text
<kit_id>__<composition_id>_phase53i_preview.ron
```

## Export contract

The export contract is `VoxelPanelCompositionMeshExportDef`.

It contains:

- source kit id
- source composition id
- generated phase and timestamp
- voxel unit and layer gap
- source axis and bake anchor
- inclusive voxel-space bounds
- baked instance records
- baked voxel records
- socket gizmo records
- connection gizmo records
- notes for the future 3D viewport

The future 3D viewport should consume this export first before direct live rendering is added.

## Local test checklist

Run locally:

```bat
cargo check
RUN_EDITOR_DEBUG.bat
```

Then verify:

1. Open `Assets -> Voxel Panels`.
2. Select a kit.
3. Select a composition.
4. Use `Export 3D preview RON`.
5. Confirm the preview export file appears under `content/editor_voxel_panels/preview_exports/`.
6. Confirm the editor still renders as one top-level shell only.
7. Confirm no gameplay/runtime behavior changed.
