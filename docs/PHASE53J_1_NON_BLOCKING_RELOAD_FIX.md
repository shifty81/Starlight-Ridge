# Phase 53j.1 — Non-Blocking Editor Reload Fix

## Problem

The top command-strip reload button and F5 path called `reload_content()` directly on the egui UI frame.
That path scanned VOX assets, reloaded the content registry, reloaded map layers and world placements, and rebuilt the map preview before returning control to the frame.
On larger content sets this could make the entire editor appear hung.

## Fix

`reload_content()` now queues a background content reload worker instead of performing the full reload synchronously.
The editor shell polls the worker each frame and applies the result only after the worker finishes.

## Scope

- Top `Reload F5` button no longer performs heavy reload work on the UI thread.
- F5 no longer performs heavy reload work on the UI thread.
- Bottom-panel `Reload now` no longer performs heavy reload work on the UI thread.
- Repeated reload clicks are ignored while a reload is already active.
- The status bar reports background reload progress.
- Stale reload results are discarded if the active map changes before the worker finishes.
- The single-root egui shell path is preserved.
- No gameplay changes.

## Validation checklist

Run locally:

```bash
cargo check --workspace
cargo run -p app
```

Then verify:

1. Open the editor.
2. Click top `Reload F5`.
3. Confirm the UI remains responsive.
4. Confirm the status bar shows background reload progress.
5. Press F5 repeatedly and confirm it does not queue multiple reloads.
6. Open the Hot Reload bottom tab and click `Reload now`.
7. Confirm no nested editor shell appears.
8. Confirm `Assets -> Voxel Panels` still loads and the Phase 53j preview still renders.
