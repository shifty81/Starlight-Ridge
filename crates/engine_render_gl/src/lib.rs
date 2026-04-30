use std::ffi::CString;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use engine_window::WindowBootstrap;
use glow::HasContext;
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::prelude::*;
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use raw_window_handle::HasWindowHandle;
use winit::window::Window;

const TEXTURED_VERTICES: [f32; 16] = [
    // x, y, u, v
    -0.55, -0.55, 0.0, 0.0, 0.55, -0.55, 1.0, 0.0, 0.55, 0.55, 1.0, 1.0, -0.55, 0.55, 0.0, 1.0,
];

const TEXTURED_INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

#[derive(Debug, Clone)]
pub struct TileMapRenderData {
    pub texture_path: PathBuf,
    pub map_width: u32,
    pub map_height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub atlas_columns: u32,
    pub atlas_rows: u32,
    pub tiles: Vec<TileInstance>,
}

#[derive(Debug, Clone, Copy)]
pub struct TileInstance {
    pub x: u32,
    pub y: u32,
    pub atlas_x: u32,
    pub atlas_y: u32,
}

#[derive(Debug, Clone)]
pub struct SpriteRenderData {
    pub texture_path: PathBuf,
    pub sprite_width: u32,
    pub sprite_height: u32,
    pub atlas_columns: u32,
    pub atlas_rows: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteInstance {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub atlas_x: u32,
    pub atlas_y: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

#[derive(Debug, Clone)]
pub struct VoxelSceneRenderData {
    pub vertices: Vec<VoxelVertex>,
    pub indices: Vec<u32>,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub object_ranges: Vec<VoxelSceneObjectRange>,
}

#[derive(Debug, Clone)]
pub struct VoxelSceneObjectRange {
    pub object_key: String,
    pub label: String,
    pub index_start: u32,
    pub index_count: u32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct VoxelEditorViewport {
    pub width: u32,
    pub height: u32,
    pub yaw_degrees: f32,
    pub pitch_degrees: f32,
    pub zoom: f32,
}

impl Default for VoxelEditorViewport {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            yaw_degrees: -35.0,
            pitch_degrees: 58.0,
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoxelEditorViewportRequest {
    pub scene: VoxelSceneRenderData,
    pub viewport: VoxelEditorViewport,
    pub selected_object_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VoxelEditorPickResult {
    pub object_key: String,
    pub label: String,
    pub distance_to_center: f32,
}

fn encode_pick_id(id: u32) -> [f32; 4] {
    [
        ((id >> 16) & 0xff) as f32 / 255.0,
        ((id >> 8) & 0xff) as f32 / 255.0,
        (id & 0xff) as f32 / 255.0,
        1.0,
    ]
}

fn decode_pick_id(pixel: [u8; 4]) -> u32 {
    ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | pixel[2] as u32
}

pub fn pick_voxel_object_by_projected_bounds(
    scene: &VoxelSceneRenderData,
    viewport: VoxelEditorViewport,
    pointer_x: f32,
    pointer_y: f32,
) -> Option<VoxelEditorPickResult> {
    if scene.object_ranges.is_empty() || viewport.width == 0 || viewport.height == 0 {
        return None;
    }
    let mvp = voxel_editor_view_projection(scene, viewport)?;
    let pointer = glam::Vec2::new(pointer_x, pointer_y);
    let mut best: Option<VoxelEditorPickResult> = None;

    for range in &scene.object_ranges {
        let rect = projected_bounds_rect(range.bounds_min, range.bounds_max, viewport, mvp)?;
        if pointer.x < rect[0] || pointer.y < rect[1] || pointer.x > rect[2] || pointer.y > rect[3]
        {
            continue;
        }
        let center = glam::Vec2::new((rect[0] + rect[2]) * 0.5, (rect[1] + rect[3]) * 0.5);
        let distance_to_center = pointer.distance(center);
        if best
            .as_ref()
            .is_none_or(|current| distance_to_center < current.distance_to_center)
        {
            best = Some(VoxelEditorPickResult {
                object_key: range.object_key.clone(),
                label: range.label.clone(),
                distance_to_center,
            });
        }
    }

    best
}

fn voxel_editor_view_projection(
    scene: &VoxelSceneRenderData,
    viewport: VoxelEditorViewport,
) -> Option<glam::Mat4> {
    let min = glam::Vec3::from_array(scene.bounds_min);
    let max = glam::Vec3::from_array(scene.bounds_max);
    if !min.is_finite() || !max.is_finite() {
        return None;
    }

    let center = (min + max) * 0.5;
    let extent = (max - min).max(glam::Vec3::splat(1.0));
    let radius = extent.length().max(1.0);
    let yaw = viewport.yaw_degrees.to_radians();
    let pitch = viewport.pitch_degrees.to_radians().clamp(0.1, 1.45);
    let distance = (radius * 1.8 / viewport.zoom.clamp(0.1, 4.0)).max(1.0);
    let eye_offset = glam::Vec3::new(
        yaw.sin() * pitch.cos(),
        -yaw.cos() * pitch.cos(),
        pitch.sin(),
    ) * distance;
    let eye = center + eye_offset;
    let aspect = viewport.width.max(1) as f32 / viewport.height.max(1) as f32;
    let view = glam::Mat4::look_at_rh(eye, center, glam::Vec3::Z);
    let projection =
        glam::Mat4::perspective_rh_gl(40.0_f32.to_radians(), aspect.max(0.1), 0.01, radius * 8.0);
    Some(projection * view)
}

fn projected_bounds_rect(
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    viewport: VoxelEditorViewport,
    mvp: glam::Mat4,
) -> Option<[f32; 4]> {
    let mut min = glam::Vec2::splat(f32::INFINITY);
    let mut max = glam::Vec2::splat(f32::NEG_INFINITY);
    for corner in voxel_bounds_corners(bounds_min, bounds_max) {
        let clip = mvp * glam::Vec4::new(corner[0], corner[1], corner[2], 1.0);
        if clip.w.abs() <= f32::EPSILON {
            continue;
        }
        let ndc = clip.truncate() / clip.w;
        if !ndc.is_finite() {
            continue;
        }
        let screen = glam::Vec2::new(
            (ndc.x * 0.5 + 0.5) * viewport.width as f32,
            (1.0 - (ndc.y * 0.5 + 0.5)) * viewport.height as f32,
        );
        min = min.min(screen);
        max = max.max(screen);
    }
    if min.is_finite() && max.is_finite() {
        Some([min.x, min.y, max.x, max.y])
    } else {
        None
    }
}

fn voxel_bounds_corners(bounds_min: [f32; 3], bounds_max: [f32; 3]) -> [[f32; 3]; 8] {
    [
        [bounds_min[0], bounds_min[1], bounds_min[2]],
        [bounds_max[0], bounds_min[1], bounds_min[2]],
        [bounds_max[0], bounds_max[1], bounds_min[2]],
        [bounds_min[0], bounds_max[1], bounds_min[2]],
        [bounds_min[0], bounds_min[1], bounds_max[2]],
        [bounds_max[0], bounds_min[1], bounds_max[2]],
        [bounds_max[0], bounds_max[1], bounds_max[2]],
        [bounds_min[0], bounds_max[1], bounds_max[2]],
    ]
}

/// Build a [`VoxelSceneRenderData`] from a baked composition mesh export.
///
/// Each baked voxel is turned into a small cube mesh positioned using its integer
/// `world` coordinates scaled by `voxel_unit` (X/Y) and `voxel_unit + layer_gap` (Z).
/// Visible faces (those bordering an empty neighbor) are emitted with directional
/// shading multipliers that match the rest of the voxel rendering pipeline.
/// All voxels are grouped into a single object range keyed by the export's `id`.
pub fn baked_composition_to_scene_render_data(
    export: &game_data::defs::VoxelPanelCompositionMeshExportDef,
) -> Option<VoxelSceneRenderData> {
    if export.voxels.is_empty() {
        return None;
    }

    let unit = export.voxel_unit.max(0.01);
    let layer_step = unit + export.layer_gap.max(0.0);

    // Build a set of occupied voxel world positions for face culling.
    let occupied: std::collections::HashSet<[i32; 3]> =
        export.voxels.iter().map(|v| v.world).collect();

    // Face normals, corner offsets within a unit cube, and shading multipliers.
    // (delta, corner offsets in unit-cube space, shade multiplier)
    let face_directions: [([i32; 3], [[f32; 3]; 4], f32); 6] = [
        (
            [0, 0, -1],
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            0.64,
        ),
        (
            [0, 0, 1],
            [
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 0.0, 1.0],
            ],
            1.08,
        ),
        (
            [0, -1, 0],
            [
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
            ],
            0.82,
        ),
        (
            [1, 0, 0],
            [
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
            ],
            0.92,
        ),
        (
            [0, 1, 0],
            [
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            0.76,
        ),
        (
            [-1, 0, 0],
            [
                [0.0, 1.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
            ],
            0.70,
        ),
    ];

    let mut vertices: Vec<VoxelVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut world_min = glam::Vec3::splat(f32::INFINITY);
    let mut world_max = glam::Vec3::splat(f32::NEG_INFINITY);

    for baked in &export.voxels {
        let wx = baked.world[0] as f32 * unit;
        let wy = baked.world[1] as f32 * unit;
        let wz = baked.world[2] as f32 * layer_step;
        let vmin = [wx, wy, wz];
        let vmax = [wx + unit, wy + unit, wz + unit];

        let base_color = [
            baked.rgba[0] as f32 / 255.0,
            baked.rgba[1] as f32 / 255.0,
            baked.rgba[2] as f32 / 255.0,
            baked.rgba[3] as f32 / 255.0,
        ];

        for (delta, corners, shade) in &face_directions {
            let neighbor = [
                baked.world[0] + delta[0],
                baked.world[1] + delta[1],
                baked.world[2] + delta[2],
            ];
            if occupied.contains(&neighbor) {
                continue;
            }

            let face_color = [
                (base_color[0] * shade).clamp(0.0, 1.0),
                (base_color[1] * shade).clamp(0.0, 1.0),
                (base_color[2] * shade).clamp(0.0, 1.0),
                base_color[3],
            ];

            let base = vertices.len() as u32;
            for corner in corners {
                let pos = [
                    vmin[0] + (vmax[0] - vmin[0]) * corner[0],
                    vmin[1] + (vmax[1] - vmin[1]) * corner[1],
                    vmin[2] + (vmax[2] - vmin[2]) * corner[2],
                ];
                let p = glam::Vec3::from_array(pos);
                world_min = world_min.min(p);
                world_max = world_max.max(p);
                vertices.push(VoxelVertex {
                    position: pos,
                    color: face_color,
                });
            }
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    if vertices.is_empty() || indices.is_empty() {
        return None;
    }

    let bounds_min = world_min.to_array();
    let bounds_max = world_max.to_array();
    let index_count = indices.len() as u32;
    Some(VoxelSceneRenderData {
        bounds_min,
        bounds_max,
        object_ranges: vec![VoxelSceneObjectRange {
            object_key: format!("composition:{}", export.id),
            label: export.display_name.clone(),
            index_start: 0,
            index_count,
            bounds_min,
            bounds_max,
        }],
        vertices,
        indices,
    })
}

#[derive(Debug, Clone, Copy)]
pub struct ClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for ClearColor {
    fn default() -> Self {
        Self {
            r: 0.07,
            g: 0.10,
            b: 0.16,
            a: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldLighting {
    pub overlay_r: f32,
    pub overlay_g: f32,
    pub overlay_b: f32,
    pub overlay_a: f32,
}

impl WorldLighting {
    pub const fn new(overlay_r: f32, overlay_g: f32, overlay_b: f32, overlay_a: f32) -> Self {
        Self {
            overlay_r,
            overlay_g,
            overlay_b,
            overlay_a,
        }
    }

    pub fn is_visible(self) -> bool {
        self.overlay_a > 0.001
    }
}

#[derive(Debug, Clone)]
pub struct EditorShellRenderState {
    pub active_tool: usize,
    pub left_dock_open: bool,
    pub right_dock_open: bool,
    pub bottom_dock_open: bool,
    pub active_left_tab: usize,
    pub active_right_tab: usize,
    pub active_bottom_tab: usize,
    pub selected_asset_index: usize,
    pub selected_tile_name: String,
    pub selected_cell: (u32, u32),
    pub selected_role: String,
    pub selected_collision: String,
    pub hover_hint: String,
    pub status_message: String,
}

impl Default for EditorShellRenderState {
    fn default() -> Self {
        Self {
            active_tool: 0,
            left_dock_open: true,
            right_dock_open: true,
            bottom_dock_open: true,
            active_left_tab: 0,
            active_right_tab: 0,
            active_bottom_tab: 0,
            selected_asset_index: 0,
            selected_tile_name: "grass_bright".to_string(),
            selected_cell: (0, 0),
            selected_role: "grass".to_string(),
            selected_collision: "walkable".to_string(),
            hover_hint: "V Select | B Brush | T Tile Picker | F1/F2/F3 Docks | F5 Reload"
                .to_string(),
            status_message: "Native editor ready. Pick a tool or select an asset.".to_string(),
        }
    }
}

pub struct RenderBootstrap {
    window: Arc<Window>,
    gl_context: PossiblyCurrentContext,
    gl_surface: Surface<WindowSurface>,
    gl: glow::Context,
    clear_color: ClearColor,
    viewport_width: u32,
    viewport_height: u32,
    textured_quad: TexturedQuadPipeline,
    grid: GridPipeline,
    voxel_mesh: VoxelMeshPipeline,
    voxel_pick: VoxelPickFramebuffer,
    ui_overlay: UiOverlayPipeline,
    editor_shell_visible: bool,
    editor_shell_state: EditorShellRenderState,
    tile_map: Option<TileMapPipeline>,
    sprite_sheet: Option<SpritePipeline>,
    prop_sprite_sheet: Option<SpritePipeline>,
    render_world_width: f32,
    render_world_height: f32,
}

impl RenderBootstrap {
    pub fn new(
        window_bootstrap: WindowBootstrap,
        tile_map_data: Option<TileMapRenderData>,
        sprite_data: Option<SpriteRenderData>,
        prop_sprite_data: Option<SpriteRenderData>,
    ) -> anyhow::Result<Self> {
        let WindowBootstrap { window, gl_config } = window_bootstrap;
        let display = gl_config.display();
        let size = window.inner_size();
        let raw_window_handle = window
            .window_handle()
            .context("failed to access raw window handle")?
            .as_raw();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(raw_window_handle));
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(Some(Version::new(3, 0))))
            .build(Some(raw_window_handle));

        let not_current_gl_context = unsafe {
            display
                .create_context(&gl_config, &context_attributes)
                .or_else(|_| display.create_context(&gl_config, &fallback_context_attributes))
        }
        .context("failed to create OpenGL context")?;

        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            non_zero(size.width),
            non_zero(size.height),
        );

        let gl_surface = unsafe { display.create_window_surface(&gl_config, &attrs) }
            .context("failed to create OpenGL window surface")?;

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .context("failed to make OpenGL context current")?;

        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(non_zero(1)))
            .ok();

        let gl = unsafe {
            glow::Context::from_loader_function(|symbol| {
                let symbol = CString::new(symbol).expect("GL symbol should not contain NUL");
                display.get_proc_address(&symbol) as *const _
            })
        };

        let clear_color = ClearColor::default();

        unsafe {
            gl.viewport(0, 0, size.width as i32, size.height as i32);
            gl.clear_color(clear_color.r, clear_color.g, clear_color.b, clear_color.a);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        let textured_quad = TexturedQuadPipeline::new(&gl)
            .context("failed to build fallback textured quad pipeline")?;
        let grid = GridPipeline::new(&gl).context("failed to build debug grid pipeline")?;
        let voxel_mesh =
            VoxelMeshPipeline::new(&gl).context("failed to build voxel mesh pipeline")?;
        let voxel_pick =
            VoxelPickFramebuffer::new(&gl, 1, 1).context("failed to build voxel pick buffer")?;
        let ui_overlay =
            UiOverlayPipeline::new(&gl).context("failed to build editor shell overlay pipeline")?;
        let render_world_width = tile_map_data
            .as_ref()
            .map(|data| (data.map_width.max(1) * data.tile_width.max(1)) as f32)
            .unwrap_or(640.0);
        let render_world_height = tile_map_data
            .as_ref()
            .map(|data| (data.map_height.max(1) * data.tile_height.max(1)) as f32)
            .unwrap_or(480.0);

        let tile_map = match tile_map_data {
            Some(data) => Some(
                TileMapPipeline::new(&gl, &data).context("failed to build tile map pipeline")?,
            ),
            None => None,
        };
        let sprite_sheet = match sprite_data {
            Some(data) => {
                Some(SpritePipeline::new(&gl, &data).context("failed to build sprite pipeline")?)
            }
            None => None,
        };
        let prop_sprite_sheet = match prop_sprite_data {
            Some(data) => Some(
                SpritePipeline::new(&gl, &data).context("failed to build prop sprite pipeline")?,
            ),
            None => None,
        };

        let renderer = Self {
            window,
            gl_context,
            gl_surface,
            gl,
            clear_color,
            viewport_width: size.width.max(1),
            viewport_height: size.height.max(1),
            textured_quad,
            grid,
            voxel_mesh,
            voxel_pick,
            ui_overlay,
            editor_shell_visible: false,
            editor_shell_state: EditorShellRenderState::default(),
            tile_map,
            sprite_sheet,
            prop_sprite_sheet,
            render_world_width,
            render_world_height,
        };

        renderer.log_renderer_info();

        Ok(renderer)
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn set_editor_shell_visible(&mut self, visible: bool) {
        self.editor_shell_visible = visible;
        self.window.request_redraw();
    }

    pub fn set_editor_shell_state(&mut self, state: EditorShellRenderState) {
        self.editor_shell_state = state;
        self.window.request_redraw();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);

        self.gl_surface
            .resize(&self.gl_context, non_zero(width), non_zero(height));

        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
        }

        self.viewport_width = width;
        self.viewport_height = height;

        log::info!("renderer resize applied: {}x{}", width, height);
    }

    pub fn replace_tile_map(
        &mut self,
        tile_map_data: Option<TileMapRenderData>,
    ) -> anyhow::Result<()> {
        if let Some(existing) = self.tile_map.take() {
            unsafe {
                existing.destroy(&self.gl);
            }
        }

        match tile_map_data {
            Some(data) => {
                self.render_world_width = (data.map_width.max(1) * data.tile_width.max(1)) as f32;
                self.render_world_height =
                    (data.map_height.max(1) * data.tile_height.max(1)) as f32;
                self.tile_map = Some(
                    TileMapPipeline::new(&self.gl, &data)
                        .context("failed to rebuild hot-reloaded tile map pipeline")?,
                );
                log::info!(
                    "hot-reloaded tile map renderer: world={}x{} texture={}",
                    self.render_world_width,
                    self.render_world_height,
                    data.texture_path.display()
                );
            }
            None => {
                self.render_world_width = 640.0;
                self.render_world_height = 480.0;
                self.tile_map = None;
                log::warn!("hot reload produced no tile map; renderer will show fallback grid");
            }
        }

        self.window.request_redraw();
        Ok(())
    }

    pub fn render_frame(
        &self,
        frame_index: u64,
        voxel_scene: Option<&VoxelSceneRenderData>,
        sprites: &[SpriteInstance],
        prop_sprites: &[SpriteInstance],
        lighting: Option<WorldLighting>,
    ) -> anyhow::Result<()> {
        unsafe {
            self.gl.clear_color(
                self.clear_color.r,
                self.clear_color.g,
                self.clear_color.b,
                self.clear_color.a,
            );
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        if let Some(tile_map) = &self.tile_map {
            tile_map.draw(&self.gl, self.viewport_width, self.viewport_height);
        } else {
            self.grid
                .draw(&self.gl, self.viewport_width, self.viewport_height);
            self.textured_quad.draw(&self.gl);
        }

        if let Some(voxel_scene) = voxel_scene {
            self.voxel_mesh.draw(
                &self.gl,
                self.viewport_width,
                self.viewport_height,
                voxel_scene,
            );
        }

        if let Some(prop_sprite_sheet) = &self.prop_sprite_sheet {
            prop_sprite_sheet.draw(
                &self.gl,
                self.viewport_width,
                self.viewport_height,
                self.render_world_width,
                self.render_world_height,
                prop_sprites,
            );
        }

        if let Some(sprite_sheet) = &self.sprite_sheet {
            sprite_sheet.draw(
                &self.gl,
                self.viewport_width,
                self.viewport_height,
                self.render_world_width,
                self.render_world_height,
                sprites,
            );
        }

        if let Some(lighting) = lighting {
            if lighting.is_visible() {
                self.ui_overlay.draw_screen_tint(
                    &self.gl,
                    [
                        lighting.overlay_r,
                        lighting.overlay_g,
                        lighting.overlay_b,
                        lighting.overlay_a,
                    ],
                );
            }
        }

        if self.editor_shell_visible {
            self.ui_overlay
                .draw_editor_shell(&self.gl, &self.editor_shell_state);
        }

        self.gl_surface
            .swap_buffers(&self.gl_context)
            .context("failed to swap OpenGL buffers")?;

        if frame_index == 1 || frame_index % 300 == 0 {
            log::info!(
                "render frame {} on window {:?} viewport={}x{} tile_map={}",
                frame_index,
                self.window.id(),
                self.viewport_width,
                self.viewport_height,
                self.tile_map.is_some(),
            );
            if !prop_sprites.is_empty() || !sprites.is_empty() {
                log::info!(
                    "render frame {} sprite_count={} prop_sprite_count={}",
                    frame_index,
                    sprites.len(),
                    prop_sprites.len()
                );
            }
        }

        Ok(())
    }

    pub fn pick_voxel_object_gpu(
        &mut self,
        scene: &VoxelSceneRenderData,
        viewport: VoxelEditorViewport,
        pointer_x: u32,
        pointer_y: u32,
    ) -> anyhow::Result<Option<VoxelEditorPickResult>> {
        if scene.object_ranges.is_empty()
            || viewport.width == 0
            || viewport.height == 0
            || pointer_x >= viewport.width
            || pointer_y >= viewport.height
        {
            return Ok(None);
        }

        self.voxel_pick
            .ensure_size(&self.gl, viewport.width, viewport.height)?;
        self.voxel_pick.bind_for_render(&self.gl);
        self.voxel_mesh.draw_pick(&self.gl, viewport, scene);
        let pixel = self.voxel_pick.read_pixel(&self.gl, pointer_x, pointer_y);
        self.voxel_pick.unbind(&self.gl);
        unsafe {
            self.gl.viewport(
                0,
                0,
                self.viewport_width as i32,
                self.viewport_height as i32,
            );
            self.gl.clear_color(
                self.clear_color.r,
                self.clear_color.g,
                self.clear_color.b,
                self.clear_color.a,
            );
        }

        let pick_id = decode_pick_id(pixel);
        if pick_id == 0 {
            return Ok(None);
        }
        let Some(range) = scene.object_ranges.get((pick_id - 1) as usize) else {
            return Ok(None);
        };

        Ok(Some(VoxelEditorPickResult {
            object_key: range.object_key.clone(),
            label: range.label.clone(),
            distance_to_center: 0.0,
        }))
    }

    fn log_renderer_info(&self) {
        unsafe {
            let version = self.gl.get_parameter_string(glow::VERSION);
            let vendor = self.gl.get_parameter_string(glow::VENDOR);
            let renderer = self.gl.get_parameter_string(glow::RENDERER);
            log::info!(
                "OpenGL ready: version='{}' vendor='{}' renderer='{}'",
                version,
                vendor,
                renderer,
            );
        }
    }
}

impl Drop for RenderBootstrap {
    fn drop(&mut self) {
        unsafe {
            if let Some(tile_map) = &self.tile_map {
                tile_map.destroy(&self.gl);
            }
            if let Some(sprite_sheet) = &self.sprite_sheet {
                sprite_sheet.destroy(&self.gl);
            }
            if let Some(prop_sprite_sheet) = &self.prop_sprite_sheet {
                prop_sprite_sheet.destroy(&self.gl);
            }
            self.voxel_mesh.destroy(&self.gl);
            self.voxel_pick.destroy(&self.gl);
            self.ui_overlay.destroy(&self.gl);
            self.textured_quad.destroy(&self.gl);
            self.grid.destroy(&self.gl);
        }
    }
}

pub struct VoxelEditorGlowRenderer {
    voxel_mesh: VoxelMeshPipeline,
    voxel_pick: VoxelPickFramebuffer,
}

impl VoxelEditorGlowRenderer {
    pub fn new(gl: &glow::Context) -> anyhow::Result<Self> {
        Ok(Self {
            voxel_mesh: VoxelMeshPipeline::new(gl)?,
            voxel_pick: VoxelPickFramebuffer::new(gl, 1, 1)?,
        })
    }

    pub fn draw_viewport(
        &mut self,
        gl: &glow::Context,
        scene: &VoxelSceneRenderData,
        viewport: VoxelEditorViewport,
        selected_object_key: Option<&str>,
        ambient: f32,
    ) {
        let selected_range_index = selected_object_key.and_then(|key| {
            scene
                .object_ranges
                .iter()
                .position(|r| r.object_key == key)
                .map(|i| i as u32)
        });
        self.voxel_mesh
            .draw_editor_viewport(gl, viewport, scene, selected_range_index, ambient);
    }

    pub fn pick_viewport(
        &mut self,
        gl: &glow::Context,
        scene: &VoxelSceneRenderData,
        viewport: VoxelEditorViewport,
        pointer_x: u32,
        pointer_y: u32,
    ) -> anyhow::Result<Option<VoxelEditorPickResult>> {
        if scene.object_ranges.is_empty()
            || viewport.width == 0
            || viewport.height == 0
            || pointer_x >= viewport.width
            || pointer_y >= viewport.height
        {
            return Ok(None);
        }

        self.voxel_pick
            .ensure_size(gl, viewport.width, viewport.height)?;
        self.voxel_pick.bind_for_render(gl);
        self.voxel_mesh.draw_pick(gl, viewport, scene);
        let pixel = self.voxel_pick.read_pixel(gl, pointer_x, pointer_y);
        self.voxel_pick.unbind(gl);

        let pick_id = decode_pick_id(pixel);
        if pick_id == 0 {
            return Ok(None);
        }
        let Some(range) = scene.object_ranges.get((pick_id - 1) as usize) else {
            return Ok(None);
        };

        Ok(Some(VoxelEditorPickResult {
            object_key: range.object_key.clone(),
            label: range.label.clone(),
            distance_to_center: 0.0,
        }))
    }

    pub unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            self.voxel_mesh.destroy(gl);
            self.voxel_pick.destroy(gl);
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct VoxelMeshVertex {
    x: f32,
    y: f32,
    z: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

struct VoxelMeshPipeline {
    program: glow::Program,
    pick_program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
}

impl VoxelMeshPipeline {
    fn new(gl: &glow::Context) -> anyhow::Result<Self> {
        let program = create_program(gl, VOXEL_MESH_VERTEX_SHADER, VOXEL_MESH_FRAGMENT_SHADER)
            .context("voxel mesh shader program failed")?;
        let pick_program = create_program(gl, VOXEL_PICK_VERTEX_SHADER, VOXEL_PICK_FRAGMENT_SHADER)
            .context("voxel pick shader program failed")?;
        let vao = unsafe { gl.create_vertex_array() }
            .map_err(|e| anyhow::anyhow!("create voxel mesh VAO: {e}"))?;
        let vbo = unsafe { gl.create_buffer() }
            .map_err(|e| anyhow::anyhow!("create voxel mesh VBO: {e}"))?;
        let ebo = unsafe { gl.create_buffer() }
            .map_err(|e| anyhow::anyhow!("create voxel mesh EBO: {e}"))?;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_size(glow::ARRAY_BUFFER, 0, glow::DYNAMIC_DRAW);
            gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, 0, glow::DYNAMIC_DRAW);

            let stride = std::mem::size_of::<VoxelMeshVertex>() as i32;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                4,
                glow::FLOAT,
                false,
                stride,
                (3 * std::mem::size_of::<f32>()) as i32,
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }

        Ok(Self {
            program,
            pick_program,
            vao,
            vbo,
            ebo,
        })
    }

    fn draw(
        &self,
        gl: &glow::Context,
        viewport_width: u32,
        viewport_height: u32,
        scene: &VoxelSceneRenderData,
    ) {
        if scene.vertices.is_empty() || scene.indices.is_empty() {
            return;
        }

        let vertices = scene
            .vertices
            .iter()
            .map(|vertex| VoxelMeshVertex {
                x: vertex.position[0],
                y: vertex.position[1],
                z: vertex.position[2],
                r: vertex.color[0],
                g: vertex.color[1],
                b: vertex.color[2],
                a: vertex.color[3],
            })
            .collect::<Vec<_>>();

        let min = glam::Vec3::from_array(scene.bounds_min);
        let max = glam::Vec3::from_array(scene.bounds_max);
        let center = (min + max) * 0.5;
        let extent = (max - min).max(glam::Vec3::splat(1.0));
        let radius = extent.length().max(64.0);
        let aspect = viewport_width.max(1) as f32 / viewport_height.max(1) as f32;
        let eye = center
            + glam::Vec3::new(
                extent.x.max(extent.y) * 0.55,
                -radius * 1.8,
                extent.z.max(48.0) + radius * 0.75,
            );
        let view = glam::Mat4::look_at_rh(eye, center, glam::Vec3::Z);
        let projection = glam::Mat4::perspective_rh_gl(
            40.0_f32.to_radians(),
            aspect.max(0.1),
            1.0,
            radius * 8.0,
        );
        let mvp = projection * view;

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            gl.depth_mask(true);

            gl.use_program(Some(self.program));
            if let Some(location) = gl.get_uniform_location(self.program, "u_mvp") {
                gl.uniform_matrix_4_f32_slice(Some(&location), false, &mvp.to_cols_array());
            }

            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&vertices),
                glow::DYNAMIC_DRAW,
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                cast_slice(&scene.indices),
                glow::DYNAMIC_DRAW,
            );
            gl.draw_elements(
                glow::TRIANGLES,
                scene.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.use_program(None);
            gl.disable(glow::DEPTH_TEST);
        }
    }

    fn draw_editor_viewport(
        &self,
        gl: &glow::Context,
        viewport: VoxelEditorViewport,
        scene: &VoxelSceneRenderData,
        selected_range_index: Option<u32>,
        ambient: f32,
    ) {
        if scene.vertices.is_empty() || scene.indices.is_empty() {
            return;
        }

        let Some(mvp) = voxel_editor_view_projection(scene, viewport) else {
            return;
        };
        let vertices = scene
            .vertices
            .iter()
            .map(|vertex| VoxelMeshVertex {
                x: vertex.position[0],
                y: vertex.position[1],
                z: vertex.position[2],
                r: vertex.color[0],
                g: vertex.color[1],
                b: vertex.color[2],
                a: vertex.color[3],
            })
            .collect::<Vec<_>>();

        unsafe {
            gl.clear_color(0.047, 0.063, 0.090, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            gl.depth_mask(true);

            gl.use_program(Some(self.program));
            if let Some(location) = gl.get_uniform_location(self.program, "u_mvp") {
                gl.uniform_matrix_4_f32_slice(Some(&location), false, &mvp.to_cols_array());
            }
            if let Some(location) = gl.get_uniform_location(self.program, "u_ambient") {
                gl.uniform_1_f32(Some(&location), ambient.clamp(0.0, 1.0));
            }

            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&vertices),
                glow::DYNAMIC_DRAW,
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                cast_slice(&scene.indices),
                glow::DYNAMIC_DRAW,
            );
            gl.draw_elements(
                glow::TRIANGLES,
                scene.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );

            // Selection highlight pass: re-draw the selected object range using the flat-color
            // pick shader with a semi-transparent yellow overlay.
            if let Some(idx) = selected_range_index {
                if let Some(range) = scene.object_ranges.get(idx as usize) {
                    if range.index_count > 0 {
                        gl.enable(glow::BLEND);
                        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
                        gl.use_program(Some(self.pick_program));
<<<<<<< Updated upstream
                        if let Some(loc) =
                            gl.get_uniform_location(self.pick_program, "u_mvp")
                        {
                            gl.uniform_matrix_4_f32_slice(
                                Some(&loc),
                                false,
                                &mvp.to_cols_array(),
                            );
=======
                        if let Some(loc) = gl.get_uniform_location(self.pick_program, "u_mvp") {
                            gl.uniform_matrix_4_f32_slice(Some(&loc), false, &mvp.to_cols_array());
>>>>>>> Stashed changes
                        }
                        if let Some(loc) =
                            gl.get_uniform_location(self.pick_program, "u_pick_color")
                        {
                            gl.uniform_4_f32(Some(&loc), 1.0, 0.85, 0.2, 0.45);
                        }
                        gl.draw_elements(
                            glow::TRIANGLES,
                            range.index_count as i32,
                            glow::UNSIGNED_INT,
                            // Byte offset into the index buffer for this range.
                            (range.index_start as usize * std::mem::size_of::<u32>()) as i32,
                        );
                        gl.use_program(None);
                        gl.disable(glow::BLEND);
                    }
                }
            }

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.use_program(None);
            gl.disable(glow::DEPTH_TEST);
        }
    }

    fn draw_pick(
        &self,
        gl: &glow::Context,
        viewport: VoxelEditorViewport,
        scene: &VoxelSceneRenderData,
    ) {
        if scene.vertices.is_empty() || scene.indices.is_empty() || scene.object_ranges.is_empty() {
            return;
        }

        let Some(mvp) = voxel_editor_view_projection(scene, viewport) else {
            return;
        };
        let vertices = scene
            .vertices
            .iter()
            .map(|vertex| VoxelMeshVertex {
                x: vertex.position[0],
                y: vertex.position[1],
                z: vertex.position[2],
                r: vertex.color[0],
                g: vertex.color[1],
                b: vertex.color[2],
                a: vertex.color[3],
            })
            .collect::<Vec<_>>();

        unsafe {
            gl.viewport(0, 0, viewport.width as i32, viewport.height as i32);
            gl.clear_color(0.0, 0.0, 0.0, 0.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
            gl.depth_mask(true);

            gl.use_program(Some(self.pick_program));
            if let Some(location) = gl.get_uniform_location(self.pick_program, "u_mvp") {
                gl.uniform_matrix_4_f32_slice(Some(&location), false, &mvp.to_cols_array());
            }

            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&vertices),
                glow::DYNAMIC_DRAW,
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                cast_slice(&scene.indices),
                glow::DYNAMIC_DRAW,
            );

            for (index, range) in scene.object_ranges.iter().enumerate() {
                if range.index_count == 0 {
                    continue;
                }
                let color = encode_pick_id(index as u32 + 1);
                if let Some(location) = gl.get_uniform_location(self.pick_program, "u_pick_color") {
                    gl.uniform_4_f32(Some(&location), color[0], color[1], color[2], color[3]);
                }
                gl.draw_elements(
                    glow::TRIANGLES,
                    range.index_count as i32,
                    glow::UNSIGNED_INT,
                    (range.index_start as usize * std::mem::size_of::<u32>()) as i32,
                );
            }

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            gl.use_program(None);
            gl.disable(glow::DEPTH_TEST);
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_program(self.pick_program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
        }
    }
}

struct VoxelPickFramebuffer {
    framebuffer: glow::Framebuffer,
    color_texture: glow::Texture,
    depth_renderbuffer: glow::Renderbuffer,
    width: u32,
    height: u32,
}

impl VoxelPickFramebuffer {
    fn new(gl: &glow::Context, width: u32, height: u32) -> anyhow::Result<Self> {
        let framebuffer = unsafe { gl.create_framebuffer() }
            .map_err(|e| anyhow::anyhow!("create voxel pick framebuffer: {e}"))?;
        let color_texture = unsafe { gl.create_texture() }
            .map_err(|e| anyhow::anyhow!("create voxel pick color texture: {e}"))?;
        let depth_renderbuffer = unsafe { gl.create_renderbuffer() }
            .map_err(|e| anyhow::anyhow!("create voxel pick depth renderbuffer: {e}"))?;
        let mut buffer = Self {
            framebuffer,
            color_texture,
            depth_renderbuffer,
            width: 0,
            height: 0,
        };
        buffer.ensure_size(gl, width, height)?;
        Ok(buffer)
    }

    fn ensure_size(&mut self, gl: &glow::Context, width: u32, height: u32) -> anyhow::Result<()> {
        let width = width.max(1);
        let height = height.max(1);
        if self.width == width && self.height == height {
            return Ok(());
        }

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.color_texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );

            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(self.depth_renderbuffer));
            gl.renderbuffer_storage(
                glow::RENDERBUFFER,
                glow::DEPTH_COMPONENT24,
                width as i32,
                height as i32,
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(self.color_texture),
                0,
            );
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(self.depth_renderbuffer),
            );
            let status = gl.check_framebuffer_status(glow::FRAMEBUFFER);
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            gl.bind_renderbuffer(glow::RENDERBUFFER, None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            if status != glow::FRAMEBUFFER_COMPLETE {
                anyhow::bail!("voxel pick framebuffer incomplete: status={status:#x}");
            }
        }

        self.width = width;
        self.height = height;
        Ok(())
    }

    fn bind_for_render(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.framebuffer));
            gl.viewport(0, 0, self.width as i32, self.height as i32);
        }
    }

