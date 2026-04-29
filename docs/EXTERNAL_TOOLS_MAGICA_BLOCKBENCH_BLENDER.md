# External Tool Plan - MagicaVoxel, Blockbench, Blender

## MagicaVoxel

Primary voxel authoring tool for `.vox` assets.

Initial editor actions:

- detect MagicaVoxel path
- open selected `.vox`
- refresh after save
- validate `.vox` dimensions, palette, and missing files
- import unregistered `.vox` assets into the registry

## Blockbench

Secondary tool for block/voxel-style rig planning and animation blocking.

Initial editor actions:

- detect Blockbench path
- open supported source files
- export/import through configured content folders

Blockbench should not replace the `.vox` source pipeline; it is a companion tool.

## Blender

Advanced render/bake/preview tool.

Initial editor actions:

- detect Blender path
- run configured bake scripts
- generate thumbnails/turntables later
- assist with mesh cleanup, sprite bake, normal/depth bake, and promotional renders
