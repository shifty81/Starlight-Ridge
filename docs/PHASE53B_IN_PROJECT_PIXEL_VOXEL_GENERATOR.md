# Phase 53b — In-Project Pixel-Voxel Generator Foundation

Phase 53b adds the foundation for a Starlight-owned generator that outputs repeatable `.vox` starting points for characters, NPCs, and tools.

The generator is not the final artist. It is the standards engine. It enforces dimensions, naming, density targets, bald/clean-shaven character bases, modular slots, and output paths.

## Architecture

```txt
Starlight Ridge Editor / Command Runner
  -> Voxel Generator Profile
  -> Generate .vox
  -> Validate profile and output
  -> Open in MagicaVoxel for cleanup
  -> Optional Blockbench pose/attachment planning
  -> Optional Blender bake/render
  -> Refresh and register in editor
```

## Scope

Included now:

- `crates/voxel_generator` crate scaffold
- generator profiles in `content/editor_tools/voxel_generator_profiles.ron`
- command profiles in `content/editor_tools/voxel_generator_commands.ron`
- external tool bridge placeholders in `content/editor_tools/external_tools.ron`
- repeatable generated `.vox` examples under `content/voxels/generated_templates/`

Deferred to Phase 53c:

- actual egui Voxel Generator tab
- generate/open/validate/register buttons
- pipeline panel hookup
- selected-asset preview wiring

## Non-negotiable base character rule

Generated character bases must have:

- no hair
- no facial hair
- no beard
- no mustache
- no baked identity traits

Hair, facial hair, hats, outfits, gloves, boots, and accessories are modular overlays later.
