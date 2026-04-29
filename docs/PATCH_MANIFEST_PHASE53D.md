# Starlight Ridge — Phase 53d Patch Manifest

Patch name:

`Starlight_Ridge_phase53d_dynamic_voxelizer_rigging_spec.zip`

## Patch type

Spec / contract / command-profile overlay.

This patch intentionally avoids compile-critical Rust implementation changes. It adds docs, RON contract examples, editor command profiles, validation rules, and placeholder scripts so the next implementation phases can wire the system safely.

## Main focus

- Dynamic voxelizer data contracts
- Skeleton/bone animation contracts
- Spring bone secondary motion contracts
- External tool bridge commands
- Editor tab plan
- Runtime/offline separation
- Performance limits
- Legal note: broad workflow inspiration only; no copied Unity asset code/content

## Files added

- `docs/PHASE53D_DYNAMIC_VOXELIZER_RIGGING_SPEC.md`
- `docs/DYNAMIC_VOXELIZER_CONTRACTS.md`
- `docs/VOXEL_RIG_AND_BONE_ANIMATION_CONTRACTS.md`
- `docs/SPRING_BONE_SECONDARY_MOTION_CONTRACTS.md`
- `docs/DYNAMIC_VOXELIZER_EDITOR_TAB_PLAN.md`
- `docs/DYNAMIC_VOXELIZER_PERFORMANCE_LIMITS.md`
- `docs/DYNAMIC_VOXELIZER_LEGAL_NOTE.md`
- `docs/PHASE53E_HANDOFF_HIGH_DENSITY_CHARACTER_TEMPLATE_PASS.md`
- `content/editor_tools/dynamic_voxelizer_profiles.ron`
- `content/editor_tools/dynamic_voxelizer_commands.ron`
- `content/editor_tools/voxel_rig_profiles.ron`
- `content/editor_tools/spring_bone_profiles.ron`
- `content/editor_tools/dynamic_voxelizer_validation_rules.ron`
- `content/voxel_contracts/dynamic_voxelizer_contracts.ron`
- `content/voxel_contracts/voxel_rig_contracts.ron`
- `content/voxel_contracts/spring_bone_contracts.ron`
- `tools/scripts/voxel/dynamic_voxelizer_offline_placeholder.py`
- `tools/scripts/voxel/run_dynamic_voxelizer_placeholder.bat`
- `tools/scripts/voxel/run_dynamic_voxelizer_placeholder.sh`

## Expected result

The project has a clear native Starlight Dynamic Voxel System plan before any heavy runtime implementation begins.

