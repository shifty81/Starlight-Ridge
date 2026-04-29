# Phase 53j — Voxel Composition 3D Preview Viewport

Phase 53j turns the Phase 53i composition mesh-preview export into the first real visual proof inside the egui editor.

## Scope

- Load the Phase 53i `VoxelPanelCompositionMeshExportDef` RON preview file.
- Add a read-only 3D preview panel inside `Assets -> Voxel Panels`.
- Provide orbit/pan/zoom camera controls.
- Draw baked voxel boxes from exported voxel coordinates.
- Draw inclusive composition bounds.
- Draw socket gizmos.
- Draw connection gizmos.
- Keep voxel panel source data as the source of truth.
- Keep the viewport read-only: no live mesh editing yet.
- Do not touch gameplay systems.
- Preserve the single-root egui shell. This phase does not call full editor shell rendering from any nested panel.

## Editor behavior

Open `Assets -> Voxel Panels` and use the `3D Preview` panel.

Available actions:

- `Export + load`: bakes the selected composition into the Phase 53i preview RON and immediately loads it into the viewport.
- `Load RON`: loads the existing Phase 53i preview file for the selected kit/composition.
- `Reset camera`: restores the default isometric preview camera.
- Drag inside the viewport to orbit.
- Use the sliders/drag values to adjust yaw, pitch, zoom, and pan.
- Toggle voxels, bounds, sockets, connections, and labels.

## Implementation notes

The viewport is intentionally egui-painter based for this phase. It does not introduce a renderer dependency, game runtime dependency, or editable mesh layer. The preview consumes the same RON contract that future real 3D rendering can consume.

## Files changed

- `crates/app/src/egui_editor.rs`
- `Cargo.toml`
- `content/editor_voxel_panels/README.md`
- `content/editor_voxel_panels/preview_exports/README.md`
- `content/editor_voxel_panels/phase53j_manifest.ron`
- `docs/PHASE53J_VOXEL_COMPOSITION_3D_PREVIEW_VIEWPORT.md`

## Test checklist

Run locally:

```text
cargo check --workspace
cargo run -p voxel_generator -- --all --project-root .
RUN_EDITOR_DEBUG.bat
```

Manual editor check:

1. Open `Assets -> Voxel Panels`.
2. Select a kit and composition.
3. Click `Export + load` in the `3D Preview` panel.
4. Confirm voxel boxes render.
5. Toggle bounds, sockets, connections, and labels.
6. Drag the viewport to orbit.
7. Adjust pan and zoom controls.
8. Confirm the editor shell is not nested inside itself.