    fn read_pixel(&self, gl: &glow::Context, x: u32, y: u32) -> [u8; 4] {
        let mut pixel = [0_u8; 4];
        let read_y = self.height.saturating_sub(1).saturating_sub(y);
        unsafe {
            gl.read_pixels(
                x.min(self.width.saturating_sub(1)) as i32,
                read_y as i32,
                1,
                1,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelPackData::Slice(Some(&mut pixel)),
            );
        }
        pixel
    }

    fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_texture(self.color_texture);
            gl.delete_renderbuffer(self.depth_renderbuffer);
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct UiVertex {
    x: f32,
    y: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

struct UiOverlayPipeline {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
}

impl UiOverlayPipeline {
    fn new(gl: &glow::Context) -> anyhow::Result<Self> {
        let program = create_program(gl, UI_OVERLAY_VERTEX_SHADER, UI_OVERLAY_FRAGMENT_SHADER)
            .context("ui overlay shader program failed")?;
        let vao = unsafe { gl.create_vertex_array() }
            .map_err(|e| anyhow::anyhow!("create UI overlay VAO: {e}"))?;
        let vbo = unsafe { gl.create_buffer() }
            .map_err(|e| anyhow::anyhow!("create UI overlay VBO: {e}"))?;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_size(glow::ARRAY_BUFFER, 0, glow::DYNAMIC_DRAW);

            let stride = std::mem::size_of::<UiVertex>() as i32;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                4,
                glow::FLOAT,
                false,
                stride,
                (2 * std::mem::size_of::<f32>()) as i32,
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }

        Ok(Self { program, vao, vbo })
    }

    fn draw_screen_tint(&self, gl: &glow::Context, color: [f32; 4]) {
        let mut vertices = Vec::new();
        push_ui_rect(&mut vertices, -1.0, -1.0, 1.0, 1.0, color);

        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&vertices),
                glow::DYNAMIC_DRAW,
            );
            gl.draw_arrays(glow::TRIANGLES, 0, vertices.len() as i32);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }

    fn draw_editor_shell(&self, gl: &glow::Context, state: &EditorShellRenderState) {
        let mut vertices = Vec::new();

        // Phase 30: clean native editor chrome. Keep the game viewport readable,
        // collapse panels on demand, and render only actionable state.
        let top_y0 = 0.920;
        let left_x1 = if state.left_dock_open { -0.765 } else { -0.965 };
        let right_x0 = if state.right_dock_open { 0.765 } else { 0.965 };
        let bottom_y1 = if state.bottom_dock_open {
            -0.835
        } else {
            -0.965
        };

        push_ui_rect(
            &mut vertices,
            -1.00,
            top_y0,
            1.00,
            1.00,
            [0.040, 0.046, 0.060, 0.985],
        );

        if state.left_dock_open {
            push_ui_rect(
                &mut vertices,
                -1.00,
                bottom_y1,
                left_x1,
                top_y0,
                [0.050, 0.058, 0.074, 0.965],
            );
            draw_left_clean_panel(&mut vertices, state);
        } else {
            draw_collapsed_dock_tab(&mut vertices, -0.995, 0.735, "ASSETS");
        }

        if state.right_dock_open {
            push_ui_rect(
                &mut vertices,
                right_x0,
                bottom_y1,
                1.00,
                top_y0,
                [0.050, 0.058, 0.074, 0.965],
            );
            draw_right_clean_panel(&mut vertices, state);
        } else {
            draw_collapsed_dock_tab(&mut vertices, 0.905, 0.735, "INSPECT");
        }

        if state.bottom_dock_open {
            push_ui_rect(
                &mut vertices,
                left_x1,
                -1.00,
                right_x0,
                bottom_y1,
                [0.040, 0.046, 0.060, 0.950],
            );
            draw_bottom_clean_panel(&mut vertices, state);
        } else {
            push_ui_rect(
                &mut vertices,
                -0.360,
                -0.992,
                0.360,
                -0.960,
                [0.050, 0.060, 0.078, 0.940],
            );
            draw_text(
                &mut vertices,
                -0.335,
                -0.972,
                state.status_message.as_str(),
                0.0023,
                [0.72, 0.82, 0.92, 0.96],
            );
        }

        // Thin viewport frame only; avoid heavy tinted overlays on top of gameplay.
        push_ui_rect(
            &mut vertices,
            left_x1 - 0.003,
            bottom_y1,
            left_x1 + 0.003,
            top_y0,
            [0.20, 0.27, 0.36, 0.78],
        );
        push_ui_rect(
            &mut vertices,
            right_x0 - 0.003,
            bottom_y1,
            right_x0 + 0.003,
            top_y0,
            [0.20, 0.27, 0.36, 0.78],
        );
        push_ui_rect(
            &mut vertices,
            left_x1,
            bottom_y1 - 0.003,
            right_x0,
            bottom_y1 + 0.003,
            [0.20, 0.27, 0.36, 0.78],
        );
        push_ui_rect(
            &mut vertices,
            -1.000,
            top_y0 - 0.003,
            1.000,
            top_y0 + 0.003,
            [0.20, 0.27, 0.36, 0.78],
        );

        draw_top_toolbar(&mut vertices, state);

        if vertices.is_empty() {
            return;
        }

        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&vertices),
                glow::DYNAMIC_DRAW,
            );
            gl.draw_arrays(glow::TRIANGLES, 0, vertices.len() as i32);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
    }
}

