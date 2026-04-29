# Phase 52g_2 — Warning Cleanup + Missing Pipeline Restore + Bake Report Fix

## Purpose

This patch stabilizes the Phase 52g generated scene bake path before the next worldgen editing pass. The goal is to keep the egui editor build-clean, restore missing editor pipeline content, and make bake reports easier to trust.

## Changes

### Warning cleanup

- `EditorMapState.map_id` is now surfaced in the map/editor UI instead of sitting unused.
- `open_web_asset_lab()` is now wired through an optional Asset Studio helper button.
- The unused `is_likely_transition()` helper was removed.

### Pipeline content restore

Added canonical content files for the missing editor pipelines:

- `content/editor_pipeline/phase19_editor_atlas_pipeline.ron`
- `content/editor_export/phase20_editor_export_validation_pipeline.ron`

These restore the registry counts for:

- `editor_atlas_pipelines`
- `editor_export_pipelines`

The Phase 19 file provides a real terrain atlas editor contract with seasonal sets, water animation frames, clipboard behavior, validation checks, and a game preview profile.

The Phase 20 file provides a real export/validation contract with export profiles, validation panels, autotile preview rules, transition editor data, collision metadata, and atlas cleanup manifests.

### Bake report cleanup

The generated scene bake report now labels its counts as:

- `terrain_changed`
- `terrain_unchanged`
- `protected_layer_cells_preserved`
- `object_markers`
- `warnings`

The previous report made protected accounting look like terrain cells had been skipped even when it was only counting unrelated non-target layers. Protected-cell reporting now only counts layers actually named in the bake contract `protected_layers` list.

If a bake contract ever marks the terrain target layer itself as protected, the preview warns and commit refuses to write that layer.

### Egui shell regression guard

A debug-only render-depth guard was added around the full editor shell path. If the full shell is ever called recursively again, the editor logs a blocker-class nesting regression and trips a debug assertion.

This is specifically meant to prevent the repeated nested-editor bug where the top bar, side panels, bottom console, and center panel render inside themselves.

### External runtime warning classification

OBS/Overwolf Vulkan overlay warnings are external loader/runtime warnings, not project compile errors. They may still appear in logs depending on installed overlay software, but they are not treated as Starlight Ridge source failures.

## Expected result

After this patch, the editor should still launch normally, but the startup log should show non-zero counts for Phase 19 and Phase 20 editor pipelines, and the generated bake preview summary should no longer report misleading protected-cell counts.

## Next phase

The next feature phase can safely move to:

`Phase 52h — WorldGen Brush + Rule Override Editor`
