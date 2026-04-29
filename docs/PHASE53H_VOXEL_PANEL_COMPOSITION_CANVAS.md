# Phase 53h — Voxel Panel Composition Canvas

Phase 53h extends the voxel-pixel panel designer from single-panel authoring into reusable panel compositions.

## Scope

This phase is still egui-only. It does not add the real 3D voxel viewport and it does not touch gameplay.

## Added

- Composition scene contracts on voxel panel kits.
- Composition instances that reference authored panel IDs.
- Socket connection records between panel instances.
- Future 3D viewport prep metadata per composition scene.
- Composition selector/editor in `Assets → Voxel Panels`.
- Instance add/remove/edit controls.
- 2D composition canvas preview.
- Click-to-select composition instances.
- Drag-to-reposition composition instances.
- Socket markers on composition instances.
- Socket connection lines.
- Snap selected instance to nearest compatible socket.
- Validation for composition scene dimensions, instance references, connection references, socket compatibility, canvas bounds, and snapped socket positions.

## Data contracts

The following structs were added to `game_data::defs`:

- `VoxelPanelCompositionSceneDef`
- `VoxelPanelCompositionViewportPrepDef`
- `VoxelPanelCompositionInstanceDef`
- `VoxelPanelCompositionConnectionDef`

`VoxelPanelKitDef` now includes:

```rust
#[serde(default)]
pub compositions: Vec<VoxelPanelCompositionSceneDef>
```

This keeps older Phase 53f/53g kits loadable.

## Editor path

Open:

```text
Assets → Voxel Panels
```

The left column now contains a **Compositions** section. The middle column contains the existing 2D depth-slice panel editor plus the new **Composition canvas**.

## Socket snapping rule

The selected composition instance can snap to the nearest compatible socket on another instance. Compatibility is currently metadata-driven:

- A socket can accept another panel ID.
- A socket can accept another panel kind.
- A socket can accept another socket ID.
- A socket can accept another socket edge.
- `any` or `*` accepts all socket targets.

The created connection is stored in the kit RON. The later 3D viewport should consume the same source data rather than introducing a separate composition format.

## Single-root shell regression guard

The composition canvas is rendered inside the existing `Assets → Voxel Panels` branch. It must not call the full egui editor shell and must not create a nested top bar, side panel, bottom panel, console stack, or duplicate editor frame.

## Local test checklist

1. Run `cargo check`.
2. Run `RUN_EDITOR_DEBUG.bat`.
3. Open `Assets → Voxel Panels`.
4. Select a starter kit.
5. Select a composition.
6. Add the selected panel as an instance.
7. Click an instance on the composition canvas.
8. Drag the instance.
9. Use `Snap selected to socket`.
10. Save the kit and confirm a backup is created.
11. Reload the kit and confirm composition instances/connections persist.
12. Confirm no nested editor shell appears.