fn push_ui_rect(vertices: &mut Vec<UiVertex>, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 4]) {
    let [r, g, b, a] = color;
    vertices.extend_from_slice(&[
        UiVertex {
            x: x0,
            y: y0,
            r,
            g,
            b,
            a,
        },
        UiVertex {
            x: x1,
            y: y0,
            r,
            g,
            b,
            a,
        },
        UiVertex {
            x: x1,
            y: y1,
            r,
            g,
            b,
            a,
        },
        UiVertex {
            x: x0,
            y: y0,
            r,
            g,
            b,
            a,
        },
        UiVertex {
            x: x1,
            y: y1,
            r,
            g,
            b,
            a,
        },
        UiVertex {
            x: x0,
            y: y1,
            r,
            g,
            b,
            a,
        },
    ]);
}

const TOOL_LABELS: [&str; 10] = [
    "Select",
    "Pan",
    "Brush",
    "Erase",
    "Fill",
    "Pick",
    "Tiles",
    "Collision",
    "Assets",
    "Play",
];

fn draw_top_toolbar(vertices: &mut Vec<UiVertex>, state: &EditorShellRenderState) {
    push_ui_rect(
        vertices,
        -0.990,
        0.932,
        -0.944,
        0.984,
        [0.070, 0.083, 0.108, 0.96],
    );
    draw_text(
        vertices,
        -0.978,
        0.964,
        "SR",
        0.0032,
        [0.74, 0.84, 0.96, 0.98],
    );

    let mut x = -0.925_f32;
    for index in 0..10 {
        draw_toolbar_icon_state(vertices, x, 0.932, index, state.active_tool == index);
        x += 0.052;
    }

    draw_status_badge(
        vertices,
        -0.365,
        0.945,
        TOOL_LABELS
            .get(state.active_tool)
            .copied()
            .unwrap_or("Tool"),
    );
    draw_status_badge(vertices, -0.255, 0.945, "F1 Assets");
    draw_status_badge(vertices, -0.125, 0.945, "F2 Inspect");
    draw_status_badge(vertices, 0.025, 0.945, "F3 Log");
    draw_text(
        vertices,
        0.230,
        0.965,
        state.hover_hint.as_str(),
        0.0024,
        [0.72, 0.80, 0.90, 0.95],
    );
}

