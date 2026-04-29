# Starlight Ridge Phase 51c — Editor Core Implementation

This patch focuses on turning the egui editor shell into the first usable project editor core before adding new gameplay systems such as destructible caves.

## Included

- Real map-cell editing in the native egui World > Map Paint panel.
- Brush/Tiles paint, Erase, Fill, Pick/eyedropper, Select behavior.
- Active layer selection and cell inspection.
- Active layer editor under World > Layers.
- Layer visibility toggles.
- Editor-only layer locks.
- Add blank layer, duplicate active layer, delete active layer, and reorder layer controls.
- Save active map layers to `content/maps/<map_id>/layers.ron`.
- Ctrl+S map-layer save shortcut.
- Unsaved/dirty map status indicator.
- Inspector buttons for painting, erasing, picking, saving metadata, and saving map layers.
- Validation panel upgraded to report active-map layer issues, missing tile refs, duplicate atlas cells, world manifest issues, and unsaved edits.
- Bottom status bar now shows active tool, map, layer, error count, warning count, and unsaved state.
- WorldGen panel now keeps visible action feedback after draft generation/bake contract writes.
- Editor opens directly to the World tab by default.

## Notes

This patch intentionally keeps interaction/spawn/trigger/object placement as scaffolds. The immediate goal is to make tile/layer edit-save-validate reliable before adding Core Keeper-style cave generation.

After applying, run:

```bash
cargo check --workspace
```

If the compiler reports egui API drift or borrow-check issues, patch the reported line directly before continuing to Phase 51d/52.
