# Dynamic Voxelizer Contracts

These contracts describe the intended data model. They are written in Rust-like notation for clarity and mirrored in RON examples under `content/voxel_contracts/`.

## DynamicVoxelizerProfile

```rust
pub struct DynamicVoxelizerProfile {
    pub id: String,
    pub display_name: String,
    pub mode: VoxelizerMode,
    pub source_kind: VoxelizerSourceKind,
    pub output_kind: VoxelizerOutputKind,
    pub grid: VoxelizerGridDef,
    pub sampling: VoxelSamplingDef,
    pub palette: VoxelPaletteQuantizationDef,
    pub material_mapping: VoxelMaterialMappingMode,
    pub bake_cache: Option<VoxelBakeCacheDef>,
    pub performance_budget: VoxelizerPerformanceBudget,
}
```

## VoxelizerMode

```rust
pub enum VoxelizerMode {
    OfflineBake,
    RuntimeVfx,
    EditorPreview,
    AnimationPoseBake,
}
```

## VoxelizerSourceKind

```rust
pub enum VoxelizerSourceKind {
    StaticMesh,
    SkinnedMeshPose,
    Glb,
    Fbx,
    Obj,
    BlenderScene,
    BlockbenchModel,
    ExistingVox,
    ProceduralPrimitive,
}
```

## VoxelizerOutputKind

```rust
pub enum VoxelizerOutputKind {
    MagicaVoxelVox,
    StarlightVoxelCache,
    RuntimeVoxelParticles,
    PreviewMesh,
    CollisionApproximation,
    SilhouetteReport,
}
```

## VoxelizerGridDef

```rust
pub struct VoxelizerGridDef {
    pub resolution: [u32; 3],
    pub voxel_size_world: f32,
    pub origin_mode: VoxelizerOriginMode,
    pub max_voxels: u32,
}
```

Recommended defaults:

```txt
Character reference bake: 64 x 40 x 112
Hero character bake:      72 x 48 x 128
Tool bake:                96 x 32 x 32
Prop bake:                64 x 64 x 64
Runtime VFX burst:        16 x 16 x 16 to 32 x 32 x 32
```

## VoxelSamplingDef

```rust
pub struct VoxelSamplingDef {
    pub surface_sample_mode: SurfaceSampleMode,
    pub fill_mode: VolumeFillMode,
    pub normal_mode: NormalSamplingMode,
    pub color_mode: ColorSamplingMode,
    pub alpha_cutoff: f32,
    pub shell_thickness_voxels: u32,
}
```

## SurfaceSampleMode

```rust
pub enum SurfaceSampleMode {
    TriangleSurface,
    RaycastShell,
    SignedDistanceApprox,
    VertexCloud,
}
```

## VolumeFillMode

```rust
pub enum VolumeFillMode {
    SurfaceOnly,
    SolidFill,
    FloodFillInterior,
    ShellOnly,
}
```

## Palette quantization

The voxelizer should avoid unlimited colors. Starlight Ridge needs palette-bound pixel-voxel readability.

```rust
pub struct VoxelPaletteQuantizationDef {
    pub palette_id: String,
    pub max_colors: u32,
    pub dither: bool,
    pub material_priority: Vec<String>,
}
```

## Offline bake flow

```txt
source mesh / GLB / skinned pose
→ normalize scale and pivot
→ sample surface or volume
→ quantize palette
→ apply material mapping
→ prune isolated noise
→ write .vox or cache
→ validate
→ register generated asset
```

## Runtime VFX flow

```txt
runtime mesh/pose/effect source
→ sample low-resolution temporary grid
→ spawn voxel particles or instanced cubes
→ apply lifetime/physics
→ never write persistent asset unless requested by editor
```