fn draw_left_clean_panel(vertices: &mut Vec<UiVertex>, state: &EditorShellRenderState) {
    draw_clean_panel_title(vertices, -0.982, 0.865, "Assets", "F1 hides");
    draw_clean_tab_row(
        vertices,
        -0.982,
        0.808,
        &["Project", "Textures", "Maps"],
        state.active_left_tab,
    );
    draw_clean_asset_row(
        vertices,
        -0.972,
        0.700,
        "Terrain Atlas",
        "active texture atlas",
        state.selected_asset_index == 0,
    );
    draw_clean_asset_row(
        vertices,
        -0.972,
        0.570,
        "Player Walk",
        "sprite sheet",
        state.selected_asset_index == 1,
    );
    draw_clean_asset_row(
        vertices,
        -0.972,
        0.440,
        "Ocean Props",
        "static props",
        state.selected_asset_index == 2,
    );
    draw_clean_panel_title(vertices, -0.982, 0.295, "Workflow", "native path");
    draw_step_chip(vertices, -0.962, 0.225, "1 Pick asset");
    draw_step_chip(vertices, -0.962, 0.165, "2 Pick tile");
    draw_step_chip(vertices, -0.962, 0.105, "3 Edit metadata");
    draw_step_chip(vertices, -0.962, 0.045, "4 Save + F5");
}

