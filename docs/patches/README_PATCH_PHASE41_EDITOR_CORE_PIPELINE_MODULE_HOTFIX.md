# Phase 41 - editor_core pipeline module hotfix

## Purpose

This hotfix repairs the build failure reported after Phase 40.

The failing log showed:

```text
error[E0583]: file not found for module `atlas_pipeline`
error[E0583]: file not found for module `export_pipeline`
```

`crates/editor_core/src/lib.rs` declared both modules, but the matching source files were not present in the package.

## Files added

```text
crates/editor_core/src/atlas_pipeline.rs
crates/editor_core/src/export_pipeline.rs
```

## Result

The editor core crate now has the missing Phase 19 atlas-pipeline report module and Phase 20 export/validation-pipeline report module.

These modules are intentionally lightweight and safe:

- they only inspect the already-loaded `ContentRegistry`
- they log summary counts
- they do not mutate project content
- they keep the egui editor startup path intact

## After applying

Run:

```text
BUILD_MENU.bat
```

Then choose the cargo check option again.
