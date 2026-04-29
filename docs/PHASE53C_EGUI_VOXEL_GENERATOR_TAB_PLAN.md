# Phase 53c — Egui Voxel Generator Tab + Command Runner Hookup

Add:

```txt
Assets -> Voxel Generator
```

Actions:

```txt
Generate Selected
Generate All Phase 53b Templates
Preview Selected .vox
Open in MagicaVoxel
Open in Blockbench
Bake in Blender
Validate Generated Assets
Register Generated Asset
```

Regression guard:

The generator tab must render as a child panel only. It must never call the full editor shell renderer or create a nested top bar, side panel, bottom panel, or full app frame.