fn draw_right_clean_panel(vertices: &mut Vec<UiVertex>, state: &EditorShellRenderState) {
    draw_clean_panel_title(vertices, 0.782, 0.865, "Inspector", "F2 hides");
    draw_clean_tab_row(
        vertices,
        0.782,
        0.808,
        &["Tile", "Seams", "Export"],
        state.active_right_tab,
    );
    draw_text(
        vertices,
        0.798,
        0.720,
        "Selected",
        0.0031,
        [0.58, 0.68, 0.80, 0.96],
    );
    draw_text(
        vertices,
        0.798,
        0.675,
        state.selected_tile_name.as_str(),
        0.0032,
        [0.84, 0.90, 0.98, 0.98],
    );
    let cell = format!("cell {},{}", state.selected_cell.0, state.selected_cell.1);
    draw_text(
        vertices,
        0.798,
        0.632,
        cell.as_str(),
        0.0028,
        [0.68, 0.76, 0.86, 0.96],
    );
    let role = format!("role {}", state.selected_role);
    draw_text(
        vertices,
        0.798,
        0.592,
        role.as_str(),
        0.0025,
        [0.68, 0.80, 0.70, 0.96],
    );
    let collision = format!("collision {}", state.selected_collision);
    draw_text(
        vertices,
        0.798,
        0.555,
        collision.as_str(),
        0.0025,
        [0.78, 0.70, 0.58, 0.96],
    );
    draw_clean_preview(vertices, 0.798, 0.445, "Tile", 0);
    draw_clean_preview(vertices, 0.890, 0.445, "Seam", 1);
    draw_clean_preview(vertices, 0.798, 0.245, "Atlas", 2);
    draw_clean_panel_title(vertices, 0.782, 0.075, "Actions", "metadata");
    draw_icon_action(vertices, 0.798, 0.005, "Role");
    draw_icon_action(vertices, 0.875, 0.005, "Block");
    draw_icon_action(vertices, 0.798, -0.058, "Clean");
    draw_icon_action(vertices, 0.875, -0.058, "Export");
}

fn draw_bottom_clean_panel(vertices: &mut Vec<UiVertex>, state: &EditorShellRenderState) {
    draw_clean_tab_row(
        vertices,
        -0.720,
        -0.872,
        &["Console", "Validation", "Hot Reload", "Runtime"],
        state.active_bottom_tab,
    );
    draw_text(
        vertices,
        -0.705,
        -0.930,
        state.status_message.as_str(),
        0.0029,
        [0.76, 0.86, 0.96, 0.98],
    );
    draw_text(
        vertices,
        -0.705,
        -0.970,
        "Click toolbar icons or use shortcuts: V B E G I T C A P. F5 reloads content.",
        0.0025,
        [0.58, 0.68, 0.80, 0.96],
    );
}

fn draw_collapsed_dock_tab(vertices: &mut Vec<UiVertex>, x: f32, y: f32, label: &str) {
    push_ui_rect(
        vertices,
        x,
        y,
        x + 0.090,
        y + 0.040,
        [0.070, 0.085, 0.112, 0.96],
    );
    draw_text(
        vertices,
        x + 0.010,
        y + 0.026,
        label,
        0.0024,
        [0.76, 0.86, 0.96, 0.96],
    );
}

fn draw_clean_panel_title(vertices: &mut Vec<UiVertex>, x: f32, y: f32, title: &str, hint: &str) {
    push_ui_rect(
        vertices,
        x,
        y,
        x + 0.195,
        y + 0.052,
        [0.080, 0.098, 0.128, 0.98],
    );
    push_ui_rect(
        vertices,
        x,
        y,
        x + 0.008,
        y + 0.052,
        [0.40, 0.58, 0.78, 0.98],
    );
    draw_text(
        vertices,
        x + 0.016,
        y + 0.034,
        title,
        0.0032,
        [0.84, 0.90, 0.98, 0.98],
    );
    draw_text(
        vertices,
        x + 0.105,
        y + 0.030,
        hint,
        0.0021,
        [0.56, 0.66, 0.78, 0.94],
    );
}

fn draw_clean_tab_row(
    vertices: &mut Vec<UiVertex>,
    x: f32,
    y: f32,
    labels: &[&str],
    active_index: usize,
) {
    let mut tx = x;
    for (index, label) in labels.iter().enumerate() {
        let w = ((*label).len() as f32 * 0.014).clamp(0.070, 0.130);
        let color = if index == active_index {
            [0.145, 0.180, 0.235, 0.98]
        } else {
            [0.075, 0.088, 0.112, 0.94]
        };
        push_ui_rect(vertices, tx, y, tx + w, y + 0.045, color);
        draw_text(
            vertices,
            tx + 0.009,
            y + 0.029,
            label,
            0.0024,
            [0.78, 0.86, 0.96, 0.98],
        );
        tx += w + 0.008;
    }
}

fn draw_clean_asset_row(
    vertices: &mut Vec<UiVertex>,
    x: f32,
    y: f32,
    title: &str,
    sub: &str,
    active: bool,
) {
    let border = if active {
        [0.42, 0.61, 0.82, 0.95]
    } else {
        [0.18, 0.23, 0.30, 0.90]
    };
    push_ui_rect(vertices, x, y, x + 0.210, y + 0.092, border);
    push_ui_rect(
        vertices,
        x + 0.004,
        y + 0.004,
        x + 0.206,
        y + 0.088,
        [0.066, 0.078, 0.102, 0.98],
    );
    push_ui_rect(
        vertices,
        x + 0.014,
        y + 0.020,
        x + 0.060,
        y + 0.072,
        [0.40, 0.58, 0.19, 0.98],
    );
    draw_text(
        vertices,
        x + 0.074,
        y + 0.061,
        title,
        0.0027,
        [0.84, 0.90, 0.98, 0.98],
    );
    draw_text(
        vertices,
        x + 0.074,
        y + 0.031,
        sub,
        0.0022,
        [0.58, 0.68, 0.80, 0.94],
    );
}

fn draw_step_chip(vertices: &mut Vec<UiVertex>, x: f32, y: f32, label: &str) {
    push_ui_rect(
        vertices,
        x,
        y,
        x + 0.160,
        y + 0.040,
        [0.068, 0.082, 0.106, 0.94],
    );
    draw_text(
        vertices,
        x + 0.010,
        y + 0.026,
        label,
        0.0024,
        [0.70, 0.80, 0.90, 0.96],
    );
}

fn draw_clean_preview(vertices: &mut Vec<UiVertex>, x: f32, y: f32, title: &str, mode: usize) {
    push_ui_rect(
        vertices,
        x,
        y,
        x + 0.078,
        y + 0.145,
        [0.080, 0.095, 0.124, 0.96],
    );
    draw_text(
        vertices,
        x + 0.009,
        y + 0.126,
        title,
        0.0024,
        [0.78, 0.86, 0.96, 0.96],
    );
    match mode {
        0 => draw_mini_tile(
            vertices,
            x + 0.020,
            y + 0.034,
            0.042,
            0.052,
            [0.42, 0.58, 0.16, 0.98],
        ),
        1 => draw_seam_grid(vertices, x + 0.016, y + 0.032),
        _ => draw_atlas_mini(vertices, x + 0.010, y + 0.034),
    }
}

fn draw_mini_tile(vertices: &mut Vec<UiVertex>, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
    push_ui_rect(vertices, x, y, x + w, y + h, color);
    push_ui_rect(vertices, x, y, x + w, y + 0.004, [0.70, 0.78, 0.44, 0.90]);
    push_ui_rect(
        vertices,
        x,
        y + h - 0.004,
        x + w,
        y + h,
        [0.24, 0.38, 0.10, 0.90],
    );
    push_ui_rect(vertices, x, y, x + 0.004, y + h, [0.70, 0.78, 0.44, 0.90]);
    push_ui_rect(
        vertices,
        x + w - 0.004,
        y,
        x + w,
        y + h,
        [0.24, 0.38, 0.10, 0.90],
    );
}

fn draw_seam_grid(vertices: &mut Vec<UiVertex>, x: f32, y: f32) {
    let s = 0.020;
    for row in 0..3 {
        for col in 0..3 {
            let shade = if (row + col) % 2 == 0 {
                [0.38, 0.56, 0.16, 0.98]
            } else {
                [0.34, 0.50, 0.14, 0.98]
            };
            draw_mini_tile(
                vertices,
                x + col as f32 * s,
                y + row as f32 * s,
                s,
                s,
                shade,
            );
        }
    }
    push_ui_rect(
        vertices,
        x + s - 0.0015,
        y,
        x + s + 0.0015,
        y + s * 3.0,
        [0.85, 0.40, 0.30, 0.90],
    );
    push_ui_rect(
        vertices,
        x + s * 2.0 - 0.0015,
        y,
        x + s * 2.0 + 0.0015,
        y + s * 3.0,
        [0.85, 0.40, 0.30, 0.90],
    );
}

fn draw_atlas_mini(vertices: &mut Vec<UiVertex>, x: f32, y: f32) {
    let w = 0.066;
    let h = 0.060;
    push_ui_rect(vertices, x, y, x + w, y + h, [0.38, 0.56, 0.16, 0.98]);
    push_ui_rect(
        vertices,
        x,
        y,
        x + w,
        y + h * 0.30,
        [0.80, 0.62, 0.34, 0.98],
    );
    push_ui_rect(
        vertices,
        x,
        y,
        x + w,
        y + h * 0.13,
        [0.10, 0.50, 0.70, 0.98],
    );
    push_ui_rect(
        vertices,
        x + 0.004,
        y + h - 0.018,
        x + 0.020,
        y + h - 0.002,
        [0.75, 0.88, 0.98, 0.98],
    );
}

fn draw_icon_action(vertices: &mut Vec<UiVertex>, x: f32, y: f32, title: &str) {
    push_ui_rect(
        vertices,
        x,
        y,
        x + 0.064,
        y + 0.045,
        [0.095, 0.120, 0.158, 0.96],
    );
    push_ui_rect(
        vertices,
        x + 0.010,
        y + 0.014,
        x + 0.024,
        y + 0.030,
        [0.44, 0.62, 0.82, 0.96],
    );
    draw_text(
        vertices,
        x + 0.030,
        y + 0.029,
        title,
        0.0021,
        [0.80, 0.88, 0.96, 0.96],
    );
}

fn draw_status_badge(vertices: &mut Vec<UiVertex>, x: f32, y: f32, title: &str) {
    let w = (title.len() as f32 * 0.014).clamp(0.080, 0.140);
    push_ui_rect(
        vertices,
        x,
        y,
        x + w,
        y + 0.034,
        [0.070, 0.092, 0.120, 0.94],
    );
    draw_text(
        vertices,
        x + 0.008,
        y + 0.023,
        title,
        0.0022,
        [0.70, 0.82, 0.94, 0.95],
    );
}

fn draw_toolbar_icon_state(
    vertices: &mut Vec<UiVertex>,
    x: f32,
    y: f32,
    index: usize,
    active: bool,
) {
    let button = if active {
        [0.180, 0.230, 0.305, 0.98]
    } else {
        [0.085, 0.100, 0.130, 0.98]
    };
    let mark = [0.72, 0.78, 0.86, 0.98];
    let accent = if active {
        [0.55, 0.76, 0.98, 0.98]
    } else {
        [0.42, 0.58, 0.76, 0.98]
    };
    let w = 0.042;
    let h = 0.046;
    push_ui_rect(vertices, x, y, x + w, y + h, button);
    if active {
        push_ui_rect(vertices, x, y, x + w, y + 0.004, accent);
    }
    let cx = x + 0.021;
    let cy = y + 0.023;
    match index {
        0 => {
            push_ui_rect(
                vertices,
                cx - 0.010,
                cy - 0.012,
                cx - 0.004,
                cy + 0.014,
                mark,
            );
            push_ui_rect(
                vertices,
                cx - 0.004,
                cy - 0.003,
                cx + 0.012,
                cy + 0.004,
                mark,
            );
        }
        1 => {
            push_ui_rect(
                vertices,
                cx - 0.012,
                cy - 0.012,
                cx + 0.008,
                cy + 0.000,
                mark,
            );
            push_ui_rect(
                vertices,
                cx + 0.006,
                cy - 0.002,
                cx + 0.014,
                cy + 0.014,
                accent,
            );
        }
        2 => push_ui_rect(
            vertices,
            cx - 0.013,
            cy - 0.010,
            cx + 0.013,
            cy + 0.010,
            mark,
        ),
        3 => {
            push_ui_rect(
                vertices,
                cx - 0.012,
                cy - 0.003,
                cx + 0.012,
                cy + 0.012,
                mark,
            );
            push_ui_rect(
                vertices,
                cx - 0.005,
                cy - 0.014,
                cx + 0.005,
                cy - 0.005,
                accent,
            );
        }
        4 => {
            push_ui_rect(
                vertices,
                cx - 0.012,
                cy - 0.012,
                cx - 0.004,
                cy + 0.012,
                mark,
            );
            push_ui_rect(
                vertices,
                cx - 0.004,
                cy + 0.005,
                cx + 0.014,
                cy + 0.012,
                accent,
            );
        }
        5 => {
            push_ui_rect(
                vertices,
                cx - 0.014,
                cy - 0.014,
                cx + 0.014,
                cy - 0.010,
                mark,
            );
            push_ui_rect(
                vertices,
                cx - 0.014,
                cy + 0.010,
                cx + 0.014,
                cy + 0.014,
                mark,
            );
            push_ui_rect(
                vertices,
                cx - 0.014,
                cy - 0.014,
                cx - 0.010,
                cy + 0.014,
                mark,
            );
            push_ui_rect(
                vertices,
                cx + 0.010,
                cy - 0.014,
                cx + 0.014,
                cy + 0.014,
                mark,
            );
        }
        6 => {
            push_ui_rect(
                vertices,
                cx - 0.003,
                cy - 0.014,
                cx + 0.003,
                cy + 0.014,
                mark,
            );
            push_ui_rect(
                vertices,
                cx - 0.014,
                cy - 0.003,
                cx + 0.014,
                cy + 0.003,
                mark,
            );
        }
        7 => {
            push_ui_rect(
                vertices,
                cx - 0.014,
                cy - 0.014,
                cx - 0.001,
                cy + 0.014,
                accent,
            );
            push_ui_rect(
                vertices,
                cx + 0.002,
                cy - 0.014,
                cx + 0.014,
                cy + 0.014,
                mark,
            );
        }
        8 => {
            push_ui_rect(
                vertices,
                cx - 0.014,
                cy - 0.011,
                cx - 0.005,
                cy + 0.011,
                mark,
            );
            push_ui_rect(
                vertices,
                cx - 0.001,
                cy - 0.011,
                cx + 0.009,
                cy + 0.011,
                accent,
            );
            push_ui_rect(
                vertices,
                cx + 0.012,
                cy - 0.011,
                cx + 0.015,
                cy + 0.011,
                mark,
            );
        }
        _ => {
            push_ui_rect(
                vertices,
                cx - 0.006,
                cy + 0.004,
                cx + 0.006,
                cy + 0.016,
                mark,
            );
            push_ui_rect(
                vertices,
                cx - 0.012,
                cy - 0.016,
                cx + 0.012,
                cy + 0.002,
                accent,
            );
        }
    }
}

fn draw_text(
    vertices: &mut Vec<UiVertex>,
    x: f32,
    y: f32,
    text: &str,
    scale: f32,
    color: [f32; 4],
) {
    let mut cursor = x;
    for ch in text.chars() {
        if ch == ' ' {
            cursor += scale * 4.0;
            continue;
        }

        let glyph = glyph_rows(ch.to_ascii_uppercase());
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..5 {
                if (*bits & (1 << (4 - col))) != 0 {
                    let x0 = cursor + col as f32 * scale;
                    let y0 = y - row as f32 * scale;
                    push_ui_rect(
                        vertices,
                        x0,
                        y0 - scale,
                        x0 + scale * 0.82,
                        y0 - scale * 0.18,
                        color,
                    );
                }
            }
        }
        cursor += scale * 6.0;
    }
}

fn glyph_rows(ch: char) -> [u8; 7] {
    match ch {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b01010, 0b10001,
        ],
        'Y' => [
            0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        ':' => [
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '_' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111,
        ],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        _ => [0, 0, 0, 0, 0, 0, 0],
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct TileVertex {
    x: f32,
    y: f32,
    u: f32,
    v: f32,
}

struct TileMapPipeline {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    texture: glow::Texture,
    vertex_count: i32,
    world_width: f32,
    world_height: f32,
}

impl TileMapPipeline {
    fn new(gl: &glow::Context, data: &TileMapRenderData) -> anyhow::Result<Self> {
        let program = create_program(gl, TILE_MAP_VERTEX_SHADER, TILE_MAP_FRAGMENT_SHADER)
            .context("tile map shader program failed")?;

        let world_width = (data.map_width.max(1) * data.tile_width.max(1)) as f32;
        let world_height = (data.map_height.max(1) * data.tile_height.max(1)) as f32;
        let vertices = build_tile_vertices(data, world_width, world_height);

        let vao = unsafe { gl.create_vertex_array() }
            .map_err(|e| anyhow::anyhow!("create tile map VAO: {e}"))?;
        let vbo = unsafe { gl.create_buffer() }
            .map_err(|e| anyhow::anyhow!("create tile map VBO: {e}"))?;
        let texture = create_texture_from_file(gl, &data.texture_path).with_context(|| {
            format!("failed to load tile atlas {}", data.texture_path.display())
        })?;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, cast_slice(&vertices), glow::STATIC_DRAW);

            let stride = std::mem::size_of::<TileVertex>() as i32;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                stride,
                (2 * std::mem::size_of::<f32>()) as i32,
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }

        log::info!(
            "tile map pipeline ready: vertices={} world={}x{} atlas={}",
            vertices.len(),
            world_width,
            world_height,
            data.texture_path.display()
        );

        Ok(Self {
            program,
            vao,
            vbo,
            texture,
            vertex_count: vertices.len() as i32,
            world_width,
            world_height,
        })
    }

    fn draw(&self, gl: &glow::Context, viewport_width: u32, viewport_height: u32) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));

            if let Some(location) = gl.get_uniform_location(self.program, "u_texture") {
                gl.uniform_1_i32(Some(&location), 0);
            }
            if let Some(location) = gl.get_uniform_location(self.program, "u_viewport_size") {
                gl.uniform_2_f32(
                    Some(&location),
                    viewport_width.max(1) as f32,
                    viewport_height.max(1) as f32,
                );
            }
            if let Some(location) = gl.get_uniform_location(self.program, "u_world_size") {
                gl.uniform_2_f32(
                    Some(&location),
                    self.world_width.max(1.0),
                    self.world_height.max(1.0),
                );
            }

            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, self.vertex_count);
            gl.bind_vertex_array(None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.use_program(None);
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_texture(self.texture);
        }
    }
}

fn build_tile_vertices(
    data: &TileMapRenderData,
    world_width: f32,
    world_height: f32,
) -> Vec<TileVertex> {
    let mut vertices = Vec::with_capacity(data.tiles.len() * 6);
    let tile_width = data.tile_width.max(1) as f32;
    let tile_height = data.tile_height.max(1) as f32;
    let columns = data.atlas_columns.max(1) as f32;
    let rows = data.atlas_rows.max(1) as f32;

    for tile in &data.tiles {
        let x0 = tile.x as f32 * tile_width - world_width * 0.5;
        let y0 = tile.y as f32 * tile_height - world_height * 0.5;
        let x1 = x0 + tile_width;
        let y1 = y0 + tile_height;

        // Slightly inset UVs inside each atlas cell to reduce transparent gutter bleed
        // and neighbor-cell sampling on scaled OpenGL output.
        let inset = 0.003_f32;
        let u0 = (tile.atlas_x as f32 + inset) / columns;
        let u1 = (tile.atlas_x as f32 + 1.0 - inset) / columns;
        let v_top = 1.0 - (tile.atlas_y as f32 + inset) / rows;
        let v_bottom = 1.0 - (tile.atlas_y as f32 + 1.0 - inset) / rows;

        vertices.extend_from_slice(&[
            TileVertex {
                x: x0,
                y: y0,
                u: u0,
                v: v_top,
            },
            TileVertex {
                x: x1,
                y: y0,
                u: u1,
                v: v_top,
            },
            TileVertex {
                x: x1,
                y: y1,
                u: u1,
                v: v_bottom,
            },
            TileVertex {
                x: x0,
                y: y0,
                u: u0,
                v: v_top,
            },
            TileVertex {
                x: x1,
                y: y1,
                u: u1,
                v: v_bottom,
            },
            TileVertex {
                x: x0,
                y: y1,
                u: u0,
                v: v_bottom,
            },
        ]);
    }

    vertices
}

fn create_texture_from_file(
    gl: &glow::Context,
    path: &std::path::Path,
) -> anyhow::Result<glow::Texture> {
    let image = image::open(path)
        .with_context(|| format!("failed to open texture {}", path.display()))?
        .flipv()
        .to_rgba8();
    let (width, height) = image.dimensions();
    let pixels = image.into_raw();

    let texture = unsafe { gl.create_texture() }
        .map_err(|e| anyhow::anyhow!("create tile atlas texture: {e}"))?;

    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            width as i32,
            height as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(Some(&pixels)),
        );
        gl.bind_texture(glow::TEXTURE_2D, None);
    }

    Ok(texture)
}

struct SpritePipeline {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    texture: glow::Texture,
    atlas_columns: u32,
    atlas_rows: u32,
}

impl SpritePipeline {
    fn new(gl: &glow::Context, data: &SpriteRenderData) -> anyhow::Result<Self> {
        let program = create_program(gl, TILE_MAP_VERTEX_SHADER, TILE_MAP_FRAGMENT_SHADER)
            .context("sprite shader program failed")?;
        let vao = unsafe { gl.create_vertex_array() }
            .map_err(|e| anyhow::anyhow!("create sprite VAO: {e}"))?;
        let vbo =
            unsafe { gl.create_buffer() }.map_err(|e| anyhow::anyhow!("create sprite VBO: {e}"))?;
        let texture = create_texture_from_file(gl, &data.texture_path).with_context(|| {
            format!(
                "failed to load sprite sheet {}",
                data.texture_path.display()
            )
        })?;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_size(glow::ARRAY_BUFFER, 0, glow::DYNAMIC_DRAW);

            let stride = std::mem::size_of::<TileVertex>() as i32;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                stride,
                (2 * std::mem::size_of::<f32>()) as i32,
            );

            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }

        log::info!(
            "sprite pipeline ready: atlas={} columns={} rows={} cell={}x{}",
            data.texture_path.display(),
            data.atlas_columns,
            data.atlas_rows,
            data.sprite_width,
            data.sprite_height,
        );

        Ok(Self {
            program,
            vao,
            vbo,
            texture,
            atlas_columns: data.atlas_columns.max(1),
            atlas_rows: data.atlas_rows.max(1),
        })
    }

    fn draw(
        &self,
        gl: &glow::Context,
        viewport_width: u32,
        viewport_height: u32,
        world_width: f32,
        world_height: f32,
        sprites: &[SpriteInstance],
    ) {
        if sprites.is_empty() {
            return;
        }

        let vertices = build_sprite_vertices(
            sprites,
            world_width.max(1.0),
            world_height.max(1.0),
            self.atlas_columns,
            self.atlas_rows,
        );

        unsafe {
            gl.use_program(Some(self.program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));

            if let Some(location) = gl.get_uniform_location(self.program, "u_texture") {
                gl.uniform_1_i32(Some(&location), 0);
            }
            if let Some(location) = gl.get_uniform_location(self.program, "u_viewport_size") {
                gl.uniform_2_f32(
                    Some(&location),
                    viewport_width.max(1) as f32,
                    viewport_height.max(1) as f32,
                );
            }
            if let Some(location) = gl.get_uniform_location(self.program, "u_world_size") {
                gl.uniform_2_f32(Some(&location), world_width.max(1.0), world_height.max(1.0));
            }

            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&vertices),
                glow::DYNAMIC_DRAW,
            );
            gl.draw_arrays(glow::TRIANGLES, 0, vertices.len() as i32);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.use_program(None);
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_texture(self.texture);
        }
    }
}

fn build_sprite_vertices(
    sprites: &[SpriteInstance],
    world_width: f32,
    world_height: f32,
    atlas_columns: u32,
    atlas_rows: u32,
) -> Vec<TileVertex> {
    let mut vertices = Vec::with_capacity(sprites.len() * 6);
    let columns = atlas_columns.max(1) as f32;
    let rows = atlas_rows.max(1) as f32;

    for sprite in sprites {
        let x0 = sprite.x - world_width * 0.5;
        let y0 = sprite.y - world_height * 0.5;
        let x1 = x0 + sprite.w.max(1.0);
        let y1 = y0 + sprite.h.max(1.0);

        let inset = 0.003_f32;
        let u0 = (sprite.atlas_x as f32 + inset) / columns;
        let u1 = (sprite.atlas_x as f32 + 1.0 - inset) / columns;
        let v_top = 1.0 - (sprite.atlas_y as f32 + inset) / rows;
        let v_bottom = 1.0 - (sprite.atlas_y as f32 + 1.0 - inset) / rows;

        vertices.extend_from_slice(&[
            TileVertex {
                x: x0,
                y: y0,
                u: u0,
                v: v_top,
            },
            TileVertex {
                x: x1,
                y: y0,
                u: u1,
                v: v_top,
            },
            TileVertex {
                x: x1,
                y: y1,
                u: u1,
                v: v_bottom,
            },
            TileVertex {
                x: x0,
                y: y0,
                u: u0,
                v: v_top,
            },
            TileVertex {
                x: x1,
                y: y1,
                u: u1,
                v: v_bottom,
            },
            TileVertex {
                x: x0,
                y: y1,
                u: u0,
                v: v_bottom,
            },
        ]);
    }

    vertices
}

struct TexturedQuadPipeline {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
    texture: glow::Texture,
}

impl TexturedQuadPipeline {
    fn new(gl: &glow::Context) -> anyhow::Result<Self> {
        let program = create_program(
            gl,
            TEXTURED_QUAD_VERTEX_SHADER,
            TEXTURED_QUAD_FRAGMENT_SHADER,
        )
        .context("textured quad shader program failed")?;

        let vao = unsafe { gl.create_vertex_array() }
            .map_err(|e| anyhow::anyhow!("create textured quad VAO: {e}"))?;
        let vbo = unsafe { gl.create_buffer() }
            .map_err(|e| anyhow::anyhow!("create textured quad VBO: {e}"))?;
        let ebo = unsafe { gl.create_buffer() }
            .map_err(|e| anyhow::anyhow!("create textured quad EBO: {e}"))?;
        let texture = unsafe { gl.create_texture() }
            .map_err(|e| anyhow::anyhow!("create debug checker texture: {e}"))?;

        unsafe {
            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&TEXTURED_VERTICES),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                cast_slice(&TEXTURED_INDICES),
                glow::STATIC_DRAW,
            );

            let stride = (4 * std::mem::size_of::<f32>()) as i32;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(
                1,
                2,
                glow::FLOAT,
                false,
                stride,
                (2 * std::mem::size_of::<f32>()) as i32,
            );

            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            let pixels: [u8; 16] = [
                255, 220, 120, 255, 64, 110, 220, 255, 64, 110, 220, 255, 255, 220, 120, 255,
            ];
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                2,
                2,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(&pixels)),
            );

            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.bind_vertex_array(None);
        }

        Ok(Self {
            program,
            vao,
            vbo,
            ebo,
            texture,
        })
    }

    fn draw(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            if let Some(location) = gl.get_uniform_location(self.program, "u_texture") {
                gl.uniform_1_i32(Some(&location), 0);
            }
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
            gl.bind_texture(glow::TEXTURE_2D, None);
            gl.use_program(None);
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
            gl.delete_texture(self.texture);
        }
    }
}

struct GridPipeline {
    program: glow::Program,
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    vertex_count: i32,
}

impl GridPipeline {
    fn new(gl: &glow::Context) -> anyhow::Result<Self> {
        let program = create_program(gl, GRID_VERTEX_SHADER, GRID_FRAGMENT_SHADER)
            .context("grid shader program failed")?;

        let vertices = build_grid_vertices(20);
        let vao = unsafe { gl.create_vertex_array() }
            .map_err(|e| anyhow::anyhow!("create grid VAO: {e}"))?;
        let vbo =
            unsafe { gl.create_buffer() }.map_err(|e| anyhow::anyhow!("create grid VBO: {e}"))?;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, cast_slice(&vertices), glow::STATIC_DRAW);

            let stride = (2 * std::mem::size_of::<f32>()) as i32;
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            gl.bind_vertex_array(None);
        }

        Ok(Self {
            program,
            vao,
            vbo,
            vertex_count: (vertices.len() / 2) as i32,
        })
    }

    fn draw(&self, gl: &glow::Context, viewport_width: u32, viewport_height: u32) {
        let aspect = if viewport_height == 0 {
            1.0
        } else {
            viewport_width as f32 / viewport_height as f32
        };

        unsafe {
            gl.use_program(Some(self.program));
            if let Some(loc) = gl.get_uniform_location(self.program, "u_aspect") {
                gl.uniform_1_f32(Some(&loc), aspect);
            }
            if let Some(loc) = gl.get_uniform_location(self.program, "u_color") {
                gl.uniform_4_f32(Some(&loc), 0.35, 0.42, 0.55, 0.55);
            }
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::LINES, 0, self.vertex_count);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
    }
}

fn create_program(
    gl: &glow::Context,
    vertex_source: &str,
    fragment_source: &str,
) -> anyhow::Result<glow::Program> {
    unsafe {
        let program = gl
            .create_program()
            .map_err(|e| anyhow::anyhow!("create GL program: {e}"))?;
        let shaders = [
            (glow::VERTEX_SHADER, vertex_source),
            (glow::FRAGMENT_SHADER, fragment_source),
        ];

        let mut compiled = Vec::with_capacity(shaders.len());
        for (kind, source) in shaders {
            let shader = gl
                .create_shader(kind)
                .map_err(|e| anyhow::anyhow!("create GL shader: {e}"))?;
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                let log = gl.get_shader_info_log(shader);
                gl.delete_shader(shader);
                gl.delete_program(program);
                anyhow::bail!("shader compile failed: {log}");
            }
            gl.attach_shader(program, shader);
            compiled.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            for shader in compiled {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }
            gl.delete_program(program);
            anyhow::bail!("program link failed: {log}");
        }

        for shader in compiled {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        Ok(program)
    }
}

fn build_grid_vertices(divisions_per_axis: usize) -> Vec<f32> {
    let mut vertices = Vec::with_capacity((divisions_per_axis + 1) * 8);
    let step = 2.0 / divisions_per_axis as f32;

    for index in 0..=divisions_per_axis {
        let pos = -1.0 + step * index as f32;
        vertices.extend_from_slice(&[pos, -1.0, pos, 1.0]);
        vertices.extend_from_slice(&[-1.0, pos, 1.0, pos]);
    }

    vertices
}

fn cast_slice<T>(data: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data)) }
}

fn non_zero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value.max(1)).expect("value is clamped to be non-zero")
}

const UI_OVERLAY_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec2 a_position;
layout (location = 1) in vec4 a_color;
out vec4 v_color;
void main() {
    v_color = a_color;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

const UI_OVERLAY_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec4 v_color;
out vec4 frag_color;
void main() {
    frag_color = v_color;
}
"#;

const TILE_MAP_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec2 a_position;
layout (location = 1) in vec2 a_uv;
out vec2 v_uv;
uniform vec2 u_viewport_size;
uniform vec2 u_world_size;
void main() {
    float scale = min((u_viewport_size.x * 0.92) / max(u_world_size.x, 1.0),
                      (u_viewport_size.y * 0.92) / max(u_world_size.y, 1.0));
    vec2 screen = a_position * scale;
    vec2 half_view = max(u_viewport_size * 0.5, vec2(1.0));
    vec2 ndc = vec2(screen.x / half_view.x, -screen.y / half_view.y);
    v_uv = a_uv;
    gl_Position = vec4(ndc, 0.0, 1.0);
}
"#;

const TILE_MAP_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_uv;
out vec4 frag_color;
uniform sampler2D u_texture;
void main() {
    vec4 texel = texture(u_texture, v_uv);
    if (texel.a < 0.05) {
        discard;
    }
    frag_color = texel;
}
"#;

const TEXTURED_QUAD_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec2 a_position;
layout (location = 1) in vec2 a_uv;
out vec2 v_uv;
void main() {
    v_uv = a_uv;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
"#;

const TEXTURED_QUAD_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec2 v_uv;
out vec4 frag_color;
uniform sampler2D u_texture;
void main() {
    frag_color = texture(u_texture, v_uv);
}
"#;

const GRID_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec2 a_position;
uniform float u_aspect;
void main() {
    vec2 pos = a_position;
    if (u_aspect > 1.0) {
        pos.x /= u_aspect;
    } else {
        pos.y *= u_aspect;
    }
    gl_Position = vec4(pos, 0.0, 1.0);
}
"#;

const GRID_FRAGMENT_SHADER: &str = r#"#version 330 core
uniform vec4 u_color;
out vec4 frag_color;
void main() {
    frag_color = u_color;
}
"#;

const VOXEL_MESH_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec3 a_position;
layout (location = 1) in vec4 a_color;
out vec4 v_color;
uniform mat4 u_mvp;
void main() {
    v_color = a_color;
    gl_Position = u_mvp * vec4(a_position, 1.0);
}
"#;

const VOXEL_MESH_FRAGMENT_SHADER: &str = r#"#version 330 core
in vec4 v_color;
out vec4 frag_color;
uniform float u_ambient;
void main() {
    float a = clamp(u_ambient, 0.3, 1.0);
    frag_color = vec4(v_color.rgb * a, v_color.a);
}
"#;

const VOXEL_PICK_VERTEX_SHADER: &str = r#"#version 330 core
layout (location = 0) in vec3 a_position;
uniform mat4 u_mvp;
void main() {
    gl_Position = u_mvp * vec4(a_position, 1.0);
}
"#;

const VOXEL_PICK_FRAGMENT_SHADER: &str = r#"#version 330 core
uniform vec4 u_pick_color;
out vec4 frag_color;
void main() {
    frag_color = u_pick_color;
}
"#;
