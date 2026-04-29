use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use editor_undo::UndoStack;
use eframe::egui;
use engine_assets::vox::{VoxAssetInfo, VoxModel, load_vox_file, scan_vox_files};
use engine_render_gl::{TileInstance, TileMapRenderData};
use game_data::defs::{
    LayerLegendEntry, MapLayersDef, TileLayerDef, TilesetDef, VoxelPanelBakedInstanceDef,
    VoxelPanelBakedVoxelDef, VoxelPanelCellDef, VoxelPanelCompositionConnectionDef,
    VoxelPanelCompositionDef, VoxelPanelCompositionInstanceDef, VoxelPanelCompositionMeshExportDef,
    VoxelPanelCompositionSceneDef, VoxelPanelCompositionViewportPrepDef,
    VoxelPanelConnectionGizmoDef, VoxelPanelDef, VoxelPanelKitCompositionDef, VoxelPanelKitDef,
    VoxelPanelMaterialDef, VoxelPanelPaletteDef, VoxelPanelPreview3dDef, VoxelPanelSocketDef,
    VoxelPanelSocketGizmoDef,
};
use game_data::registry::ContentRegistry;
use game_world::{PropPlacement, SpawnPoint, TriggerZone};
use game_worldgen::{GeneratedScene, SceneGenRequest, SemanticTerrainId};
use serde::{Deserialize, Serialize};

use super::{
    EDITOR_COLLISION_CYCLE, EDITOR_ROLE_CYCLE, TileRoleState, build_tile_map_render_data,
    load_tile_role_state, locate_project_root, save_tile_role_state,
    write_editor_live_preview_manifest,
};

const TOOL_NAMES: [&str; 10] = [
    "Select",
    "Pan",
    "Brush",
    "Erase",
    "Fill",
    "Pick",
    "Tiles",
    "Collision",
    "Assets",
    "Playtest",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftTab {
    Project,
    Textures,
    Maps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RightTab {
    Tile,
    Seams,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomTab {
    Console,
    Validation,
    HotReload,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceTab {
    Project,
    World,
    Assets,
    Animation,
    Character,
    Logic,
    Data,
    Playtest,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetSubTab {
    TerrainAtlas,
    AtlasCompare,
    PixelEditor,
    VoxelPanels,
    Voxels,
    VoxelGenerator,
    Props,
    Seasons,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorldSubTab {
    MapPaint,
    Layers,
    Objects,
    TerrainRules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorldObjectFilter {
    All,
    Props,
    Spawns,
    Triggers,
    VoxelObjects,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LogicSubTab {
    Graphs,
    EventBindings,
    Tools,
    Blocks,
    Validation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectSubTab {
    Overview,
    Validation,
    Build,
    Export,
    Diagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimationSubTab {
    Clips,
    Timeline,
    Events,
    Sockets,
    Hitboxes,
    SeasonalVariants,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterSubTab {
    Bases,
    Outfits,
    Tools,
    DirectionSets,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataSubTab {
    Items,
    Crops,
    Npcs,
    Dialogue,
    Quests,
    Shops,
    Schedules,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlaytestSubTab {
    Launch,
    Runtime,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsSubTab {
    Preferences,
    Keybinds,
    Paths,
    WebCompanion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorSelection {
    Tile,
    MapCell,
    Layer,
    Prop,
    Spawn,
    Trigger,
    VoxelObject,
    PixelSelection,
    VoxelPanelSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PixelTool {
    Pencil,
    Eraser,
    Eyedropper,
    Fill,
    Line,
    RectSelect,
    Paste,
    ReplaceColorFill,
}

impl PixelTool {
    const ALL: [PixelTool; 8] = [
        PixelTool::Pencil,
        PixelTool::Eraser,
        PixelTool::Eyedropper,
        PixelTool::Fill,
        PixelTool::Line,
        PixelTool::RectSelect,
        PixelTool::Paste,
        PixelTool::ReplaceColorFill,
    ];

    fn label(self) -> &'static str {
        match self {
            PixelTool::Pencil => "Pencil",
            PixelTool::Eraser => "Eraser",
            PixelTool::Eyedropper => "Eyedropper",
            PixelTool::Fill => "Fill",
            PixelTool::Line => "Line",
            PixelTool::RectSelect => "Rect Select",
            PixelTool::Paste => "Paste",
            PixelTool::ReplaceColorFill => "Replace Color",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BrushShape {
    Square,
    Circle,
    Diamond,
    Dither,
}

impl BrushShape {
    const ALL: [BrushShape; 4] = [
        BrushShape::Square,
        BrushShape::Circle,
        BrushShape::Diamond,
        BrushShape::Dither,
    ];

    fn label(self) -> &'static str {
        match self {
            BrushShape::Square => "Square",
            BrushShape::Circle => "Circle",
            BrushShape::Diamond => "Diamond",
            BrushShape::Dither => "Dither",
        }
    }
}

#[derive(Clone)]
struct PixelSnapshot {
    label: String,
    pixels: Vec<u8>,
}

#[derive(Clone)]
struct PixelClipboard {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

struct PixelEditorState {
    image_path: PathBuf,
    image: image::RgbaImage,
    texture: Option<egui::TextureHandle>,
    dirty: bool,
    undo_stack: Vec<PixelSnapshot>,
    redo_stack: Vec<PixelSnapshot>,
    clipboard: Option<PixelClipboard>,
    tool: PixelTool,
    brush_shape: BrushShape,
    brush_size: u32,
    zoom: f32,
    primary_color: [u8; 4],
    mirror_x: bool,
    mirror_y: bool,
    rotate_paste_quarters: u8,
    flip_paste_x: bool,
    flip_paste_y: bool,
    selection_start: Option<(u32, u32)>,
    selection_end: Option<(u32, u32)>,
    line_start: Option<(u32, u32)>,
    hover_pixel: Option<(u32, u32)>,
    last_canvas_rect: Option<egui::Rect>,
}

impl PixelEditorState {
    fn load_for_active_tileset(
        project_root: &std::path::Path,
        registry: &ContentRegistry,
        active_map_id: &str,
    ) -> Self {
        let path = pixel_editor_texture_path(project_root, registry, active_map_id);
        let image = image::open(&path)
            .map(|image| image.to_rgba8())
            .unwrap_or_else(|_| image::RgbaImage::from_pixel(256, 256, image::Rgba([0, 0, 0, 0])));

        Self {
            image_path: path,
            image,
            texture: None,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            tool: PixelTool::Pencil,
            brush_shape: BrushShape::Square,
            brush_size: 1,
            zoom: 2.0,
            primary_color: [60, 168, 86, 255],
            mirror_x: false,
            mirror_y: false,
            rotate_paste_quarters: 0,
            flip_paste_x: false,
            flip_paste_y: false,
            selection_start: None,
            selection_end: None,
            line_start: None,
            hover_pixel: None,
            last_canvas_rect: None,
        }
    }

    fn width(&self) -> u32 {
        self.image.width()
    }

    fn height(&self) -> u32 {
        self.image.height()
    }

    fn ensure_texture(&mut self, ctx: &egui::Context) {
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [self.width() as usize, self.height() as usize],
            self.image.as_raw(),
        );
        if let Some(texture) = self.texture.as_mut() {
            texture.set(image, egui::TextureOptions::NEAREST);
        } else {
            self.texture = Some(ctx.load_texture(
                "phase51e_pixel_editor_atlas_texture",
                image,
                egui::TextureOptions::NEAREST,
            ));
        }
    }

    fn push_undo(&mut self, label: impl Into<String>) {
        self.undo_stack.push(PixelSnapshot {
            label: label.into(),
            pixels: self.image.as_raw().clone(),
        });
        if self.undo_stack.len() > 80 {
            let drop_count = self.undo_stack.len() - 80;
            self.undo_stack.drain(0..drop_count);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> Option<String> {
        let snapshot = self.undo_stack.pop()?;
        self.redo_stack.push(PixelSnapshot {
            label: "redo".to_string(),
            pixels: self.image.as_raw().clone(),
        });
        if snapshot.pixels.len() == self.image.as_raw().len() {
            self.image.as_mut().copy_from_slice(&snapshot.pixels);
            self.dirty = true;
        }
        Some(snapshot.label)
    }

    fn redo(&mut self) -> Option<String> {
        let snapshot = self.redo_stack.pop()?;
        self.undo_stack.push(PixelSnapshot {
            label: "undo redo".to_string(),
            pixels: self.image.as_raw().clone(),
        });
        if snapshot.pixels.len() == self.image.as_raw().len() {
            self.image.as_mut().copy_from_slice(&snapshot.pixels);
            self.dirty = true;
        }
        Some(snapshot.label)
    }

    fn save_png_with_backup(&mut self) -> anyhow::Result<PathBuf> {
        if self.image_path.exists() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            let backup_path = self.image_path.with_file_name(format!(
                "{}.phase51e.{}.bak.png",
                self.image_path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("atlas"),
                timestamp
            ));
            std::fs::copy(&self.image_path, &backup_path).with_context(|| {
                format!("failed to create atlas backup {}", backup_path.display())
            })?;
        }

        if let Some(parent) = self.image_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        self.image
            .save(&self.image_path)
            .with_context(|| format!("failed to save {}", self.image_path.display()))?;
        self.dirty = false;
        Ok(self.image_path.clone())
    }

    fn copy_region(&mut self, x: u32, y: u32, width: u32, height: u32) -> bool {
        if width == 0 || height == 0 || x >= self.width() || y >= self.height() {
            return false;
        }
        let width = width.min(self.width() - x);
        let height = height.min(self.height() - y);
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        for py in y..y + height {
            for px in x..x + width {
                pixels.extend_from_slice(&self.image.get_pixel(px, py).0);
            }
        }
        self.clipboard = Some(PixelClipboard {
            width,
            height,
            pixels,
        });
        true
    }

    fn copy_selection(&mut self) -> bool {
        let Some((x, y, width, height)) = self.normalized_selection() else {
            return false;
        };
        self.copy_region(x, y, width, height)
    }

    fn copy_tile(&mut self, tile_cell: (u32, u32), tile_size: (u32, u32)) -> bool {
        let x = tile_cell.0.saturating_mul(tile_size.0);
        let y = tile_cell.1.saturating_mul(tile_size.1);
        self.copy_region(x, y, tile_size.0, tile_size.1)
    }

    fn normalized_selection(&self) -> Option<(u32, u32, u32, u32)> {
        let start = self.selection_start?;
        let end = self.selection_end?;
        let min_x = start.0.min(end.0);
        let min_y = start.1.min(end.1);
        let max_x = start.0.max(end.0);
        let max_y = start.1.max(end.1);
        Some((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
    }

    fn pixel_at_screen_pos(&self, rect: egui::Rect, pos: egui::Pos2) -> Option<(u32, u32)> {
        if !rect.contains(pos) || self.width() == 0 || self.height() == 0 {
            return None;
        }
        let x = ((pos.x - rect.left()) / rect.width() * self.width() as f32).floor() as u32;
        let y = ((pos.y - rect.top()) / rect.height() * self.height() as f32).floor() as u32;
        if x < self.width() && y < self.height() {
            Some((x, y))
        } else {
            None
        }
    }

    fn pixel_rect(&self, image_rect: egui::Rect, x: u32, y: u32) -> egui::Rect {
        let px = image_rect.left() + x as f32 / self.width().max(1) as f32 * image_rect.width();
        let py = image_rect.top() + y as f32 / self.height().max(1) as f32 * image_rect.height();
        let pw = image_rect.width() / self.width().max(1) as f32;
        let ph = image_rect.height() / self.height().max(1) as f32;
        egui::Rect::from_min_size(egui::pos2(px, py), egui::vec2(pw, ph))
    }

    fn set_pixel_with_mirror(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let mut points = vec![(x, y)];
        if self.mirror_x && self.width() > 0 {
            points.push((self.width() - 1 - x, y));
        }
        if self.mirror_y && self.height() > 0 {
            points.push((x, self.height() - 1 - y));
        }
        if self.mirror_x && self.mirror_y && self.width() > 0 && self.height() > 0 {
            points.push((self.width() - 1 - x, self.height() - 1 - y));
        }

        for (px, py) in points {
            if px < self.width() && py < self.height() {
                self.image.put_pixel(px, py, image::Rgba(color));
            }
        }
        self.dirty = true;
    }

    fn apply_brush(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let radius = self.brush_size.saturating_sub(1) as i32 / 2;
        for oy in -radius..=radius {
            for ox in -radius..=radius {
                let px = x as i32 + ox;
                let py = y as i32 + oy;
                if px < 0 || py < 0 || px >= self.width() as i32 || py >= self.height() as i32 {
                    continue;
                }
                let include = match self.brush_shape {
                    BrushShape::Square => true,
                    BrushShape::Circle => {
                        let dist = ((ox * ox + oy * oy) as f32).sqrt();
                        dist <= radius as f32 + 0.35
                    }
                    BrushShape::Diamond => ox.abs() + oy.abs() <= radius.max(1),
                    BrushShape::Dither => ((px + py) & 1) == 0,
                };
                if include {
                    self.set_pixel_with_mirror(px as u32, py as u32, color);
                }
            }
        }
    }

    fn flood_fill(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x >= self.width() || y >= self.height() {
            return;
        }
        let target = self.image.get_pixel(x, y).0;
        if target == color {
            return;
        }
        let mut stack = vec![(x, y)];
        let mut visited = vec![false; (self.width() * self.height()) as usize];
        while let Some((px, py)) = stack.pop() {
            let index = (py * self.width() + px) as usize;
            if visited[index] || self.image.get_pixel(px, py).0 != target {
                continue;
            }
            visited[index] = true;
            self.image.put_pixel(px, py, image::Rgba(color));
            if px > 0 {
                stack.push((px - 1, py));
            }
            if py > 0 {
                stack.push((px, py - 1));
            }
            if px + 1 < self.width() {
                stack.push((px + 1, py));
            }
            if py + 1 < self.height() {
                stack.push((px, py + 1));
            }
        }
        self.dirty = true;
    }

    fn replace_color_fill(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x >= self.width() || y >= self.height() {
            return;
        }
        let target = self.image.get_pixel(x, y).0;
        if target == color {
            return;
        }
        for py in 0..self.height() {
            for px in 0..self.width() {
                if self.image.get_pixel(px, py).0 == target {
                    self.image.put_pixel(px, py, image::Rgba(color));
                }
            }
        }
        self.dirty = true;
    }

    fn draw_line_pixels(&mut self, start: (u32, u32), end: (u32, u32), color: [u8; 4]) {
        let (mut x0, mut y0) = (start.0 as i32, start.1 as i32);
        let (x1, y1) = (end.0 as i32, end.1 as i32);
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            if x0 >= 0 && y0 >= 0 && x0 < self.width() as i32 && y0 < self.height() as i32 {
                self.apply_brush(x0 as u32, y0 as u32, color);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn transformed_clipboard(&self) -> Option<PixelClipboard> {
        let source = self.clipboard.as_ref()?;
        let turns = self.rotate_paste_quarters % 4;
        let width = if turns % 2 == 0 {
            source.width
        } else {
            source.height
        };
        let height = if turns % 2 == 0 {
            source.height
        } else {
            source.width
        };
        let mut pixels = vec![0; (width * height * 4) as usize];

        for sy in 0..source.height {
            for sx in 0..source.width {
                let (mut tx, mut ty) = match turns {
                    0 => (sx, sy),
                    1 => (source.height - 1 - sy, sx),
                    2 => (source.width - 1 - sx, source.height - 1 - sy),
                    _ => (sy, source.width - 1 - sx),
                };
                if self.flip_paste_x {
                    tx = width - 1 - tx;
                }
                if self.flip_paste_y {
                    ty = height - 1 - ty;
                }
                let source_index = ((sy * source.width + sx) * 4) as usize;
                let target_index = ((ty * width + tx) * 4) as usize;
                pixels[target_index..target_index + 4]
                    .copy_from_slice(&source.pixels[source_index..source_index + 4]);
            }
        }

        Some(PixelClipboard {
            width,
            height,
            pixels,
        })
    }

    fn paste_at(&mut self, x: u32, y: u32) -> bool {
        let Some(clip) = self.transformed_clipboard() else {
            return false;
        };
        for cy in 0..clip.height {
            for cx in 0..clip.width {
                let tx = x + cx;
                let ty = y + cy;
                if tx >= self.width() || ty >= self.height() {
                    continue;
                }
                let index = ((cy * clip.width + cx) * 4) as usize;
                let color = [
                    clip.pixels[index],
                    clip.pixels[index + 1],
                    clip.pixels[index + 2],
                    clip.pixels[index + 3],
                ];
                self.image.put_pixel(tx, ty, image::Rgba(color));
            }
        }
        self.dirty = true;
        true
    }

    fn handle_canvas_interaction(
        &mut self,
        response: &egui::Response,
        image_rect: egui::Rect,
    ) -> Option<String> {
        let pointer_pos = response.interact_pointer_pos();
        self.hover_pixel = pointer_pos.and_then(|pos| self.pixel_at_screen_pos(image_rect, pos));
        let pixel = self.hover_pixel?;

        if response.drag_started() && self.tool == PixelTool::RectSelect {
            self.selection_start = Some(pixel);
            self.selection_end = Some(pixel);
            return Some(format!("Started selection at {},{}.", pixel.0, pixel.1));
        }
        if response.dragged() && self.tool == PixelTool::RectSelect {
            self.selection_end = Some(pixel);
            return None;
        }

        if response.drag_started() && matches!(self.tool, PixelTool::Pencil | PixelTool::Eraser) {
            self.push_undo(self.tool.label());
        }

        if response.dragged() || response.clicked() {
            match self.tool {
                PixelTool::Pencil => {
                    if response.clicked() {
                        self.push_undo("Pencil");
                    }
                    self.apply_brush(pixel.0, pixel.1, self.primary_color);
                    return None;
                }
                PixelTool::Eraser => {
                    if response.clicked() {
                        self.push_undo("Eraser");
                    }
                    self.apply_brush(pixel.0, pixel.1, [0, 0, 0, 0]);
                    return None;
                }
                PixelTool::Eyedropper => {
                    self.primary_color = self.image.get_pixel(pixel.0, pixel.1).0;
                    return Some(format!(
                        "Picked RGBA {},{},{},{}.",
                        self.primary_color[0],
                        self.primary_color[1],
                        self.primary_color[2],
                        self.primary_color[3]
                    ));
                }
                PixelTool::Fill => {
                    self.push_undo("Fill");
                    self.flood_fill(pixel.0, pixel.1, self.primary_color);
                    return Some(format!("Filled region at {},{}.", pixel.0, pixel.1));
                }
                PixelTool::ReplaceColorFill => {
                    self.push_undo("Replace color fill");
                    self.replace_color_fill(pixel.0, pixel.1, self.primary_color);
                    return Some(format!(
                        "Replaced matching color from {},{}.",
                        pixel.0, pixel.1
                    ));
                }
                PixelTool::Line => {
                    if let Some(start) = self.line_start.take() {
                        self.push_undo("Line");
                        self.draw_line_pixels(start, pixel, self.primary_color);
                        return Some(format!(
                            "Drew line {},{} -> {},{}.",
                            start.0, start.1, pixel.0, pixel.1
                        ));
                    }
                    self.line_start = Some(pixel);
                    return Some(format!("Line start set at {},{}.", pixel.0, pixel.1));
                }
                PixelTool::Paste => {
                    if response.clicked() {
                        self.push_undo("Paste");
                        if self.paste_at(pixel.0, pixel.1) {
                            return Some(format!("Pasted clipboard at {},{}.", pixel.0, pixel.1));
                        }
                    }
                }
                PixelTool::RectSelect => {}
            }
        }
        None
    }

    fn paint_checkerboard(&self, painter: &egui::Painter, rect: egui::Rect) {
        let tile = 12.0;
        let cols = (rect.width() / tile).ceil() as i32;
        let rows = (rect.height() / tile).ceil() as i32;
        for y in 0..rows {
            for x in 0..cols {
                let color = if (x + y) % 2 == 0 {
                    egui::Color32::from_rgb(45, 49, 58)
                } else {
                    egui::Color32::from_rgb(32, 36, 44)
                };
                let r = egui::Rect::from_min_size(
                    egui::pos2(rect.left() + x as f32 * tile, rect.top() + y as f32 * tile),
                    egui::vec2(tile, tile),
                );
                painter.rect_filled(r.intersect(rect), 0.0, color);
            }
        }
    }

    fn paint_overlays(
        &self,
        painter: &egui::Painter,
        image_rect: egui::Rect,
        selected_cell: (u32, u32),
        tile_size: (u32, u32),
    ) {
        let grid_w = image_rect.width() / self.width().max(1) as f32;
        let grid_h = image_rect.height() / self.height().max(1) as f32;
        if grid_w >= 4.0 && grid_h >= 4.0 {
            let stroke =
                egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180));
            for x in 0..=self.width() {
                let px = image_rect.left() + x as f32 * grid_w;
                painter.line_segment(
                    [
                        egui::pos2(px, image_rect.top()),
                        egui::pos2(px, image_rect.bottom()),
                    ],
                    stroke,
                );
            }
            for y in 0..=self.height() {
                let py = image_rect.top() + y as f32 * grid_h;
                painter.line_segment(
                    [
                        egui::pos2(image_rect.left(), py),
                        egui::pos2(image_rect.right(), py),
                    ],
                    stroke,
                );
            }
        }

        let tile_x = selected_cell.0.saturating_mul(tile_size.0);
        let tile_y = selected_cell.1.saturating_mul(tile_size.1);
        if tile_x < self.width() && tile_y < self.height() {
            let tile_rect = egui::Rect::from_min_max(
                self.pixel_rect(image_rect, tile_x, tile_y).min,
                self.pixel_rect(
                    image_rect,
                    (tile_x + tile_size.0).min(self.width()) - 1,
                    (tile_y + tile_size.1).min(self.height()) - 1,
                )
                .max,
            );
            painter.rect_stroke(
                tile_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(245, 216, 92)),
                egui::StrokeKind::Inside,
            );
        }

        if self.mirror_x {
            let x = image_rect.center().x;
            painter.line_segment(
                [
                    egui::pos2(x, image_rect.top()),
                    egui::pos2(x, image_rect.bottom()),
                ],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 70, 70)),
            );
        }
        if self.mirror_y {
            let y = image_rect.center().y;
            painter.line_segment(
                [
                    egui::pos2(image_rect.left(), y),
                    egui::pos2(image_rect.right(), y),
                ],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 70, 70)),
            );
        }

        if let Some((x, y, width, height)) = self.normalized_selection() {
            let selection_rect = egui::Rect::from_min_max(
                self.pixel_rect(image_rect, x, y).min,
                self.pixel_rect(image_rect, x + width - 1, y + height - 1)
                    .max,
            );
            painter.rect_stroke(
                selection_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(120, 185, 255)),
                egui::StrokeKind::Inside,
            );
        }

        if let Some(start) = self.line_start {
            let rect = self.pixel_rect(image_rect, start.0, start.1);
            painter.rect_stroke(
                rect.expand(2.0),
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 255)),
                egui::StrokeKind::Inside,
            );
        }

        if self.tool == PixelTool::Paste {
            if let (Some(clip), Some((x, y))) = (self.transformed_clipboard(), self.hover_pixel) {
                let max_x = (x + clip.width).min(self.width()).saturating_sub(1);
                let max_y = (y + clip.height).min(self.height()).saturating_sub(1);
                if x < self.width() && y < self.height() {
                    let paste_rect = egui::Rect::from_min_max(
                        self.pixel_rect(image_rect, x, y).min,
                        self.pixel_rect(image_rect, max_x, max_y).max,
                    );
                    painter.rect_filled(
                        paste_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(120, 185, 255, 42),
                    );
                    painter.rect_stroke(
                        paste_rect,
                        0.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(120, 185, 255)),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct MapLayerStroke {
    label: String,
    layers_before: MapLayersDef,
    changed: bool,
}

#[derive(Debug, Clone)]
struct EditorMapState {
    map_id: String,
    layers_path: PathBuf,
    layers: MapLayersDef,
    selected_layer_index: usize,
    selected_symbol: char,
    dirty: bool,
    last_painted_cell: Option<(u32, u32)>,
    active_stroke: Option<MapLayerStroke>,
    history: UndoStack<MapLayersDef>,
}

#[derive(Debug, Clone)]
struct WorldgenBakePreviewState {
    scene: GeneratedScene,
    draft_id: String,
    contract_id: String,
    target_map_id: String,
    target_layer_id: String,
    target_object_layer_id: Option<String>,
    report: WorldgenBakeReport,
}

#[derive(Debug, Clone, Default)]
struct WorldgenBakeReport {
    changed_cells: usize,
    unchanged_cells: usize,
    skipped_protected_cells: usize,
    object_cells: usize,
    terrain_counts: BTreeMap<SemanticTerrainId, usize>,
    warnings: Vec<String>,
    backup_path: Option<PathBuf>,
    committed: bool,
}

#[derive(Debug, Clone)]
struct WorldPlacementState {
    map_id: String,
    props_path: PathBuf,
    spawns_path: PathBuf,
    triggers_path: PathBuf,
    voxel_objects_path: PathBuf,
    props: Vec<PropPlacement>,
    spawns: Vec<SpawnPoint>,
    triggers: Vec<TriggerZone>,
    voxel_objects: VoxelObjectPlacementList,
    selected_prop_index: usize,
    selected_spawn_index: usize,
    selected_trigger_index: usize,
    selected_voxel_object_index: usize,
    active_selection: WorldPlacementKind,
    props_dirty: bool,
    spawns_dirty: bool,
    triggers_dirty: bool,
    voxel_objects_dirty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VoxelObjectPlacementList {
    schema_version: u32,
    map_id: String,
    objects: Vec<VoxelObjectPlacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VoxelObjectPlacement {
    id: String,
    display_name: String,
    source_kind: String,
    source_id: String,
    source_path: String,
    x: f32,
    y: f32,
    z: f32,
    yaw_degrees: f32,
    footprint_width: f32,
    footprint_height: f32,
    height: f32,
    anchor_x: f32,
    anchor_y: f32,
    collision_kind: String,
    locked: bool,
    notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorldPlacementKind {
    Prop,
    Spawn,
    Trigger,
    VoxelObject,
}

impl WorldPlacementState {
    fn load(project_root: &std::path::Path, map_id: &str) -> anyhow::Result<Self> {
        let props_path = map_content_path(project_root, map_id, "props.ron");
        let spawns_path = map_content_path(project_root, map_id, "spawns.ron");
        let triggers_path = map_content_path(project_root, map_id, "triggers.ron");
        let voxel_objects_path = map_content_path(project_root, map_id, "voxel_objects.ron");
        let voxel_objects = if voxel_objects_path.exists() {
            game_data::loader::load_ron_file(&voxel_objects_path)
                .with_context(|| format!("failed to load {}", voxel_objects_path.display()))?
        } else {
            VoxelObjectPlacementList {
                schema_version: 1,
                map_id: map_id.to_string(),
                objects: Vec::new(),
            }
        };
        Ok(Self {
            map_id: map_id.to_string(),
            props: game_data::loader::load_prop_list(&props_path)
                .with_context(|| format!("failed to load {}", props_path.display()))?,
            spawns: game_data::loader::load_spawn_list(&spawns_path)
                .with_context(|| format!("failed to load {}", spawns_path.display()))?,
            triggers: game_data::loader::load_trigger_list(&triggers_path)
                .with_context(|| format!("failed to load {}", triggers_path.display()))?,
            props_path,
            spawns_path,
            triggers_path,
            voxel_objects_path,
            voxel_objects,
            selected_prop_index: 0,
            selected_spawn_index: 0,
            selected_trigger_index: 0,
            selected_voxel_object_index: 0,
            active_selection: WorldPlacementKind::Prop,
            props_dirty: false,
            spawns_dirty: false,
            triggers_dirty: false,
            voxel_objects_dirty: false,
        })
    }

    fn add_prop(&mut self) {
        self.props.push(PropPlacement {
            id: self.unique_prop_id("new_prop"),
            kind: "sign".to_string(),
            x: 0,
            y: 0,
        });
        self.selected_prop_index = self.props.len().saturating_sub(1);
        self.active_selection = WorldPlacementKind::Prop;
        self.props_dirty = true;
    }

    fn add_spawn(&mut self) {
        self.spawns.push(SpawnPoint {
            id: self.unique_spawn_id("new_spawn"),
            kind: "npc".to_string(),
            ref_id: None,
            x: 0,
            y: 0,
        });
        self.selected_spawn_index = self.spawns.len().saturating_sub(1);
        self.active_selection = WorldPlacementKind::Spawn;
        self.spawns_dirty = true;
    }

    fn add_trigger(&mut self) {
        self.triggers.push(TriggerZone {
            id: self.unique_trigger_id("new_trigger"),
            kind: "inspection".to_string(),
            target_map: self.map_id.clone(),
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        });
        self.selected_trigger_index = self.triggers.len().saturating_sub(1);
        self.active_selection = WorldPlacementKind::Trigger;
        self.triggers_dirty = true;
    }

    fn add_voxel_object(&mut self) {
        self.voxel_objects.objects.push(VoxelObjectPlacement {
            id: self.unique_voxel_object_id("new_voxel_object"),
            display_name: "New Voxel Object".to_string(),
            source_kind: "manual".to_string(),
            source_id: "unknown_voxel".to_string(),
            source_path: String::new(),
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw_degrees: 0.0,
            footprint_width: 1.0,
            footprint_height: 1.0,
            height: 1.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
            collision_kind: "none".to_string(),
            locked: false,
            notes: String::new(),
        });
        self.selected_voxel_object_index = self.voxel_objects.objects.len().saturating_sub(1);
        self.active_selection = WorldPlacementKind::VoxelObject;
        self.voxel_objects_dirty = true;
    }

    fn duplicate_selected_prop(&mut self) -> bool {
        let Some(mut prop) = self.props.get(self.selected_prop_index).cloned() else {
            return false;
        };
        prop.id = self.unique_prop_id(&prop.id);
        prop.x += 1;
        prop.y += 1;
        self.props.push(prop);
        self.selected_prop_index = self.props.len().saturating_sub(1);
        self.active_selection = WorldPlacementKind::Prop;
        self.props_dirty = true;
        true
    }

    fn duplicate_selected_spawn(&mut self) -> bool {
        let Some(mut spawn) = self.spawns.get(self.selected_spawn_index).cloned() else {
            return false;
        };
        spawn.id = self.unique_spawn_id(&spawn.id);
        spawn.x += 1;
        spawn.y += 1;
        self.spawns.push(spawn);
        self.selected_spawn_index = self.spawns.len().saturating_sub(1);
        self.active_selection = WorldPlacementKind::Spawn;
        self.spawns_dirty = true;
        true
    }

    fn duplicate_selected_trigger(&mut self) -> bool {
        let Some(mut trigger) = self.triggers.get(self.selected_trigger_index).cloned() else {
            return false;
        };
        trigger.id = self.unique_trigger_id(&trigger.id);
        trigger.x += 1;
        trigger.y += 1;
        self.triggers.push(trigger);
        self.selected_trigger_index = self.triggers.len().saturating_sub(1);
        self.active_selection = WorldPlacementKind::Trigger;
        self.triggers_dirty = true;
        true
    }

    fn duplicate_selected_voxel_object(&mut self) -> bool {
        let Some(mut object) = self
            .voxel_objects
            .objects
            .get(self.selected_voxel_object_index)
            .cloned()
        else {
            return false;
        };
        object.id = self.unique_voxel_object_id(&object.id);
        object.x += 1.0;
        object.y += 1.0;
        self.voxel_objects.objects.push(object);
        self.selected_voxel_object_index = self.voxel_objects.objects.len().saturating_sub(1);
        self.active_selection = WorldPlacementKind::VoxelObject;
        self.voxel_objects_dirty = true;
        true
    }

    fn remove_selected_prop(&mut self) -> bool {
        if self.props.is_empty() {
            return false;
        }
        let index = self.selected_prop_index.min(self.props.len() - 1);
        self.props.remove(index);
        self.selected_prop_index = self
            .selected_prop_index
            .min(self.props.len().saturating_sub(1));
        self.active_selection = WorldPlacementKind::Prop;
        self.props_dirty = true;
        true
    }

    fn remove_selected_spawn(&mut self) -> bool {
        if self.spawns.is_empty() {
            return false;
        }
        let index = self.selected_spawn_index.min(self.spawns.len() - 1);
        self.spawns.remove(index);
        self.selected_spawn_index = self
            .selected_spawn_index
            .min(self.spawns.len().saturating_sub(1));
        self.active_selection = WorldPlacementKind::Spawn;
        self.spawns_dirty = true;
        true
    }

    fn remove_selected_trigger(&mut self) -> bool {
        if self.triggers.is_empty() {
            return false;
        }
        let index = self.selected_trigger_index.min(self.triggers.len() - 1);
        self.triggers.remove(index);
        self.selected_trigger_index = self
            .selected_trigger_index
            .min(self.triggers.len().saturating_sub(1));
        self.active_selection = WorldPlacementKind::Trigger;
        self.triggers_dirty = true;
        true
    }

    fn remove_selected_voxel_object(&mut self) -> bool {
        if self.voxel_objects.objects.is_empty() {
            return false;
        }
        let index = self
            .selected_voxel_object_index
            .min(self.voxel_objects.objects.len() - 1);
        self.voxel_objects.objects.remove(index);
        self.selected_voxel_object_index = self
            .selected_voxel_object_index
            .min(self.voxel_objects.objects.len().saturating_sub(1));
        self.active_selection = WorldPlacementKind::VoxelObject;
        self.voxel_objects_dirty = true;
        true
    }

    fn unique_prop_id(&self, base: &str) -> String {
        unique_id(base, self.props.iter().map(|prop| prop.id.as_str()))
    }

    fn unique_spawn_id(&self, base: &str) -> String {
        unique_id(base, self.spawns.iter().map(|spawn| spawn.id.as_str()))
    }

    fn unique_trigger_id(&self, base: &str) -> String {
        unique_id(
            base,
            self.triggers.iter().map(|trigger| trigger.id.as_str()),
        )
    }

    fn unique_voxel_object_id(&self, base: &str) -> String {
        unique_id(
            base,
            self.voxel_objects
                .objects
                .iter()
                .map(|object| object.id.as_str()),
        )
    }
}

impl WorldgenBakeReport {
    fn summary(&self) -> String {
        format!(
            "terrain_changed={} terrain_unchanged={} protected_layer_cells_preserved={} object_markers={} warnings={}{}",
            self.changed_cells,
            self.unchanged_cells,
            self.skipped_protected_cells,
            self.object_cells,
            self.warnings.len(),
            self.backup_path
                .as_ref()
                .map(|path| format!(" backup={}", path.display()))
                .unwrap_or_default()
        )
    }
}
impl EditorMapState {
    fn load(project_root: &std::path::Path, map_id: &str) -> anyhow::Result<Self> {
        let layers_path = map_layers_path(project_root, map_id);
        let layers = game_data::loader::load_map_layers(&layers_path)
            .with_context(|| format!("failed to load editable layers for map '{map_id}'"))?;
        let selected_layer_index = layers
            .layers
            .iter()
            .position(|layer| layer.visible)
            .unwrap_or(0);
        let selected_symbol = layers
            .layers
            .get(selected_layer_index)
            .and_then(|layer| layer.legend.first())
            .and_then(|entry| entry.symbol.chars().next())
            .unwrap_or('.');

        Ok(Self {
            map_id: map_id.to_string(),
            layers_path,
            layers,
            selected_layer_index,
            selected_symbol,
            dirty: false,
            last_painted_cell: None,
            active_stroke: None,
            history: UndoStack::new(80),
        })
    }

    fn selected_layer(&self) -> Option<&TileLayerDef> {
        self.layers.layers.get(self.selected_layer_index)
    }

    fn selected_layer_mut(&mut self) -> Option<&mut TileLayerDef> {
        self.layers.layers.get_mut(self.selected_layer_index)
    }

    fn selected_layer_id(&self) -> String {
        self.selected_layer()
            .map(|layer| layer.id.clone())
            .unwrap_or_else(|| "<none>".to_string())
    }

    fn push_history_value(&mut self, label: impl Into<String>, layers: MapLayersDef) {
        self.history.push_value(label, layers);
    }

    fn begin_stroke(&mut self, label: impl Into<String>) {
        if self.active_stroke.is_none() {
            self.active_stroke = Some(MapLayerStroke {
                label: label.into(),
                layers_before: self.layers.clone(),
                changed: false,
            });
        }
    }

    fn mark_stroke_changed(&mut self) -> bool {
        if let Some(stroke) = self.active_stroke.as_mut() {
            stroke.changed = true;
            true
        } else {
            false
        }
    }

    fn finish_stroke(&mut self) {
        if let Some(stroke) = self.active_stroke.take() {
            if stroke.changed {
                self.push_history_value(stroke.label, stroke.layers_before);
            }
        }
    }

    fn undo_layers(&mut self) -> Option<String> {
        let result = self.history.undo("redo map layer edit", &self.layers)?;
        self.layers = result.value;
        self.selected_layer_index = self
            .selected_layer_index
            .min(self.layers.layers.len().saturating_sub(1));
        self.dirty = true;
        self.last_painted_cell = None;
        self.active_stroke = None;
        Some(result.label)
    }

    fn redo_layers(&mut self) -> Option<String> {
        let result = self.history.redo("undo map layer edit", &self.layers)?;
        self.layers = result.value;
        self.selected_layer_index = self
            .selected_layer_index
            .min(self.layers.layers.len().saturating_sub(1));
        self.dirty = true;
        self.last_painted_cell = None;
        self.active_stroke = None;
        Some(result.label)
    }

    fn unique_layer_id(&self, base: &str) -> String {
        if !self.layers.layers.iter().any(|layer| layer.id == base) {
            return base.to_string();
        }
        for index in 2..1000 {
            let candidate = format!("{base}_{index}");
            if !self.layers.layers.iter().any(|layer| layer.id == candidate) {
                return candidate;
            }
        }
        format!("{base}_{}", self.layers.layers.len() + 1)
    }

    fn add_blank_layer(&mut self, width: usize, height: usize) {
        let before_layers = self.layers.clone();
        let layer_id = self.unique_layer_id("new_layer");
        self.layers.layers.push(TileLayerDef {
            id: layer_id,
            visible: true,
            locked: false,
            opacity: 1.0,
            legend: Vec::new(),
            rows: vec![".".repeat(width.max(1)); height.max(1)],
        });
        self.selected_layer_index = self.layers.layers.len().saturating_sub(1);
        self.selected_symbol = '.';
        self.push_history_value("add blank layer", before_layers);
        self.dirty = true;
    }

    fn duplicate_layer(&mut self, index: usize) -> bool {
        let Some(mut layer) = self.layers.layers.get(index).cloned() else {
            return false;
        };
        let before_layers = self.layers.clone();
        layer.id = self.unique_layer_id(&format!("{}_copy", layer.id));
        let insert_index = (index + 1).min(self.layers.layers.len());
        self.layers.layers.insert(insert_index, layer);
        self.selected_layer_index = insert_index;
        self.push_history_value("duplicate layer", before_layers);
        self.dirty = true;
        true
    }

    fn remove_layer(&mut self, index: usize) -> bool {
        if self.layers.layers.len() <= 1 || index >= self.layers.layers.len() {
            return false;
        }
        let before_layers = self.layers.clone();
        self.layers.layers.remove(index);
        self.selected_layer_index = self
            .selected_layer_index
            .min(self.layers.layers.len().saturating_sub(1));
        self.push_history_value("remove layer", before_layers);
        self.dirty = true;
        true
    }

    fn move_layer(&mut self, index: usize, direction: i32) -> bool {
        if index >= self.layers.layers.len() {
            return false;
        }
        let target = if direction < 0 {
            index.checked_sub(1)
        } else {
            let next = index + 1;
            (next < self.layers.layers.len()).then_some(next)
        };
        let Some(target) = target else {
            return false;
        };
        let before_layers = self.layers.clone();
        self.layers.layers.swap(index, target);
        self.selected_layer_index = target;
        self.push_history_value("reorder layer", before_layers);
        self.dirty = true;
        true
    }

    fn select_layer_by_id(&mut self, layer_id: &str) {
        if let Some(index) = self
            .layers
            .layers
            .iter()
            .position(|layer| layer.id == layer_id)
        {
            self.selected_layer_index = index;
            self.selected_symbol = self
                .selected_layer()
                .and_then(|layer| layer.legend.first())
                .and_then(|entry| entry.symbol.chars().next())
                .unwrap_or('.');
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VoxelPanelTool {
    Paint,
    Erase,
    Pick,
}

impl VoxelPanelTool {
    const ALL: [VoxelPanelTool; 3] = [
        VoxelPanelTool::Paint,
        VoxelPanelTool::Erase,
        VoxelPanelTool::Pick,
    ];

    fn label(self) -> &'static str {
        match self {
            VoxelPanelTool::Paint => "Paint",
            VoxelPanelTool::Erase => "Erase",
            VoxelPanelTool::Pick => "Pick",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VoxelPanelWorkspaceMode {
    PanelEditor,
    Composition,
    Preview3d,
    Diagnostics,
}

impl VoxelPanelWorkspaceMode {
    const ALL: [VoxelPanelWorkspaceMode; 4] = [
        VoxelPanelWorkspaceMode::PanelEditor,
        VoxelPanelWorkspaceMode::Composition,
        VoxelPanelWorkspaceMode::Preview3d,
        VoxelPanelWorkspaceMode::Diagnostics,
    ];

    fn label(self) -> &'static str {
        match self {
            VoxelPanelWorkspaceMode::PanelEditor => "Panel Editor",
            VoxelPanelWorkspaceMode::Composition => "Composition",
            VoxelPanelWorkspaceMode::Preview3d => "3D Preview",
            VoxelPanelWorkspaceMode::Diagnostics => "Diagnostics",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            VoxelPanelWorkspaceMode::PanelEditor => {
                "Focused 2D voxel-pixel slice editing, panel metadata, sockets, and kit controls."
            }
            VoxelPanelWorkspaceMode::Composition => {
                "Focused panel placement, socket snapping records, composition layout, and export prep."
            }
            VoxelPanelWorkspaceMode::Preview3d => {
                "Read-only inspection viewport for the Phase 53i baked preview RON."
            }
            VoxelPanelWorkspaceMode::Diagnostics => {
                "Palette validation, preview diagnostics, material counts, and connection warnings."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VoxelPanelLeftTab {
    Panels,
    Sockets,
    Compositions,
}

impl VoxelPanelLeftTab {
    const ALL: [VoxelPanelLeftTab; 3] = [
        VoxelPanelLeftTab::Panels,
        VoxelPanelLeftTab::Sockets,
        VoxelPanelLeftTab::Compositions,
    ];

    fn label(self) -> &'static str {
        match self {
            VoxelPanelLeftTab::Panels => "Panels",
            VoxelPanelLeftTab::Sockets => "Sockets",
            VoxelPanelLeftTab::Compositions => "Compositions",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VoxelPanelRightTab {
    Slice,
    PaletteValidation,
}

impl VoxelPanelRightTab {
    const ALL: [VoxelPanelRightTab; 2] = [
        VoxelPanelRightTab::Slice,
        VoxelPanelRightTab::PaletteValidation,
    ];

    fn label(self) -> &'static str {
        match self {
            VoxelPanelRightTab::Slice => "Slice",
            VoxelPanelRightTab::PaletteValidation => "Palette / Validation",
        }
    }
}

#[derive(Debug, Clone)]
struct VoxelPanelPreviewCameraState {
    yaw_degrees: f32,
    pitch_degrees: f32,
    zoom: f32,
    pan_x: f32,
    pan_y: f32,
    show_voxels: bool,
    show_bounds: bool,
    show_socket_gizmos: bool,
    show_connection_gizmos: bool,
    show_floor_grid: bool,
    show_axis_gizmo: bool,
    show_hover_labels: bool,
    show_diagnostics: bool,
    show_material_legend: bool,
    show_labels: bool,
}

impl Default for VoxelPanelPreviewCameraState {
    fn default() -> Self {
        Self {
            yaw_degrees: 45.0,
            pitch_degrees: 35.0,
            zoom: 18.0,
            pan_x: 0.0,
            pan_y: 0.0,
            show_voxels: true,
            show_bounds: true,
            show_socket_gizmos: true,
            show_connection_gizmos: true,
            show_floor_grid: true,
            show_axis_gizmo: true,
            show_hover_labels: true,
            show_diagnostics: true,
            show_material_legend: true,
            show_labels: true,
        }
    }
}

#[derive(Debug, Clone)]
struct VoxelPanelDesignerState {
    kit_paths: Vec<PathBuf>,
    selected_kit_index: usize,
    kit_path: PathBuf,
    kit: VoxelPanelKitDef,
    selected_panel_index: usize,
    selected_palette_index: usize,
    selected_material_id: String,
    active_depth: u32,
    tool: VoxelPanelTool,
    workspace_mode: VoxelPanelWorkspaceMode,
    dirty: bool,
    hover_cell: Option<(u32, u32)>,
    selected_socket_index: usize,
    clipboard_cells: Vec<VoxelPanelCellDef>,
    transform_active_depth_only: bool,
    selected_composition_index: usize,
    selected_composition_instance_index: usize,
    selected_composition_connection_index: usize,
    composition_canvas_zoom: f32,
    preview_camera: VoxelPanelPreviewCameraState,
    preview_3d_export: Option<VoxelPanelCompositionMeshExportDef>,
    preview_3d_export_path: Option<PathBuf>,
    preview_export_paths: Vec<PathBuf>,
    selected_preview_export_index: usize,
    last_mesh_export_summary: Option<String>,
    last_mesh_export_path: Option<PathBuf>,
    panel_undo: UndoStack<Vec<VoxelPanelDef>>,
}

impl VoxelPanelDesignerState {
    fn load(project_root: &std::path::Path) -> Self {
        let mut kit_paths = voxel_panel_kit_paths(project_root);
        let fallback_path = project_root
            .join("content")
            .join("editor_voxel_panels")
            .join("panel_kits")
            .join("starter_gui_panel_kit.ron");
        if kit_paths.is_empty() {
            kit_paths.push(fallback_path.clone());
        }

        let selected_kit_index = 0;
        let kit_path = kit_paths
            .get(selected_kit_index)
            .cloned()
            .unwrap_or(fallback_path);
        let kit = game_data::loader::load_voxel_panel_kit(&kit_path)
            .unwrap_or_else(|_| default_voxel_panel_kit());

        let mut state = Self {
            kit_paths,
            selected_kit_index,
            kit_path,
            kit,
            selected_panel_index: 0,
            selected_palette_index: 0,
            selected_material_id: String::new(),
            active_depth: 0,
            tool: VoxelPanelTool::Paint,
            workspace_mode: VoxelPanelWorkspaceMode::PanelEditor,
            dirty: false,
            hover_cell: None,
            selected_socket_index: 0,
            clipboard_cells: Vec::new(),
            transform_active_depth_only: true,
            selected_composition_index: 0,
            selected_composition_instance_index: 0,
            selected_composition_connection_index: 0,
            composition_canvas_zoom: 1.0,
            preview_camera: VoxelPanelPreviewCameraState::default(),
            preview_3d_export: None,
            preview_3d_export_path: None,
            preview_export_paths: Vec::new(),
            selected_preview_export_index: 0,
            last_mesh_export_summary: None,
            last_mesh_export_path: None,
            panel_undo: UndoStack::new(60),
        };
        state.normalize_selection();
        state.refresh_preview_export_history(project_root);
        state
    }

    fn reload_paths(&mut self, project_root: &std::path::Path) {
        self.kit_paths = voxel_panel_kit_paths(project_root);
        if self.kit_paths.is_empty() {
            self.kit_paths.push(self.kit_path.clone());
        }
        self.selected_kit_index = self
            .kit_paths
            .iter()
            .position(|path| *path == self.kit_path)
            .unwrap_or(0);
        self.refresh_preview_export_history(project_root);
    }

    fn load_selected_kit(&mut self, index: usize) -> anyhow::Result<()> {
        let Some(path) = self.kit_paths.get(index).cloned() else {
            anyhow::bail!("voxel panel kit index {index} is out of range");
        };
        let kit = game_data::loader::load_voxel_panel_kit(&path)
            .with_context(|| format!("failed to load voxel panel kit {}", path.display()))?;
        self.selected_kit_index = index;
        self.kit_path = path;
        self.kit = kit;
        self.selected_panel_index = 0;
        self.selected_palette_index = 0;
        self.selected_material_id.clear();
        self.active_depth = 0;
        self.dirty = false;
        self.selected_socket_index = 0;
        self.clipboard_cells.clear();
        self.selected_composition_index = 0;
        self.selected_composition_instance_index = 0;
        self.selected_composition_connection_index = 0;
        self.preview_3d_export = None;
        self.preview_3d_export_path = None;
        self.preview_export_paths.clear();
        self.selected_preview_export_index = 0;
        self.last_mesh_export_summary = None;
        self.last_mesh_export_path = None;
        self.normalize_selection();
        Ok(())
    }

    fn save_with_backup(&mut self) -> anyhow::Result<Option<PathBuf>> {
        if let Some(parent) = self.kit_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let backup_path = if self.kit_path.exists() {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0);
            let backup_path = self.kit_path.with_file_name(format!(
                "{}.phase53i.{}.bak.ron",
                self.kit_path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("voxel_panel_kit"),
                timestamp
            ));
            std::fs::copy(&self.kit_path, &backup_path).with_context(|| {
                format!(
                    "failed to create voxel panel kit backup {}",
                    backup_path.display()
                )
            })?;
            Some(backup_path)
        } else {
            None
        };

        game_data::loader::save_ron_file(&self.kit_path, &self.kit)?;
        self.dirty = false;
        Ok(backup_path)
    }

    fn normalize_selection(&mut self) {
        if self.kit.panels.is_empty() {
            self.kit.panels.push(default_voxel_panel());
        }
        if self.kit.palettes.is_empty() {
            self.kit.palettes.push(default_voxel_palette());
        }
        if self.kit.compositions.is_empty() {
            self.kit
                .compositions
                .push(default_voxel_panel_composition_scene());
        }

        self.selected_panel_index = self
            .selected_panel_index
            .min(self.kit.panels.len().saturating_sub(1));
        self.selected_palette_index = self
            .kit
            .palettes
            .iter()
            .position(|palette| palette.id == self.kit.default_palette_id)
            .unwrap_or(self.selected_palette_index)
            .min(self.kit.palettes.len().saturating_sub(1));

        let max_depth = self
            .selected_panel()
            .map(|panel| panel.depth.saturating_sub(1))
            .unwrap_or(0);
        self.active_depth = self.active_depth.min(max_depth);
        if let Some(panel) = self.selected_panel() {
            self.selected_socket_index = self
                .selected_socket_index
                .min(panel.sockets.len().saturating_sub(1));
        } else {
            self.selected_socket_index = 0;
        }
        self.selected_composition_index = self
            .selected_composition_index
            .min(self.kit.compositions.len().saturating_sub(1));
        if let Some((instance_len, connection_len)) = self
            .selected_composition()
            .map(|composition| (composition.instances.len(), composition.connections.len()))
        {
            self.selected_composition_instance_index = self
                .selected_composition_instance_index
                .min(instance_len.saturating_sub(1));
            self.selected_composition_connection_index = self
                .selected_composition_connection_index
                .min(connection_len.saturating_sub(1));
        } else {
            self.selected_composition_instance_index = 0;
            self.selected_composition_connection_index = 0;
        }

        if self.selected_material_id.is_empty() || self.selected_material().is_none() {
            self.selected_material_id = self
                .selected_palette()
                .and_then(|palette| palette.materials.first())
                .map(|material| material.id.clone())
                .unwrap_or_else(|| "empty".to_string());
        }
    }

    fn selected_panel(&self) -> Option<&VoxelPanelDef> {
        self.kit.panels.get(self.selected_panel_index)
    }

    fn selected_panel_mut(&mut self) -> Option<&mut VoxelPanelDef> {
        self.kit.panels.get_mut(self.selected_panel_index)
    }

    fn panel_by_id(&self, panel_id: &str) -> Option<&VoxelPanelDef> {
        self.kit.panels.iter().find(|panel| panel.id == panel_id)
    }

    fn selected_composition(&self) -> Option<&VoxelPanelCompositionSceneDef> {
        self.kit.compositions.get(self.selected_composition_index)
    }

    fn selected_composition_mut(&mut self) -> Option<&mut VoxelPanelCompositionSceneDef> {
        self.kit
            .compositions
            .get_mut(self.selected_composition_index)
    }

    fn selected_composition_instance(&self) -> Option<&VoxelPanelCompositionInstanceDef> {
        self.selected_composition()?
            .instances
            .get(self.selected_composition_instance_index)
    }

    fn selected_composition_instance_mut(
        &mut self,
    ) -> Option<&mut VoxelPanelCompositionInstanceDef> {
        let index = self.selected_composition_instance_index;
        self.selected_composition_mut()?.instances.get_mut(index)
    }

    fn selected_palette(&self) -> Option<&VoxelPanelPaletteDef> {
        self.kit.palettes.get(self.selected_palette_index)
    }

    fn selected_material(&self) -> Option<&VoxelPanelMaterialDef> {
        let selected_material_id = self.selected_material_id.as_str();
        self.selected_palette().and_then(|palette| {
            palette
                .materials
                .iter()
                .find(|material| material.id == selected_material_id)
        })
    }

    fn material_color(&self, material_id: &str) -> egui::Color32 {
        self.kit
            .palettes
            .iter()
            .flat_map(|palette| palette.materials.iter())
            .find(|material| material.id == material_id)
            .map(|material| {
                egui::Color32::from_rgba_unmultiplied(
                    material.rgba[0],
                    material.rgba[1],
                    material.rgba[2],
                    material.rgba[3],
                )
            })
            .unwrap_or_else(|| egui::Color32::from_rgba_unmultiplied(255, 0, 255, 220))
    }
    fn material_def_by_id(&self, material_id: &str) -> Option<&VoxelPanelMaterialDef> {
        self.kit
            .palettes
            .iter()
            .flat_map(|palette| palette.materials.iter())
            .find(|material| material.id == material_id)
    }

    fn push_panel_undo(&mut self) {
        self.panel_undo.push("panel edit", &self.kit.panels);
    }

    fn undo_panels(&mut self) -> Option<String> {
        let result = self.panel_undo.undo("redo panel edit", &self.kit.panels)?;
        self.kit.panels = result.value;
        self.dirty = true;
        self.normalize_selection();
        Some(result.label)
    }

    fn redo_panels(&mut self) -> Option<String> {
        let result = self.panel_undo.redo("undo panel edit", &self.kit.panels)?;
        self.kit.panels = result.value;
        self.dirty = true;
        self.normalize_selection();
        Some(result.label)
    }

    fn cell_at(&self, x: u32, y: u32, z: u32) -> Option<&VoxelPanelCellDef> {
        self.selected_panel()?
            .cells
            .iter()
            .find(|cell| cell.x == x && cell.y == y && cell.z == z)
    }

    fn set_cell(&mut self, x: u32, y: u32, z: u32, material_id: String) -> bool {
        let Some(panel) = self.selected_panel_mut() else {
            return false;
        };
        if x >= panel.width || y >= panel.height || z >= panel.depth {
            return false;
        }
        if let Some(cell) = panel
            .cells
            .iter_mut()
            .find(|cell| cell.x == x && cell.y == y && cell.z == z)
        {
            if cell.material_id == material_id {
                return false;
            }
            cell.material_id = material_id;
        } else {
            panel.cells.push(VoxelPanelCellDef {
                x,
                y,
                z,
                material_id,
            });
        }
        self.dirty = true;
        true
    }

    fn erase_cell(&mut self, x: u32, y: u32, z: u32) -> bool {
        let Some(panel) = self.selected_panel_mut() else {
            return false;
        };
        let before = panel.cells.len();
        panel
            .cells
            .retain(|cell| !(cell.x == x && cell.y == y && cell.z == z));
        if panel.cells.len() != before {
            self.dirty = true;
            return true;
        }
        false
    }

    fn pick_cell(&mut self, x: u32, y: u32, z: u32) -> bool {
        let Some(material_id) = self.cell_at(x, y, z).map(|cell| cell.material_id.clone()) else {
            return false;
        };
        self.selected_material_id = material_id;
        true
    }

    fn copy_active_depth_cells(&mut self) -> usize {
        let z = self.active_depth;
        self.clipboard_cells = self
            .selected_panel()
            .map(|panel| {
                panel
                    .cells
                    .iter()
                    .filter(|cell| cell.z == z)
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.clipboard_cells.len()
    }

    fn paste_cells_to_active_depth(&mut self) -> usize {
        let z = self.active_depth;
        let clipboard = self.clipboard_cells.clone();
        let Some(panel) = self.selected_panel_mut() else {
            return 0;
        };
        let mut pasted = 0;
        for cell in clipboard {
            if cell.x >= panel.width || cell.y >= panel.height || z >= panel.depth {
                continue;
            }
            panel.cells.retain(|existing| {
                !(existing.x == cell.x && existing.y == cell.y && existing.z == z)
            });
            panel.cells.push(VoxelPanelCellDef {
                x: cell.x,
                y: cell.y,
                z,
                material_id: cell.material_id,
            });
            pasted += 1;
        }
        if pasted > 0 {
            self.dirty = true;
        }
        pasted
    }

    fn clear_active_depth_cells(&mut self) -> usize {
        let z = self.active_depth;
        let Some(panel) = self.selected_panel_mut() else {
            return 0;
        };
        let before = panel.cells.len();
        panel.cells.retain(|cell| cell.z != z);
        let removed = before.saturating_sub(panel.cells.len());
        if removed > 0 {
            self.dirty = true;
        }
        removed
    }

    fn mirror_cells_x(&mut self) -> usize {
        let z = self.active_depth;
        let active_only = self.transform_active_depth_only;
        let Some(panel) = self.selected_panel_mut() else {
            return 0;
        };
        let mut changed = 0;
        for cell in &mut panel.cells {
            if !active_only || cell.z == z {
                cell.x = panel.width.saturating_sub(1).saturating_sub(cell.x);
                changed += 1;
            }
        }
        if !active_only {
            for socket in &mut panel.sockets {
                socket.x = panel.width.saturating_sub(1).saturating_sub(socket.x);
            }
        }
        if changed > 0 {
            self.dirty = true;
        }
        changed
    }

    fn mirror_cells_y(&mut self) -> usize {
        let z = self.active_depth;
        let active_only = self.transform_active_depth_only;
        let Some(panel) = self.selected_panel_mut() else {
            return 0;
        };
        let mut changed = 0;
        for cell in &mut panel.cells {
            if !active_only || cell.z == z {
                cell.y = panel.height.saturating_sub(1).saturating_sub(cell.y);
                changed += 1;
            }
        }
        if !active_only {
            for socket in &mut panel.sockets {
                socket.y = panel.height.saturating_sub(1).saturating_sub(socket.y);
            }
        }
        if changed > 0 {
            self.dirty = true;
        }
        changed
    }

    fn rotate_cells_cw(&mut self) -> Result<usize, String> {
        let z = self.active_depth;
        let active_only = self.transform_active_depth_only;
        let Some(panel) = self.selected_panel_mut() else {
            return Ok(0);
        };
        if active_only && panel.width != panel.height {
            return Err("Layer-only rotation currently requires a square panel; switch transform scope to Full panel to rotate and swap dimensions.".to_string());
        }

        let old_width = panel.width;
        let old_height = panel.height;
        let mut changed = 0;
        for cell in &mut panel.cells {
            if !active_only || cell.z == z {
                let old_x = cell.x;
                let old_y = cell.y;
                cell.x = old_height.saturating_sub(1).saturating_sub(old_y);
                cell.y = old_x;
                changed += 1;
            }
        }
        if !active_only {
            for socket in &mut panel.sockets {
                let old_x = socket.x;
                let old_y = socket.y;
                socket.x = old_height.saturating_sub(1).saturating_sub(old_y);
                socket.y = old_x;
            }
            panel.width = old_height;
            panel.height = old_width;
        }
        if changed > 0 {
            self.dirty = true;
        }
        Ok(changed)
    }

    fn rotate_cells_ccw(&mut self) -> Result<usize, String> {
        let z = self.active_depth;
        let active_only = self.transform_active_depth_only;
        let Some(panel) = self.selected_panel_mut() else {
            return Ok(0);
        };
        if active_only && panel.width != panel.height {
            return Err("Layer-only rotation currently requires a square panel; switch transform scope to Full panel to rotate and swap dimensions.".to_string());
        }

        let old_width = panel.width;
        let old_height = panel.height;
        let mut changed = 0;
        for cell in &mut panel.cells {
            if !active_only || cell.z == z {
                let old_x = cell.x;
                let old_y = cell.y;
                cell.x = old_y;
                cell.y = old_width.saturating_sub(1).saturating_sub(old_x);
                changed += 1;
            }
        }
        if !active_only {
            for socket in &mut panel.sockets {
                let old_x = socket.x;
                let old_y = socket.y;
                socket.x = old_y;
                socket.y = old_width.saturating_sub(1).saturating_sub(old_x);
            }
            panel.width = old_height;
            panel.height = old_width;
        }
        if changed > 0 {
            self.dirty = true;
        }
        Ok(changed)
    }

    fn add_selected_panel_instance_to_composition(&mut self) -> Option<String> {
        let panel_id = self.selected_panel()?.id.clone();
        let (instance_id, selected_index) = {
            let composition = self.selected_composition_mut()?;
            let index = composition.instances.len() + 1;
            let instance_id = format!("{}_inst_{index:02}", panel_id);
            composition
                .instances
                .push(VoxelPanelCompositionInstanceDef {
                    id: instance_id.clone(),
                    panel_id: panel_id.clone(),
                    x: (index as i32 - 1) * 3,
                    y: (index as i32 - 1) * 2,
                    z: 0,
                    rotation_degrees: 0,
                    mirror_x: false,
                    mirror_y: false,
                    locked: false,
                });
            (instance_id, composition.instances.len().saturating_sub(1))
        };
        self.selected_composition_instance_index = selected_index;
        self.dirty = true;
        Some(instance_id)
    }

    fn remove_selected_composition_instance(&mut self) -> Option<String> {
        let index = self.selected_composition_instance_index;
        let selected_connection_index = self.selected_composition_connection_index;
        let (removed_id, next_instance_index, next_connection_index) = {
            let composition = self.selected_composition_mut()?;
            if composition.instances.is_empty() {
                return None;
            }
            let remove_index = index.min(composition.instances.len().saturating_sub(1));
            let removed = composition.instances.remove(remove_index);
            composition.connections.retain(|connection| {
                connection.from_instance != removed.id && connection.to_instance != removed.id
            });
            (
                removed.id,
                remove_index.min(composition.instances.len().saturating_sub(1)),
                selected_connection_index.min(composition.connections.len().saturating_sub(1)),
            )
        };
        self.selected_composition_instance_index = next_instance_index;
        self.selected_composition_connection_index = next_connection_index;
        self.dirty = true;
        Some(removed_id)
    }

    fn move_selected_composition_instance_to(&mut self, x: i32, y: i32) -> bool {
        let Some(composition) = self.selected_composition() else {
            return false;
        };
        let canvas_width = composition.canvas_width as i32;
        let canvas_height = composition.canvas_height as i32;
        let Some(instance) = self.selected_composition_instance_mut() else {
            return false;
        };
        if instance.locked {
            return false;
        }
        let next_x = x.clamp(0, canvas_width.saturating_sub(1));
        let next_y = y.clamp(0, canvas_height.saturating_sub(1));
        if instance.x == next_x && instance.y == next_y {
            return false;
        }
        instance.x = next_x;
        instance.y = next_y;
        self.dirty = true;
        true
    }

    fn snap_selected_instance_to_nearest_socket(&mut self) -> Result<String, String> {
        let selected_index = self.selected_composition_instance_index;
        let Some(composition) = self.selected_composition().cloned() else {
            return Err("No composition scene selected.".to_string());
        };
        let Some(selected_instance) = composition.instances.get(selected_index).cloned() else {
            return Err("No composition instance selected.".to_string());
        };
        let Some(selected_panel) = self.panel_by_id(&selected_instance.panel_id).cloned() else {
            return Err(format!(
                "Selected instance '{}' references missing panel '{}'.",
                selected_instance.id, selected_instance.panel_id
            ));
        };
        if selected_panel.sockets.is_empty() {
            return Err(format!(
                "Panel '{}' has no sockets to snap.",
                selected_panel.id
            ));
        }

        let mut best: Option<(i32, i32, i32, String, String, String, String, [i32; 3], i32)> = None;
        for source_socket in &selected_panel.sockets {
            let source_world = voxel_panel_socket_world_position(
                &selected_panel,
                &selected_instance,
                source_socket,
            );
            let source_local = voxel_panel_socket_local_position(
                &selected_panel,
                &selected_instance,
                source_socket,
            );
            for (other_index, other_instance) in composition.instances.iter().enumerate() {
                if other_index == selected_index {
                    continue;
                }
                let Some(other_panel) = self.panel_by_id(&other_instance.panel_id) else {
                    continue;
                };
                for target_socket in &other_panel.sockets {
                    if !voxel_panel_sockets_compatible(
                        &selected_panel,
                        source_socket,
                        other_panel,
                        target_socket,
                    ) {
                        continue;
                    }
                    let target_world = voxel_panel_socket_world_position(
                        other_panel,
                        other_instance,
                        target_socket,
                    );
                    let distance = (source_world.0 - target_world.0).abs()
                        + (source_world.1 - target_world.1).abs()
                        + (source_world.2 - target_world.2).abs();
                    let new_x = target_world.0 - source_local.0;
                    let new_y = target_world.1 - source_local.1;
                    let new_z = target_world.2 - source_local.2;
                    let connection_id = format!(
                        "{}_{}_to_{}_{}",
                        selected_instance.id, source_socket.id, other_instance.id, target_socket.id
                    );
                    let offset = [
                        source_world.0 - target_world.0,
                        source_world.1 - target_world.1,
                        source_world.2 - target_world.2,
                    ];
                    let candidate = (
                        new_x,
                        new_y,
                        new_z,
                        connection_id,
                        source_socket.id.clone(),
                        other_instance.id.clone(),
                        target_socket.id.clone(),
                        offset,
                        distance,
                    );
                    if best
                        .as_ref()
                        .map(|current| distance < current.8)
                        .unwrap_or(true)
                    {
                        best = Some(candidate);
                    }
                }
            }
        }

        let Some((
            new_x,
            new_y,
            new_z,
            connection_id,
            from_socket,
            target_instance,
            to_socket,
            _offset,
            _distance,
        )) = best
        else {
            return Err("No compatible socket pair found for the selected instance.".to_string());
        };

        let selected_connection_index = {
            let composition = self
                .selected_composition_mut()
                .ok_or_else(|| "No composition scene selected.".to_string())?;
            let canvas_width = composition.canvas_width as i32;
            let canvas_height = composition.canvas_height as i32;
            let canvas_depth = composition.canvas_depth as i32;
            if let Some(instance) = composition.instances.get_mut(selected_index) {
                if instance.locked {
                    return Err(format!("Instance '{}' is locked.", instance.id));
                }
                instance.x = new_x.clamp(0, canvas_width);
                instance.y = new_y.clamp(0, canvas_height);
                instance.z = new_z.clamp(0, canvas_depth);
            }

            let snapped_offset = [0, 0, 0];
            composition
                .connections
                .retain(|connection| connection.id != connection_id);
            composition
                .connections
                .push(VoxelPanelCompositionConnectionDef {
                    id: connection_id.clone(),
                    from_instance: selected_instance.id.clone(),
                    from_socket,
                    to_instance: target_instance,
                    to_socket,
                    snapped: true,
                    offset: snapped_offset,
                });
            composition.connections.len().saturating_sub(1)
        };
        self.selected_composition_connection_index = selected_connection_index;
        self.dirty = true;
        Ok(format!(
            "Snapped '{}' with connection '{}'.",
            selected_instance.id, connection_id
        ))
    }

    fn bake_selected_composition_mesh_export(
        &self,
    ) -> Result<VoxelPanelCompositionMeshExportDef, String> {
        let composition = self
            .selected_composition()
            .ok_or_else(|| "No composition scene selected.".to_string())?;

        let generated_at_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        let mut baked_instances = Vec::new();
        let mut baked_voxels = Vec::new();
        let mut socket_gizmos = Vec::new();
        let mut connection_gizmos = Vec::new();
        let mut bounds_min = [i32::MAX, i32::MAX, i32::MAX];
        let mut bounds_max = [i32::MIN, i32::MIN, i32::MIN];
        let mut has_bounds = false;

        for instance in &composition.instances {
            let panel = self.panel_by_id(&instance.panel_id).ok_or_else(|| {
                format!(
                    "Instance '{}' references missing panel '{}'.",
                    instance.id, instance.panel_id
                )
            })?;

            let mut instance_min = [i32::MAX, i32::MAX, i32::MAX];
            let mut instance_max = [i32::MIN, i32::MIN, i32::MIN];
            let mut instance_has_bounds = false;
            let mut instance_voxel_count = 0usize;

            for cell in &panel.cells {
                let local = voxel_panel_cell_local_position(panel, instance, cell);
                let raw_world = [
                    instance.x + local.0,
                    instance.y + local.1,
                    instance.z + local.2,
                ];
                let world =
                    voxel_panel_map_source_axis(raw_world, &composition.viewport_prep.source_axis);
                let local_array = [local.0, local.1, local.2];
                let (rgba, render_hint) = self
                    .material_def_by_id(&cell.material_id)
                    .map(|material| (material.rgba, material.render_hint.clone()))
                    .unwrap_or(([255, 0, 255, 220], "missing_material".to_string()));

                voxel_panel_expand_bounds(&mut bounds_min, &mut bounds_max, &mut has_bounds, world);
                voxel_panel_expand_bounds(
                    &mut instance_min,
                    &mut instance_max,
                    &mut instance_has_bounds,
                    world,
                );
                instance_voxel_count += 1;

                baked_voxels.push(VoxelPanelBakedVoxelDef {
                    instance_id: instance.id.clone(),
                    panel_id: panel.id.clone(),
                    material_id: cell.material_id.clone(),
                    local: local_array,
                    world,
                    rgba,
                    render_hint,
                });
            }

            if !instance_has_bounds && composition.viewport_prep.include_empty_bounds {
                let (width, height) = voxel_panel_instance_size(panel, instance);
                let empty_min = voxel_panel_map_source_axis(
                    [instance.x, instance.y, instance.z],
                    &composition.viewport_prep.source_axis,
                );
                let empty_max = voxel_panel_map_source_axis(
                    [
                        instance.x + width.max(1) as i32 - 1,
                        instance.y + height.max(1) as i32 - 1,
                        instance.z + panel.depth.max(1) as i32 - 1,
                    ],
                    &composition.viewport_prep.source_axis,
                );
                voxel_panel_expand_bounds(
                    &mut bounds_min,
                    &mut bounds_max,
                    &mut has_bounds,
                    empty_min,
                );
                voxel_panel_expand_bounds(
                    &mut bounds_min,
                    &mut bounds_max,
                    &mut has_bounds,
                    empty_max,
                );
                voxel_panel_expand_bounds(
                    &mut instance_min,
                    &mut instance_max,
                    &mut instance_has_bounds,
                    empty_min,
                );
                voxel_panel_expand_bounds(
                    &mut instance_min,
                    &mut instance_max,
                    &mut instance_has_bounds,
                    empty_max,
                );
            }

            if !instance_has_bounds {
                instance_min = [instance.x, instance.y, instance.z];
                instance_max = [instance.x, instance.y, instance.z];
            }

            if composition.viewport_prep.emit_socket_gizmos {
                for socket in &panel.sockets {
                    let world = voxel_panel_socket_world_position(panel, instance, socket);
                    let world = voxel_panel_map_source_axis(
                        [world.0, world.1, world.2],
                        &composition.viewport_prep.source_axis,
                    );
                    socket_gizmos.push(VoxelPanelSocketGizmoDef {
                        instance_id: instance.id.clone(),
                        panel_id: panel.id.clone(),
                        socket_id: socket.id.clone(),
                        edge: socket.edge.clone(),
                        world,
                        accepts: socket.accepts.clone(),
                        required: socket.required,
                    });
                }
            }

            baked_instances.push(VoxelPanelBakedInstanceDef {
                instance_id: instance.id.clone(),
                panel_id: panel.id.clone(),
                transform_origin: voxel_panel_map_source_axis(
                    [instance.x, instance.y, instance.z],
                    &composition.viewport_prep.source_axis,
                ),
                rotation_degrees: voxel_panel_normalized_rotation(instance.rotation_degrees),
                mirror_x: instance.mirror_x,
                mirror_y: instance.mirror_y,
                bounds_min: instance_min,
                bounds_max: instance_max,
                voxel_count: instance_voxel_count,
            });
        }

        for connection in &composition.connections {
            let Some(from_instance) = composition
                .instances
                .iter()
                .find(|instance| instance.id == connection.from_instance)
            else {
                continue;
            };
            let Some(to_instance) = composition
                .instances
                .iter()
                .find(|instance| instance.id == connection.to_instance)
            else {
                continue;
            };
            let Some(from_panel) = self.panel_by_id(&from_instance.panel_id) else {
                continue;
            };
            let Some(to_panel) = self.panel_by_id(&to_instance.panel_id) else {
                continue;
            };
            let Some(from_socket) = from_panel
                .sockets
                .iter()
                .find(|socket| socket.id == connection.from_socket)
            else {
                continue;
            };
            let Some(to_socket) = to_panel
                .sockets
                .iter()
                .find(|socket| socket.id == connection.to_socket)
            else {
                continue;
            };
            let from_world =
                voxel_panel_socket_world_position(from_panel, from_instance, from_socket);
            let to_world = voxel_panel_socket_world_position(to_panel, to_instance, to_socket);
            let from_world = voxel_panel_map_source_axis(
                [from_world.0, from_world.1, from_world.2],
                &composition.viewport_prep.source_axis,
            );
            let to_world = voxel_panel_map_source_axis(
                [to_world.0, to_world.1, to_world.2],
                &composition.viewport_prep.source_axis,
            );
            connection_gizmos.push(VoxelPanelConnectionGizmoDef {
                connection_id: connection.id.clone(),
                from_instance: connection.from_instance.clone(),
                from_socket: connection.from_socket.clone(),
                to_instance: connection.to_instance.clone(),
                to_socket: connection.to_socket.clone(),
                from_world,
                to_world,
                snapped: connection.snapped,
                offset: voxel_panel_map_source_axis(
                    connection.offset,
                    &composition.viewport_prep.source_axis,
                ),
            });
        }

        if !has_bounds {
            bounds_min = [0, 0, 0];
            bounds_max = [0, 0, 0];
        }
        let anchor_offset = voxel_panel_anchor_offset(
            bounds_min,
            bounds_max,
            &composition.viewport_prep.bake_anchor,
        );
        if anchor_offset != [0, 0, 0] {
            bounds_min = voxel_panel_translate_point(bounds_min, anchor_offset);
            bounds_max = voxel_panel_translate_point(bounds_max, anchor_offset);
            for instance in &mut baked_instances {
                instance.transform_origin =
                    voxel_panel_translate_point(instance.transform_origin, anchor_offset);
                instance.bounds_min =
                    voxel_panel_translate_point(instance.bounds_min, anchor_offset);
                instance.bounds_max =
                    voxel_panel_translate_point(instance.bounds_max, anchor_offset);
            }
            for voxel in &mut baked_voxels {
                voxel.world = voxel_panel_translate_point(voxel.world, anchor_offset);
            }
            for gizmo in &mut socket_gizmos {
                gizmo.world = voxel_panel_translate_point(gizmo.world, anchor_offset);
            }
            for gizmo in &mut connection_gizmos {
                gizmo.from_world = voxel_panel_translate_point(gizmo.from_world, anchor_offset);
                gizmo.to_world = voxel_panel_translate_point(gizmo.to_world, anchor_offset);
            }
        }

        Ok(VoxelPanelCompositionMeshExportDef {
            id: format!("{}_{}_phase53i_preview", self.kit.id, composition.id),
            display_name: format!("{} / {} Phase 53i Preview", self.kit.display_name, composition.display_name),
            source_kit_id: self.kit.id.clone(),
            source_composition_id: composition.id.clone(),
            generated_by_phase: "phase53i_voxel_composition_3d_preview_prep".to_string(),
            generated_at_unix,
            voxel_unit: self.kit.preview_3d.voxel_unit,
            layer_gap: composition.viewport_prep.bake_layer_gap.max(self.kit.preview_3d.layer_gap),
            source_axis: composition.viewport_prep.source_axis.clone(),
            bake_anchor: composition.viewport_prep.bake_anchor.clone(),
            bounds_min,
            bounds_max,
            voxel_count: baked_voxels.len(),
            instance_count: baked_instances.len(),
            socket_gizmo_count: socket_gizmos.len(),
            connection_gizmo_count: connection_gizmos.len(),
            instances: baked_instances,
            voxels: baked_voxels,
            socket_gizmos,
            connection_gizmos,
            notes: vec![
                "Non-rendering Phase 53i export. The future 3D viewport should consume this as preview mesh/gizmo input.".to_string(),
                "Bounds are inclusive integer voxel coordinates in composition space.".to_string(),
                "Voxel coordinates already include panel instance rotation and mirror transforms.".to_string(),
                "source_axis remaps source panel coordinates before bake_anchor is applied.".to_string(),
            ],
        })
    }

    fn mesh_export_path_for_selected_composition(
        &self,
        project_root: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let composition = self
            .selected_composition()
            .ok_or_else(|| "No composition scene selected.".to_string())?;

        let export_root = if composition.viewport_prep.mesh_export_path.trim().is_empty() {
            PathBuf::from("content/editor_voxel_panels/preview_exports")
        } else {
            PathBuf::from(composition.viewport_prep.mesh_export_path.trim())
        };

        let export_dir = if export_root.is_absolute() {
            export_root
        } else {
            project_root.join(export_root)
        };

        let file_name = format!(
            "{}__{}_phase53i_preview.ron",
            voxel_panel_sanitize_file_part(&self.kit.id),
            voxel_panel_sanitize_file_part(&composition.id)
        );
        Ok(export_dir.join(file_name))
    }

    fn mesh_export_dir_for_selected_composition(
        &self,
        project_root: &std::path::Path,
    ) -> Result<PathBuf, String> {
        let composition = self
            .selected_composition()
            .ok_or_else(|| "No composition scene selected.".to_string())?;
        let export_root = if composition.viewport_prep.mesh_export_path.trim().is_empty() {
            PathBuf::from("content/editor_voxel_panels/preview_exports")
        } else {
            PathBuf::from(composition.viewport_prep.mesh_export_path.trim())
        };
        Ok(if export_root.is_absolute() {
            export_root
        } else {
            project_root.join(export_root)
        })
    }

    fn refresh_preview_export_history(&mut self, project_root: &std::path::Path) {
        let paths = self
            .mesh_export_dir_for_selected_composition(project_root)
            .ok()
            .and_then(|dir| game_data::loader::ron_files_in(&dir).ok())
            .unwrap_or_default();
        self.preview_export_paths = paths;
        if let Some(current_path) = &self.preview_3d_export_path {
            self.selected_preview_export_index = self
                .preview_export_paths
                .iter()
                .position(|path| path == current_path)
                .unwrap_or(self.selected_preview_export_index);
        }
        self.selected_preview_export_index = self
            .selected_preview_export_index
            .min(self.preview_export_paths.len().saturating_sub(1));
    }

    fn load_mesh_preview_from_path(&mut self, export_path: PathBuf) -> anyhow::Result<PathBuf> {
        let export: VoxelPanelCompositionMeshExportDef =
            game_data::loader::load_ron_file(&export_path).with_context(|| {
                format!(
                    "failed to load voxel panel 3D preview {}",
                    export_path.display()
                )
            })?;
        self.last_mesh_export_summary = Some(format!(
            "Loaded {} voxel(s), {} instance(s), {} socket gizmo(s), and {} connection gizmo(s).",
            export.voxel_count,
            export.instance_count,
            export.socket_gizmo_count,
            export.connection_gizmo_count
        ));
        self.preview_3d_export = Some(export);
        self.preview_3d_export_path = Some(export_path.clone());
        self.last_mesh_export_path = Some(export_path.clone());
        Ok(export_path)
    }

    fn export_selected_composition_mesh_preview(
        &mut self,
        project_root: &std::path::Path,
    ) -> anyhow::Result<PathBuf> {
        let export = self
            .bake_selected_composition_mesh_export()
            .map_err(|message| anyhow::anyhow!(message))?;
        let export_path = self
            .mesh_export_path_for_selected_composition(project_root)
            .map_err(|message| anyhow::anyhow!(message))?;

        game_data::loader::save_ron_file(&export_path, &export)?;
        self.last_mesh_export_summary = Some(format!(
            "Exported {} voxel(s), {} instance(s), {} socket gizmo(s), and {} connection gizmo(s).",
            export.voxel_count,
            export.instance_count,
            export.socket_gizmo_count,
            export.connection_gizmo_count
        ));
        self.preview_3d_export = Some(export);
        self.preview_3d_export_path = Some(export_path.clone());
        self.last_mesh_export_path = Some(export_path.clone());
        self.refresh_preview_export_history(project_root);
        Ok(export_path)
    }

    fn load_selected_composition_mesh_preview(
        &mut self,
        project_root: &std::path::Path,
    ) -> anyhow::Result<PathBuf> {
        let export_path = self
            .mesh_export_path_for_selected_composition(project_root)
            .map_err(|message| anyhow::anyhow!(message))?;
        let loaded_path = self.load_mesh_preview_from_path(export_path)?;
        self.refresh_preview_export_history(project_root);
        Ok(loaded_path)
    }

    fn preview_material_counts(
        &self,
        export: &VoxelPanelCompositionMeshExportDef,
    ) -> Vec<(String, [u8; 4], usize)> {
        let mut counts = BTreeMap::<String, ([u8; 4], usize)>::new();
        for voxel in &export.voxels {
            let entry = counts
                .entry(voxel.material_id.clone())
                .or_insert((voxel.rgba, 0));
            entry.0 = voxel.rgba;
            entry.1 += 1;
        }
        counts
            .into_iter()
            .map(|(material_id, (rgba, count))| (material_id, rgba, count))
            .collect()
    }

    fn preview_diagnostics_messages(
        &self,
        export: &VoxelPanelCompositionMeshExportDef,
    ) -> Vec<String> {
        let mut messages = Vec::new();
        if export.voxel_count != export.voxels.len() {
            messages.push(format!(
                "Export header voxel_count {} does not match {} baked voxel records.",
                export.voxel_count,
                export.voxels.len()
            ));
        }
        if export.instance_count != export.instances.len() {
            messages.push(format!(
                "Export header instance_count {} does not match {} baked instance records.",
                export.instance_count,
                export.instances.len()
            ));
        }
        if export.socket_gizmo_count != export.socket_gizmos.len() {
            messages.push(format!(
                "Export header socket_gizmo_count {} does not match {} socket records.",
                export.socket_gizmo_count,
                export.socket_gizmos.len()
            ));
        }
        if export.connection_gizmo_count != export.connection_gizmos.len() {
            messages.push(format!(
                "Export header connection_gizmo_count {} does not match {} connection records.",
                export.connection_gizmo_count,
                export.connection_gizmos.len()
            ));
        }

        let material_ids = self
            .kit
            .palettes
            .iter()
            .flat_map(|palette| palette.materials.iter())
            .map(|material| material.id.as_str())
            .collect::<BTreeSet<_>>();
        for (material_id, _, count) in self.preview_material_counts(export) {
            if !material_ids.contains(material_id.as_str()) {
                messages.push(format!(
                    "Preview contains {} voxel(s) using missing material '{}'.",
                    count, material_id
                ));
            }
        }

        let socket_lookup = export
            .socket_gizmos
            .iter()
            .map(|socket| {
                (
                    (socket.instance_id.as_str(), socket.socket_id.as_str()),
                    socket,
                )
            })
            .collect::<BTreeMap<_, _>>();
        for socket in &export.socket_gizmos {
            if socket.required {
                let connected = export.connection_gizmos.iter().any(|connection| {
                    (connection.from_instance == socket.instance_id
                        && connection.from_socket == socket.socket_id)
                        || (connection.to_instance == socket.instance_id
                            && connection.to_socket == socket.socket_id)
                });
                if !connected {
                    messages.push(format!(
                        "Required socket '{}.{}' is disconnected in the preview export.",
                        socket.instance_id, socket.socket_id
                    ));
                }
            }
        }

        for connection in &export.connection_gizmos {
            let from_socket = socket_lookup
                .get(&(
                    connection.from_instance.as_str(),
                    connection.from_socket.as_str(),
                ))
                .copied();
            let to_socket = socket_lookup
                .get(&(
                    connection.to_instance.as_str(),
                    connection.to_socket.as_str(),
                ))
                .copied();
            let (Some(from_socket), Some(to_socket)) = (from_socket, to_socket) else {
                messages.push(format!(
                    "Connection '{}' points at one or more missing socket gizmos.",
                    connection.connection_id
                ));
                continue;
            };
            if !voxel_panel_socket_gizmo_accepts(from_socket, to_socket)
                && !voxel_panel_socket_gizmo_accepts(to_socket, from_socket)
                && !(from_socket.accepts == to_socket.accepts && !from_socket.accepts.is_empty())
            {
                messages.push(format!(
                    "Connection '{}' links sockets that do not appear compatible from preview metadata.",
                    connection.connection_id
                ));
            }
            if connection.snapped && connection.from_world != connection.to_world {
                messages.push(format!(
                    "Connection '{}' is marked snapped but preview endpoints differ: {:?} vs {:?}.",
                    connection.connection_id, connection.from_world, connection.to_world
                ));
            }
            if !connection.snapped {
                messages.push(format!(
                    "Connection '{}' is not snapped; inspect offset {:?}.",
                    connection.connection_id, connection.offset
                ));
            }
        }

        if messages.is_empty() {
            messages.push("Preview diagnostics passed: no missing materials, invalid connections, or disconnected required sockets found.".to_string());
        }
        messages
    }

    fn validation_messages(&self) -> Vec<String> {
        let mut messages = Vec::new();

        if self.kit.id.trim().is_empty() {
            messages.push("Kit id is empty.".to_string());
        }
        if self.kit.palettes.is_empty() {
            messages.push("Kit has no palettes.".to_string());
        }
        if self.kit.panels.is_empty() {
            messages.push("Kit has no panels.".to_string());
        }

        let palette_ids = self
            .kit
            .palettes
            .iter()
            .map(|palette| palette.id.as_str())
            .collect::<BTreeSet<_>>();
        if !palette_ids.contains(self.kit.default_palette_id.as_str()) {
            messages.push(format!(
                "Default palette '{}' does not exist.",
                self.kit.default_palette_id
            ));
        }
        if self.kit.composition.snap_unit_px == 0 {
            messages.push("Kit composition snap_unit_px must be at least 1.".to_string());
        }
        if self.kit.preview_3d.voxel_unit <= 0.0 {
            messages.push("Kit 3D preview voxel_unit must be greater than 0.".to_string());
        }

        let material_ids = self
            .kit
            .palettes
            .iter()
            .flat_map(|palette| palette.materials.iter())
            .map(|material| material.id.as_str())
            .collect::<BTreeSet<_>>();

        for palette in &self.kit.palettes {
            if palette.id.trim().is_empty() {
                messages.push("A palette has an empty id.".to_string());
            }
            let mut local_material_ids = BTreeSet::new();
            for material in &palette.materials {
                if material.id.trim().is_empty() {
                    messages.push(format!(
                        "Palette '{}' has a material with an empty id.",
                        palette.id
                    ));
                }
                if !local_material_ids.insert(material.id.as_str()) {
                    messages.push(format!(
                        "Palette '{}' has duplicate material id '{}'.",
                        palette.id, material.id
                    ));
                }
            }
        }

        let mut panel_ids = BTreeSet::new();
        for panel in &self.kit.panels {
            if panel.id.trim().is_empty() {
                messages.push("A panel has an empty id.".to_string());
            }
            if !panel_ids.insert(panel.id.as_str()) {
                messages.push(format!("Duplicate panel id '{}'.", panel.id));
            }
            if panel.width == 0 || panel.height == 0 || panel.depth == 0 {
                messages.push(format!("Panel '{}' has a zero dimension.", panel.id));
            }
            if panel.width > 128 || panel.height > 128 || panel.depth > 32 {
                messages.push(format!(
                    "Panel '{}' exceeds Phase 53g safe editor limits 128x128x32.",
                    panel.id
                ));
            }

            if !self.kit.composition.allowed_panel_kinds.is_empty()
                && !self
                    .kit
                    .composition
                    .allowed_panel_kinds
                    .iter()
                    .any(|kind| kind == &panel.panel_kind)
            {
                messages.push(format!(
                    "Panel '{}' kind '{}' is not listed in kit composition allowed_panel_kinds.",
                    panel.id, panel.panel_kind
                ));
            }
            if panel.composition.group_id.trim().is_empty() {
                messages.push(format!(
                    "Panel '{}' has an empty composition group_id.",
                    panel.id
                ));
            }
            if panel.composition.anchor.trim().is_empty() {
                messages.push(format!(
                    "Panel '{}' has an empty composition anchor.",
                    panel.id
                ));
            }
            if !matches!(
                panel.composition.anchor.as_str(),
                "north" | "south" | "east" | "west" | "top" | "bottom" | "center" | "origin"
            ) {
                messages.push(format!(
                    "Panel '{}' has unsupported composition anchor '{}'.",
                    panel.id, panel.composition.anchor
                ));
            }

            let mut occupied_cells = BTreeSet::new();
            for cell in &panel.cells {
                if !occupied_cells.insert((cell.x, cell.y, cell.z)) {
                    messages.push(format!(
                        "Panel '{}' has duplicate cell {},{},{}.",
                        panel.id, cell.x, cell.y, cell.z
                    ));
                }
                if cell.x >= panel.width || cell.y >= panel.height || cell.z >= panel.depth {
                    messages.push(format!(
                        "Panel '{}' has out-of-bounds cell {},{},{}.",
                        panel.id, cell.x, cell.y, cell.z
                    ));
                }
                if !material_ids.contains(cell.material_id.as_str()) {
                    messages.push(format!(
                        "Panel '{}' cell {},{},{} references missing material '{}'.",
                        panel.id, cell.x, cell.y, cell.z, cell.material_id
                    ));
                }
            }

            let mut socket_ids = BTreeSet::new();
            for socket in &panel.sockets {
                if socket.id.trim().is_empty() {
                    messages.push(format!(
                        "Panel '{}' has a socket with an empty id.",
                        panel.id
                    ));
                }
                if !socket_ids.insert(socket.id.as_str()) {
                    messages.push(format!(
                        "Panel '{}' has duplicate socket id '{}'.",
                        panel.id, socket.id
                    ));
                }
                if socket.x >= panel.width || socket.y >= panel.height || socket.z >= panel.depth {
                    messages.push(format!(
                        "Panel '{}' socket '{}' is out of bounds at {},{},{}.",
                        panel.id, socket.id, socket.x, socket.y, socket.z
                    ));
                }
                if !matches!(
                    socket.edge.as_str(),
                    "north" | "south" | "east" | "west" | "top" | "bottom" | "center"
                ) {
                    messages.push(format!(
                        "Panel '{}' socket '{}' has unsupported edge '{}'.",
                        panel.id, socket.id, socket.edge
                    ));
                }
            }
        }

        let panel_lookup = self
            .kit
            .panels
            .iter()
            .map(|panel| (panel.id.as_str(), panel))
            .collect::<BTreeMap<_, _>>();
        let mut composition_ids = BTreeSet::new();
        for composition in &self.kit.compositions {
            if composition.id.trim().is_empty() {
                messages.push("A composition scene has an empty id.".to_string());
            }
            if !composition_ids.insert(composition.id.as_str()) {
                messages.push(format!(
                    "Duplicate composition scene id '{}'.",
                    composition.id
                ));
            }
            if composition.canvas_width == 0
                || composition.canvas_height == 0
                || composition.canvas_depth == 0
            {
                messages.push(format!(
                    "Composition '{}' has a zero canvas dimension.",
                    composition.id
                ));
            }
            if composition.grid_unit_px == 0 {
                messages.push(format!(
                    "Composition '{}' grid_unit_px must be at least 1.",
                    composition.id
                ));
            }
            if !matches!(
                composition.viewport_prep.source_axis.as_str(),
                "xy_depth_z" | "xz_depth_y" | "yz_depth_x"
            ) {
                messages.push(format!(
                    "Composition '{}' has unsupported viewport source_axis '{}'.",
                    composition.id, composition.viewport_prep.source_axis
                ));
            }
            if !matches!(
                composition.viewport_prep.bake_anchor.as_str(),
                "origin" | "center" | "bounds_min"
            ) {
                messages.push(format!(
                    "Composition '{}' has unsupported bake_anchor '{}'.",
                    composition.id, composition.viewport_prep.bake_anchor
                ));
            }
            if composition.viewport_prep.mesh_export_path.trim().is_empty() {
                messages.push(format!(
                    "Composition '{}' has an empty Phase 53i mesh export path.",
                    composition.id
                ));
            }
            if composition.viewport_prep.bake_layer_gap < 0.0 {
                messages.push(format!(
                    "Composition '{}' has a negative bake_layer_gap.",
                    composition.id
                ));
            }

            let mut instance_ids = BTreeSet::new();
            for instance in &composition.instances {
                if instance.id.trim().is_empty() {
                    messages.push(format!(
                        "Composition '{}' has an instance with an empty id.",
                        composition.id
                    ));
                }
                if !instance_ids.insert(instance.id.as_str()) {
                    messages.push(format!(
                        "Composition '{}' has duplicate instance id '{}'.",
                        composition.id, instance.id
                    ));
                }
                let Some(panel) = panel_lookup.get(instance.panel_id.as_str()).copied() else {
                    messages.push(format!(
                        "Composition '{}' instance '{}' references missing panel '{}'.",
                        composition.id, instance.id, instance.panel_id
                    ));
                    continue;
                };
                let (width, height) = voxel_panel_instance_size(panel, instance);
                if instance.x < 0 || instance.y < 0 || instance.z < 0 {
                    messages.push(format!(
                        "Composition '{}' instance '{}' has a negative position.",
                        composition.id, instance.id
                    ));
                }
                if instance.x + (width as i32) > composition.canvas_width as i32
                    || instance.y + (height as i32) > composition.canvas_height as i32
                    || instance.z + (panel.depth as i32) > composition.canvas_depth as i32
                {
                    messages.push(format!(
                        "Composition '{}' instance '{}' exceeds the composition canvas bounds.",
                        composition.id, instance.id
                    ));
                }
                if !matches!(
                    voxel_panel_normalized_rotation(instance.rotation_degrees),
                    0 | 90 | 180 | 270
                ) {
                    messages.push(format!(
                        "Composition '{}' instance '{}' rotation should be 0/90/180/270 degrees.",
                        composition.id, instance.id
                    ));
                }
            }

            let mut connection_ids = BTreeSet::new();
            for connection in &composition.connections {
                if connection.id.trim().is_empty() {
                    messages.push(format!(
                        "Composition '{}' has a connection with an empty id.",
                        composition.id
                    ));
                }
                if !connection_ids.insert(connection.id.as_str()) {
                    messages.push(format!(
                        "Composition '{}' has duplicate connection id '{}'.",
                        composition.id, connection.id
                    ));
                }
                let from_instance = composition
                    .instances
                    .iter()
                    .find(|instance| instance.id == connection.from_instance);
                let to_instance = composition
                    .instances
                    .iter()
                    .find(|instance| instance.id == connection.to_instance);
                let Some(from_instance) = from_instance else {
                    messages.push(format!(
                        "Composition '{}' connection '{}' references missing from_instance '{}'.",
                        composition.id, connection.id, connection.from_instance
                    ));
                    continue;
                };
                let Some(to_instance) = to_instance else {
                    messages.push(format!(
                        "Composition '{}' connection '{}' references missing to_instance '{}'.",
                        composition.id, connection.id, connection.to_instance
                    ));
                    continue;
                };
                let Some(from_panel) = panel_lookup.get(from_instance.panel_id.as_str()).copied()
                else {
                    messages.push(format!(
                        "Composition '{}' connection '{}' has missing from panel '{}'.",
                        composition.id, connection.id, from_instance.panel_id
                    ));
                    continue;
                };
                let Some(to_panel) = panel_lookup.get(to_instance.panel_id.as_str()).copied()
                else {
                    messages.push(format!(
                        "Composition '{}' connection '{}' has missing to panel '{}'.",
                        composition.id, connection.id, to_instance.panel_id
                    ));
                    continue;
                };
                let from_socket = from_panel
                    .sockets
                    .iter()
                    .find(|socket| socket.id == connection.from_socket);
                let to_socket = to_panel
                    .sockets
                    .iter()
                    .find(|socket| socket.id == connection.to_socket);
                let Some(from_socket) = from_socket else {
                    messages.push(format!(
                        "Composition '{}' connection '{}' references missing from_socket '{}'.",
                        composition.id, connection.id, connection.from_socket
                    ));
                    continue;
                };
                let Some(to_socket) = to_socket else {
                    messages.push(format!(
                        "Composition '{}' connection '{}' references missing to_socket '{}'.",
                        composition.id, connection.id, connection.to_socket
                    ));
                    continue;
                };
                if !voxel_panel_sockets_compatible(from_panel, from_socket, to_panel, to_socket) {
                    messages.push(format!(
                        "Composition '{}' connection '{}' links sockets that do not declare compatible accepts metadata.",
                        composition.id, connection.id
                    ));
                }
                let from_world =
                    voxel_panel_socket_world_position(from_panel, from_instance, from_socket);
                let to_world = voxel_panel_socket_world_position(to_panel, to_instance, to_socket);
                if connection.snapped && from_world != to_world {
                    messages.push(format!(
                        "Composition '{}' connection '{}' is marked snapped but socket world positions differ: {:?} vs {:?}.",
                        composition.id, connection.id, from_world, to_world
                    ));
                }
            }
        }

        messages
    }
}

fn voxel_panel_normalized_rotation(degrees: i32) -> i32 {
    degrees.rem_euclid(360)
}

fn voxel_panel_cell_local_position(
    panel: &VoxelPanelDef,
    instance: &VoxelPanelCompositionInstanceDef,
    cell: &VoxelPanelCellDef,
) -> (i32, i32, i32) {
    let width = panel.width.max(1) as i32;
    let height = panel.height.max(1) as i32;
    let mut x = cell.x.min(panel.width.saturating_sub(1)) as i32;
    let mut y = cell.y.min(panel.height.saturating_sub(1)) as i32;

    if instance.mirror_x {
        x = width.saturating_sub(1).saturating_sub(x);
    }
    if instance.mirror_y {
        y = height.saturating_sub(1).saturating_sub(y);
    }

    match voxel_panel_normalized_rotation(instance.rotation_degrees) {
        90 => (height.saturating_sub(1).saturating_sub(y), x, cell.z as i32),
        180 => (
            width.saturating_sub(1).saturating_sub(x),
            height.saturating_sub(1).saturating_sub(y),
            cell.z as i32,
        ),
        270 => (y, width.saturating_sub(1).saturating_sub(x), cell.z as i32),
        _ => (x, y, cell.z as i32),
    }
}

fn voxel_panel_expand_bounds(
    min: &mut [i32; 3],
    max: &mut [i32; 3],
    has_bounds: &mut bool,
    point: [i32; 3],
) {
    if !*has_bounds {
        *min = point;
        *max = point;
        *has_bounds = true;
        return;
    }

    for axis in 0..3 {
        min[axis] = min[axis].min(point[axis]);
        max[axis] = max[axis].max(point[axis]);
    }
}

fn voxel_panel_map_source_axis(point: [i32; 3], source_axis: &str) -> [i32; 3] {
    match source_axis {
        "xz_depth_y" => [point[0], point[2], point[1]],
        "yz_depth_x" => [point[2], point[0], point[1]],
        _ => point,
    }
}

fn voxel_panel_anchor_offset(
    bounds_min: [i32; 3],
    bounds_max: [i32; 3],
    bake_anchor: &str,
) -> [i32; 3] {
    match bake_anchor {
        "bounds_min" => [-bounds_min[0], -bounds_min[1], -bounds_min[2]],
        "center" => [
            -(bounds_min[0] + bounds_max[0]).div_euclid(2),
            -(bounds_min[1] + bounds_max[1]).div_euclid(2),
            -(bounds_min[2] + bounds_max[2]).div_euclid(2),
        ],
        _ => [0, 0, 0],
    }
}

fn voxel_panel_translate_point(point: [i32; 3], offset: [i32; 3]) -> [i32; 3] {
    [
        point[0] + offset[0],
        point[1] + offset[1],
        point[2] + offset[2],
    ]
}

fn voxel_panel_sanitize_file_part(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.trim_matches('_').is_empty() {
        "voxel_panel".to_string()
    } else {
        sanitized
    }
}

fn voxel_panel_socket_local_position(
    panel: &VoxelPanelDef,
    instance: &VoxelPanelCompositionInstanceDef,
    socket: &VoxelPanelSocketDef,
) -> (i32, i32, i32) {
    let width = panel.width.max(1) as i32;
    let height = panel.height.max(1) as i32;
    let mut x = socket.x.min(panel.width.saturating_sub(1)) as i32;
    let mut y = socket.y.min(panel.height.saturating_sub(1)) as i32;

    if instance.mirror_x {
        x = width.saturating_sub(1).saturating_sub(x);
    }
    if instance.mirror_y {
        y = height.saturating_sub(1).saturating_sub(y);
    }

    match voxel_panel_normalized_rotation(instance.rotation_degrees) {
        90 => (
            height.saturating_sub(1).saturating_sub(y),
            x,
            socket.z as i32,
        ),
        180 => (
            width.saturating_sub(1).saturating_sub(x),
            height.saturating_sub(1).saturating_sub(y),
            socket.z as i32,
        ),
        270 => (
            y,
            width.saturating_sub(1).saturating_sub(x),
            socket.z as i32,
        ),
        _ => (x, y, socket.z as i32),
    }
}

fn voxel_panel_socket_world_position(
    panel: &VoxelPanelDef,
    instance: &VoxelPanelCompositionInstanceDef,
    socket: &VoxelPanelSocketDef,
) -> (i32, i32, i32) {
    let local = voxel_panel_socket_local_position(panel, instance, socket);
    (
        instance.x + local.0,
        instance.y + local.1,
        instance.z + local.2,
    )
}

fn voxel_panel_instance_size(
    panel: &VoxelPanelDef,
    instance: &VoxelPanelCompositionInstanceDef,
) -> (u32, u32) {
    match voxel_panel_normalized_rotation(instance.rotation_degrees) {
        90 | 270 => (panel.height, panel.width),
        _ => (panel.width, panel.height),
    }
}

fn voxel_panel_socket_accepts(
    socket: &VoxelPanelSocketDef,
    other_panel: &VoxelPanelDef,
    other_socket: &VoxelPanelSocketDef,
) -> bool {
    socket.accepts.iter().any(|accepted| {
        accepted == &other_panel.id
            || accepted == &other_panel.panel_kind
            || accepted == &other_socket.id
            || accepted == &other_socket.edge
            || accepted == "any"
            || accepted == "*"
    })
}

fn voxel_panel_sockets_compatible(
    a_panel: &VoxelPanelDef,
    a_socket: &VoxelPanelSocketDef,
    b_panel: &VoxelPanelDef,
    b_socket: &VoxelPanelSocketDef,
) -> bool {
    voxel_panel_socket_accepts(a_socket, b_panel, b_socket)
        || voxel_panel_socket_accepts(b_socket, a_panel, a_socket)
        || (!a_socket.accepts.is_empty() && a_socket.accepts == b_socket.accepts)
}

fn voxel_panel_composition_instance_rect(
    composition_rect: egui::Rect,
    cell_size: f32,
    panel: &VoxelPanelDef,
    instance: &VoxelPanelCompositionInstanceDef,
) -> egui::Rect {
    let (width, height) = voxel_panel_instance_size(panel, instance);
    let min = egui::pos2(
        composition_rect.left() + instance.x as f32 * cell_size,
        composition_rect.top() + instance.y as f32 * cell_size,
    );
    egui::Rect::from_min_size(
        min,
        egui::vec2(
            width.max(1) as f32 * cell_size,
            height.max(1) as f32 * cell_size,
        ),
    )
}

fn voxel_panel_kit_paths(project_root: &std::path::Path) -> Vec<PathBuf> {
    let panel_root = project_root
        .join("content")
        .join("editor_voxel_panels")
        .join("panel_kits");
    game_data::loader::ron_files_in(&panel_root).unwrap_or_default()
}

fn voxel_panel_cell_rect(container: egui::Rect, cell_size: f32, x: u32, y: u32) -> egui::Rect {
    let min = egui::pos2(
        container.left() + x as f32 * cell_size,
        container.top() + y as f32 * cell_size,
    );
    egui::Rect::from_min_size(min, egui::vec2(cell_size, cell_size))
}

fn voxel_panel_parse_list(value: &str) -> Vec<String> {
    value
        .split(|ch| ch == ',' || ch == '|')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn voxel_panel_preview_world_point(
    point: [i32; 3],
    dx: f32,
    dy: f32,
    dz: f32,
    export: &VoxelPanelCompositionMeshExportDef,
) -> [f32; 3] {
    let unit = export.voxel_unit.max(0.1);
    let layer_step = unit + export.layer_gap.max(0.0);
    [
        (point[0] as f32 + dx) * unit,
        (point[1] as f32 + dy) * unit,
        point[2] as f32 * layer_step + dz * unit,
    ]
}

fn voxel_panel_preview_center(export: &VoxelPanelCompositionMeshExportDef) -> [f32; 3] {
    let min = voxel_panel_preview_world_point(export.bounds_min, 0.0, 0.0, 0.0, export);
    let max = voxel_panel_preview_world_point(export.bounds_max, 1.0, 1.0, 1.0, export);
    [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ]
}

fn voxel_panel_project_preview_point(
    point: [f32; 3],
    center: [f32; 3],
    camera: &VoxelPanelPreviewCameraState,
    rect: egui::Rect,
) -> egui::Pos2 {
    let x = point[0] - center[0];
    let y = point[1] - center[1];
    let z = point[2] - center[2];
    let yaw = camera.yaw_degrees.to_radians();
    let pitch = camera.pitch_degrees.to_radians();
    let yaw_cos = yaw.cos();
    let yaw_sin = yaw.sin();
    let pitch_cos = pitch.cos();
    let pitch_sin = pitch.sin();

    let rx = x * yaw_cos - y * yaw_sin;
    let ry = x * yaw_sin + y * yaw_cos;
    let vertical = z * pitch_cos - ry * pitch_sin;

    egui::pos2(
        rect.center().x + rx * camera.zoom + camera.pan_x,
        rect.center().y - vertical * camera.zoom + camera.pan_y,
    )
}

fn voxel_panel_preview_depth(
    point: [f32; 3],
    center: [f32; 3],
    camera: &VoxelPanelPreviewCameraState,
) -> f32 {
    let x = point[0] - center[0];
    let y = point[1] - center[1];
    let z = point[2] - center[2];
    let yaw = camera.yaw_degrees.to_radians();
    let pitch = camera.pitch_degrees.to_radians();
    let ry = x * yaw.sin() + y * yaw.cos();
    ry * pitch.cos() + z * pitch.sin()
}

fn voxel_panel_preview_color(rgba: [u8; 4], factor: f32) -> egui::Color32 {
    let scale = |channel: u8| -> u8 { ((channel as f32 * factor).clamp(0.0, 255.0)) as u8 };
    egui::Color32::from_rgba_unmultiplied(scale(rgba[0]), scale(rgba[1]), scale(rgba[2]), rgba[3])
}

fn voxel_panel_draw_preview_grid(painter: &egui::Painter, rect: egui::Rect) {
    let grid_stroke =
        egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(80, 96, 112, 70));
    let step = 24.0;
    let mut x = rect.left();
    while x <= rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            grid_stroke,
        );
        x += step;
    }
    let mut y = rect.top();
    while y <= rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            grid_stroke,
        );
        y += step;
    }
}

fn voxel_panel_draw_preview_floor_grid(
    painter: &egui::Painter,
    rect: egui::Rect,
    export: &VoxelPanelCompositionMeshExportDef,
    camera: &VoxelPanelPreviewCameraState,
    center: [f32; 3],
) {
    let min_x = export.bounds_min[0].saturating_sub(2);
    let max_x = export.bounds_max[0].saturating_add(2);
    let min_y = export.bounds_min[1].saturating_sub(2);
    let max_y = export.bounds_max[1].saturating_add(2);
    let z = export.bounds_min[2];
    let stroke = egui::Stroke::new(
        0.75,
        egui::Color32::from_rgba_unmultiplied(98, 118, 136, 85),
    );
    let major_stroke = egui::Stroke::new(
        1.0,
        egui::Color32::from_rgba_unmultiplied(136, 156, 178, 125),
    );
    for x in min_x..=max_x {
        let a = voxel_panel_project_preview_point(
            voxel_panel_preview_world_point([x, min_y, z], 0.0, 0.0, 0.0, export),
            center,
            camera,
            rect,
        );
        let b = voxel_panel_project_preview_point(
            voxel_panel_preview_world_point([x, max_y, z], 0.0, 1.0, 0.0, export),
            center,
            camera,
            rect,
        );
        painter.line_segment([a, b], if x == 0 { major_stroke } else { stroke });
    }
    for y in min_y..=max_y {
        let a = voxel_panel_project_preview_point(
            voxel_panel_preview_world_point([min_x, y, z], 0.0, 0.0, 0.0, export),
            center,
            camera,
            rect,
        );
        let b = voxel_panel_project_preview_point(
            voxel_panel_preview_world_point([max_x, y, z], 1.0, 0.0, 0.0, export),
            center,
            camera,
            rect,
        );
        painter.line_segment([a, b], if y == 0 { major_stroke } else { stroke });
    }
}

fn voxel_panel_draw_axis_gizmo(
    painter: &egui::Painter,
    rect: egui::Rect,
    camera: &VoxelPanelPreviewCameraState,
) {
    let origin = rect.left_bottom() + egui::vec2(36.0, -34.0);
    let axis_rect = egui::Rect::from_center_size(origin, egui::vec2(1.0, 1.0));
    let mut axis_camera = camera.clone();
    axis_camera.zoom = 30.0;
    axis_camera.pan_x = 0.0;
    axis_camera.pan_y = 0.0;
    let center = [0.0, 0.0, 0.0];
    let project = |point| voxel_panel_project_preview_point(point, center, &axis_camera, axis_rect);
    let axes = [
        ([1.0, 0.0, 0.0], "X", egui::Color32::from_rgb(232, 96, 96)),
        ([0.0, 1.0, 0.0], "Y", egui::Color32::from_rgb(100, 214, 128)),
        ([0.0, 0.0, 1.0], "Z", egui::Color32::from_rgb(102, 174, 255)),
    ];
    painter.circle_filled(
        origin,
        3.0,
        egui::Color32::from_rgba_unmultiplied(238, 244, 250, 210),
    );
    for (axis, label, color) in axes {
        let end = project(axis);
        painter.line_segment([origin, end], egui::Stroke::new(2.0, color));
        painter.circle_filled(end, 3.0, color);
        painter.text(
            end + egui::vec2(4.0, -4.0),
            egui::Align2::LEFT_BOTTOM,
            label,
            egui::FontId::monospace(10.0),
            color,
        );
    }
}

fn voxel_panel_draw_preview_instance_bounds(
    painter: &egui::Painter,
    rect: egui::Rect,
    export: &VoxelPanelCompositionMeshExportDef,
    camera: &VoxelPanelPreviewCameraState,
    center: [f32; 3],
    instance: &VoxelPanelBakedInstanceDef,
    stroke: egui::Stroke,
) {
    let min = instance.bounds_min;
    let max = instance.bounds_max;
    let corners = [
        voxel_panel_preview_world_point(min, 0.0, 0.0, 0.0, export),
        voxel_panel_preview_world_point([max[0], min[1], min[2]], 1.0, 0.0, 0.0, export),
        voxel_panel_preview_world_point([min[0], max[1], min[2]], 0.0, 1.0, 0.0, export),
        voxel_panel_preview_world_point([max[0], max[1], min[2]], 1.0, 1.0, 0.0, export),
        voxel_panel_preview_world_point([min[0], min[1], max[2]], 0.0, 0.0, 1.0, export),
        voxel_panel_preview_world_point([max[0], min[1], max[2]], 1.0, 0.0, 1.0, export),
        voxel_panel_preview_world_point([min[0], max[1], max[2]], 0.0, 1.0, 1.0, export),
        voxel_panel_preview_world_point(max, 1.0, 1.0, 1.0, export),
    ];
    let p = |index: usize| voxel_panel_project_preview_point(corners[index], center, camera, rect);
    for (a, b) in [
        (0, 1),
        (0, 2),
        (1, 3),
        (2, 3),
        (4, 5),
        (4, 6),
        (5, 7),
        (6, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        painter.line_segment([p(a), p(b)], stroke);
    }
}

fn voxel_panel_socket_gizmo_accepts(
    socket: &VoxelPanelSocketGizmoDef,
    other: &VoxelPanelSocketGizmoDef,
) -> bool {
    socket.accepts.iter().any(|accepted| {
        accepted == &other.panel_id
            || accepted == &other.socket_id
            || accepted == &other.edge
            || accepted == "any"
            || accepted == "*"
    })
}

fn voxel_panel_preview_distance_to_segment(point: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> f32 {
    let ab = b - a;
    let ap = point - a;
    let denom = (ab.x * ab.x + ab.y * ab.y).max(0.001);
    let t = ((ap.x * ab.x + ap.y * ab.y) / denom).clamp(0.0, 1.0);
    let closest = a + ab * t;
    point.distance(closest)
}

fn voxel_panel_preview_hover_label(
    export: &VoxelPanelCompositionMeshExportDef,
    camera: &VoxelPanelPreviewCameraState,
    rect: egui::Rect,
    hover_pos: egui::Pos2,
) -> Option<String> {
    let center = voxel_panel_preview_center(export);
    for socket in &export.socket_gizmos {
        let pos = voxel_panel_project_preview_point(
            voxel_panel_preview_world_point(socket.world, 0.5, 0.5, 0.5, export),
            center,
            camera,
            rect,
        );
        if hover_pos.distance(pos) <= 10.0 {
            return Some(format!(
                "socket {}.{} · edge {} · required {} · accepts {}",
                socket.instance_id,
                socket.socket_id,
                socket.edge,
                socket.required,
                if socket.accepts.is_empty() {
                    "<none>".to_string()
                } else {
                    socket.accepts.join(", ")
                }
            ));
        }
    }
    for connection in &export.connection_gizmos {
        let from = voxel_panel_project_preview_point(
            voxel_panel_preview_world_point(connection.from_world, 0.5, 0.5, 0.5, export),
            center,
            camera,
            rect,
        );
        let to = voxel_panel_project_preview_point(
            voxel_panel_preview_world_point(connection.to_world, 0.5, 0.5, 0.5, export),
            center,
            camera,
            rect,
        );
        if voxel_panel_preview_distance_to_segment(hover_pos, from, to) <= 7.0 {
            return Some(format!(
                "connection {} · {}.{} -> {}.{} · snapped {} · offset {:?}",
                connection.connection_id,
                connection.from_instance,
                connection.from_socket,
                connection.to_instance,
                connection.to_socket,
                connection.snapped,
                connection.offset
            ));
        }
    }
    for instance in &export.instances {
        let center_world = [
            (instance.bounds_min[0] + instance.bounds_max[0]) as f32 * 0.5,
            (instance.bounds_min[1] + instance.bounds_max[1]) as f32 * 0.5,
            (instance.bounds_min[2] + instance.bounds_max[2]) as f32 * 0.5,
        ];
        let pos = voxel_panel_project_preview_point(center_world, center, camera, rect);
        if hover_pos.distance(pos) <= 16.0 {
            return Some(format!(
                "instance {} · panel {} · {} voxel(s) · bounds {:?}..{:?}",
                instance.instance_id,
                instance.panel_id,
                instance.voxel_count,
                instance.bounds_min,
                instance.bounds_max
            ));
        }
    }
    None
}

fn voxel_panel_draw_cube_face(
    painter: &egui::Painter,
    points: Vec<egui::Pos2>,
    color: egui::Color32,
    stroke: egui::Stroke,
) {
    painter.add(egui::Shape::convex_polygon(points, color, stroke));
}

fn voxel_panel_draw_preview_cube(
    painter: &egui::Painter,
    rect: egui::Rect,
    export: &VoxelPanelCompositionMeshExportDef,
    camera: &VoxelPanelPreviewCameraState,
    center: [f32; 3],
    voxel: &VoxelPanelBakedVoxelDef,
) {
    let p000 = voxel_panel_preview_world_point(voxel.world, 0.0, 0.0, 0.0, export);
    let p100 = voxel_panel_preview_world_point(voxel.world, 1.0, 0.0, 0.0, export);
    let p010 = voxel_panel_preview_world_point(voxel.world, 0.0, 1.0, 0.0, export);
    let p110 = voxel_panel_preview_world_point(voxel.world, 1.0, 1.0, 0.0, export);
    let p001 = voxel_panel_preview_world_point(voxel.world, 0.0, 0.0, 1.0, export);
    let p101 = voxel_panel_preview_world_point(voxel.world, 1.0, 0.0, 1.0, export);
    let p011 = voxel_panel_preview_world_point(voxel.world, 0.0, 1.0, 1.0, export);
    let p111 = voxel_panel_preview_world_point(voxel.world, 1.0, 1.0, 1.0, export);

    let project = |point| voxel_panel_project_preview_point(point, center, camera, rect);
    let stroke = egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(8, 10, 14, 170));
    voxel_panel_draw_cube_face(
        painter,
        vec![project(p001), project(p101), project(p111), project(p011)],
        voxel_panel_preview_color(voxel.rgba, 1.12),
        stroke,
    );
    voxel_panel_draw_cube_face(
        painter,
        vec![project(p100), project(p110), project(p111), project(p101)],
        voxel_panel_preview_color(voxel.rgba, 0.90),
        stroke,
    );
    voxel_panel_draw_cube_face(
        painter,
        vec![project(p010), project(p110), project(p111), project(p011)],
        voxel_panel_preview_color(voxel.rgba, 0.74),
        stroke,
    );
    painter.line_segment([project(p000), project(p100)], stroke);
    painter.line_segment([project(p000), project(p010)], stroke);
    painter.line_segment([project(p000), project(p001)], stroke);
}

fn voxel_panel_draw_preview_bounds(
    painter: &egui::Painter,
    rect: egui::Rect,
    export: &VoxelPanelCompositionMeshExportDef,
    camera: &VoxelPanelPreviewCameraState,
    center: [f32; 3],
) {
    let min = export.bounds_min;
    let max = export.bounds_max;
    let corners = [
        voxel_panel_preview_world_point(min, 0.0, 0.0, 0.0, export),
        voxel_panel_preview_world_point([max[0], min[1], min[2]], 1.0, 0.0, 0.0, export),
        voxel_panel_preview_world_point([min[0], max[1], min[2]], 0.0, 1.0, 0.0, export),
        voxel_panel_preview_world_point([max[0], max[1], min[2]], 1.0, 1.0, 0.0, export),
        voxel_panel_preview_world_point([min[0], min[1], max[2]], 0.0, 0.0, 1.0, export),
        voxel_panel_preview_world_point([max[0], min[1], max[2]], 1.0, 0.0, 1.0, export),
        voxel_panel_preview_world_point([min[0], max[1], max[2]], 0.0, 1.0, 1.0, export),
        voxel_panel_preview_world_point(max, 1.0, 1.0, 1.0, export),
    ];
    let p = |index: usize| voxel_panel_project_preview_point(corners[index], center, camera, rect);
    let stroke = egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 216, 96));
    for (a, b) in [
        (0, 1),
        (0, 2),
        (1, 3),
        (2, 3),
        (4, 5),
        (4, 6),
        (5, 7),
        (6, 7),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ] {
        painter.line_segment([p(a), p(b)], stroke);
    }
}

fn voxel_panel_draw_3d_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    export: &VoxelPanelCompositionMeshExportDef,
    camera: &VoxelPanelPreviewCameraState,
    selected_instance_id: Option<&str>,
    selected_socket_key: Option<(&str, &str)>,
) {
    let center = voxel_panel_preview_center(export);

    if camera.show_floor_grid {
        voxel_panel_draw_preview_floor_grid(painter, rect, export, camera, center);
    }

    if camera.show_voxels {
        let mut voxels = export.voxels.iter().collect::<Vec<_>>();
        voxels.sort_by(|a, b| {
            let ac = voxel_panel_preview_world_point(a.world, 0.5, 0.5, 0.5, export);
            let bc = voxel_panel_preview_world_point(b.world, 0.5, 0.5, 0.5, export);
            voxel_panel_preview_depth(ac, center, camera)
                .partial_cmp(&voxel_panel_preview_depth(bc, center, camera))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for voxel in voxels {
            voxel_panel_draw_preview_cube(painter, rect, export, camera, center, voxel);
        }
    }

    if camera.show_bounds {
        voxel_panel_draw_preview_bounds(painter, rect, export, camera, center);
        if let Some(selected_instance_id) = selected_instance_id {
            if let Some(instance) = export
                .instances
                .iter()
                .find(|instance| instance.instance_id == selected_instance_id)
            {
                voxel_panel_draw_preview_instance_bounds(
                    painter,
                    rect,
                    export,
                    camera,
                    center,
                    instance,
                    egui::Stroke::new(2.25, egui::Color32::from_rgb(255, 236, 120)),
                );
            }
        }
    }

    if camera.show_connection_gizmos {
        for connection in &export.connection_gizmos {
            let from = voxel_panel_project_preview_point(
                voxel_panel_preview_world_point(connection.from_world, 0.5, 0.5, 0.5, export),
                center,
                camera,
                rect,
            );
            let to = voxel_panel_project_preview_point(
                voxel_panel_preview_world_point(connection.to_world, 0.5, 0.5, 0.5, export),
                center,
                camera,
                rect,
            );
            let selected = selected_instance_id
                .map(|selected| {
                    selected == connection.from_instance || selected == connection.to_instance
                })
                .unwrap_or(false);
            let stroke = if selected {
                egui::Stroke::new(2.75, egui::Color32::from_rgb(255, 236, 120))
            } else if connection.snapped {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(126, 196, 137))
            } else {
                egui::Stroke::new(1.5, egui::Color32::from_rgb(255, 180, 84))
            };
            painter.line_segment([from, to], stroke);
        }
    }

    if camera.show_socket_gizmos {
        for socket in &export.socket_gizmos {
            let pos = voxel_panel_project_preview_point(
                voxel_panel_preview_world_point(socket.world, 0.5, 0.5, 0.5, export),
                center,
                camera,
                rect,
            );
            let selected_socket = selected_socket_key
                .map(|(instance_id, socket_id)| {
                    instance_id == socket.instance_id && socket_id == socket.socket_id
                })
                .unwrap_or(false);
            let color = if selected_socket {
                egui::Color32::from_rgb(255, 248, 154)
            } else if socket.required {
                egui::Color32::from_rgb(255, 216, 96)
            } else {
                egui::Color32::from_rgb(112, 192, 255)
            };
            painter.circle_filled(pos, if selected_socket { 6.0 } else { 4.0 }, color);
            painter.circle_stroke(
                pos,
                if selected_socket { 7.0 } else { 5.0 },
                egui::Stroke::new(1.0, egui::Color32::from_rgb(8, 10, 14)),
            );
            if camera.show_labels {
                let label = if selected_socket {
                    format!("★ {}", socket.socket_id)
                } else {
                    socket.socket_id.clone()
                };
                painter.text(
                    pos + egui::vec2(6.0, -6.0),
                    egui::Align2::LEFT_BOTTOM,
                    label,
                    egui::FontId::monospace(9.0),
                    egui::Color32::from_rgba_unmultiplied(238, 244, 250, 215),
                );
            }
        }
    }

    if camera.show_axis_gizmo {
        voxel_panel_draw_axis_gizmo(painter, rect, camera);
    }
}

fn default_voxel_panel_kit() -> VoxelPanelKitDef {
    VoxelPanelKitDef {
        id: "starter_gui_panel_kit".to_string(),
        display_name: "Starter GUI Panel Kit".to_string(),
        description: "Fallback voxel-pixel GUI kit generated by the editor when no RON kit is available.".to_string(),
        default_palette_id: "default_gui".to_string(),
        composition: VoxelPanelKitCompositionDef {
            target_view: "ui_and_building_panels".to_string(),
            snap_unit_px: 1,
            allowed_panel_kinds: vec!["gui".to_string(), "building_wall".to_string(), "trim".to_string()],
            notes: vec!["Phase 53i mesh-preview exports are consumed by the future 3D viewport and panel composer.".to_string()],
        },
        preview_3d: VoxelPanelPreview3dDef::default(),
        palettes: vec![default_voxel_palette()],
        panels: vec![default_voxel_panel()],
        compositions: vec![default_voxel_panel_composition_scene()],
    }
}

fn default_voxel_panel_composition_scene() -> VoxelPanelCompositionSceneDef {
    VoxelPanelCompositionSceneDef {
        id: "starter_composition".to_string(),
        display_name: "Starter Composition".to_string(),
        canvas_width: 64,
        canvas_height: 40,
        canvas_depth: 8,
        grid_unit_px: 1,
        viewport_prep: VoxelPanelCompositionViewportPrepDef::default(),
        instances: vec![VoxelPanelCompositionInstanceDef {
            id: "fallback_panel_inst_01".to_string(),
            panel_id: "fallback_panel".to_string(),
            x: 4,
            y: 4,
            z: 0,
            rotation_degrees: 0,
            mirror_x: false,
            mirror_y: false,
            locked: false,
        }],
        connections: Vec::new(),
    }
}

fn default_voxel_palette() -> VoxelPanelPaletteDef {
    VoxelPanelPaletteDef {
        id: "default_gui".to_string(),
        display_name: "Default GUI".to_string(),
        materials: vec![
            VoxelPanelMaterialDef {
                id: "frame_dark".to_string(),
                display_name: "Frame Dark".to_string(),
                rgba: [38, 43, 52, 255],
                render_hint: "solid".to_string(),
            },
            VoxelPanelMaterialDef {
                id: "frame_light".to_string(),
                display_name: "Frame Light".to_string(),
                rgba: [87, 101, 117, 255],
                render_hint: "bevel".to_string(),
            },
            VoxelPanelMaterialDef {
                id: "accent".to_string(),
                display_name: "Accent".to_string(),
                rgba: [126, 196, 137, 255],
                render_hint: "emissive_soft".to_string(),
            },
        ],
    }
}

fn default_voxel_panel() -> VoxelPanelDef {
    VoxelPanelDef {
        id: "fallback_panel".to_string(),
        display_name: "Fallback Panel".to_string(),
        panel_kind: "gui".to_string(),
        width: 16,
        height: 9,
        depth: 2,
        composition: VoxelPanelCompositionDef::default(),
        cells: Vec::new(),
        sockets: vec![VoxelPanelSocketDef {
            id: "center_mount".to_string(),
            edge: "center".to_string(),
            x: 8,
            y: 4,
            z: 0,
            accepts: vec!["gui_anchor".to_string()],
            required: false,
        }],
    }
}

fn worldgen_scene_kind_from_shared(kind: shared_types::SceneKind) -> game_worldgen::SceneKind {
    match kind {
        shared_types::SceneKind::StarterFarm => game_worldgen::SceneKind::StarterFarm,
        shared_types::SceneKind::CoastalFarm => game_worldgen::SceneKind::CoastalFarm,
        shared_types::SceneKind::FarmPlot => game_worldgen::SceneKind::FarmPlot,
        shared_types::SceneKind::Town => game_worldgen::SceneKind::Town,
        shared_types::SceneKind::Forest => game_worldgen::SceneKind::Forest,
        shared_types::SceneKind::Beach => game_worldgen::SceneKind::Beach,
        shared_types::SceneKind::Cave => game_worldgen::SceneKind::Cave,
        shared_types::SceneKind::Dungeon => game_worldgen::SceneKind::Dungeon,
        shared_types::SceneKind::Interior => game_worldgen::SceneKind::Interior,
        shared_types::SceneKind::EventMap => game_worldgen::SceneKind::EventMap,
    }
}

fn select_bake_target_layer(
    state: Option<&EditorMapState>,
    generated_layers: &[String],
) -> Option<String> {
    let state = state?;
    for wanted in generated_layers {
        if state.layers.layers.iter().any(|layer| layer.id == *wanted) {
            return Some(wanted.clone());
        }
    }
    if state.layers.layers.iter().any(|layer| layer.id == "ground") {
        return Some("ground".to_string());
    }
    state.layers.layers.first().map(|layer| layer.id.clone())
}

fn select_object_layer(state: Option<&EditorMapState>) -> Option<String> {
    let state = state?;
    for wanted in ["natural_objects", "objects", "placed_objects"] {
        if state.layers.layers.iter().any(|layer| layer.id == wanted) {
            return Some(wanted.to_string());
        }
    }
    None
}

fn terrain_to_ground_tile_id(terrain: SemanticTerrainId) -> &'static str {
    match terrain {
        SemanticTerrainId::Void => "grass_bright",
        SemanticTerrainId::Grass => "grass_bright",
        SemanticTerrainId::GrassDark => "grass_dark",
        SemanticTerrainId::ForestFloor => "grass_dark",
        SemanticTerrainId::FarmableSoil => "dirt",
        SemanticTerrainId::PathDirt => "path_sand",
        SemanticTerrainId::PathStone => "path_sand",
        SemanticTerrainId::Sand => "sand",
        SemanticTerrainId::ShallowWater => "water_shallow",
        SemanticTerrainId::DeepWater => "water_deep",
        SemanticTerrainId::CoastFoam => "water_shallow",
        SemanticTerrainId::Cliff => "cliff",
        SemanticTerrainId::Rock => "grass_dark",
        SemanticTerrainId::TreeSpawn => "grass_dark",
        SemanticTerrainId::WeedSpawn => "grass_dark",
        SemanticTerrainId::BuildingZone => "wood_floor",
        SemanticTerrainId::ExitZone => "path_sand",
        SemanticTerrainId::Protected => "grass_bright",
    }
}

fn object_spawn_to_tile_id(object_id: &str) -> &'static str {
    if object_id.contains("tree") {
        "tree_pine"
    } else if object_id.contains("stone") || object_id.contains("rock") {
        "rock"
    } else {
        "bush"
    }
}

fn ensure_layer_symbol_for_tile(layer: &mut TileLayerDef, tile_id: &str) -> anyhow::Result<char> {
    if let Some(symbol) = layer_symbol_for_tile(layer, tile_id) {
        return Ok(symbol);
    }
    let symbol = allocate_layer_symbol(layer).ok_or_else(|| {
        anyhow::anyhow!(
            "no available legend symbol for tile '{}' on layer '{}'",
            tile_id,
            layer.id
        )
    })?;
    layer.legend.push(LayerLegendEntry {
        symbol: symbol.to_string(),
        tile_id: tile_id.to_string(),
    });
    Ok(symbol)
}

fn normalize_layer_rows(layer: &mut TileLayerDef, width: usize, height: usize) {
    let width = width.max(1);
    let height = height.max(1);
    while layer.rows.len() < height {
        layer.rows.push(".".repeat(width));
    }
    layer.rows.truncate(height);
    for row in &mut layer.rows {
        let mut chars = row.chars().collect::<Vec<_>>();
        if chars.len() < width {
            chars.extend(std::iter::repeat('.').take(width - chars.len()));
        }
        chars.truncate(width);
        *row = chars.into_iter().collect();
    }
}
fn map_layers_path(project_root: &std::path::Path, map_id: &str) -> PathBuf {
    map_content_path(project_root, map_id, "layers.ron")
}

fn map_content_path(project_root: &std::path::Path, map_id: &str, file_name: &str) -> PathBuf {
    project_root
        .join("content")
        .join("maps")
        .join(map_id)
        .join(file_name)
}

fn unique_id<'a>(base: &str, existing: impl Iterator<Item = &'a str>) -> String {
    let existing = existing.collect::<std::collections::HashSet<_>>();
    if !existing.contains(base) {
        return base.to_string();
    }
    for index in 2..1000 {
        let candidate = format!("{base}_{index}");
        if !existing.contains(candidate.as_str()) {
            return candidate;
        }
    }
    format!("{base}_{}", existing.len() + 1)
}

fn save_ron_with_backup<T: Serialize>(
    path: &std::path::Path,
    value: &T,
    phase_tag: &str,
) -> anyhow::Result<Option<PathBuf>> {
    let backup_path = if path.exists() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let backup_path = path.with_file_name(format!(
            "{}.{}.{}.bak.ron",
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("content"),
            phase_tag,
            timestamp
        ));
        std::fs::copy(path, &backup_path).with_context(|| {
            format!(
                "failed to create editor content backup {}",
                backup_path.display()
            )
        })?;
        Some(backup_path)
    } else {
        None
    };
    game_data::loader::save_ron_file(path, value)?;
    Ok(backup_path)
}

fn layer_symbol_for_tile(layer: &TileLayerDef, tile_id: &str) -> Option<char> {
    layer
        .legend
        .iter()
        .find(|entry| entry.tile_id == tile_id)
        .and_then(|entry| entry.symbol.chars().next())
}

fn layer_tile_for_symbol(layer: &TileLayerDef, symbol: char) -> Option<String> {
    layer
        .legend
        .iter()
        .find(|entry| entry.symbol.chars().next() == Some(symbol))
        .map(|entry| entry.tile_id.clone())
}

fn layer_symbol_at(layer: &TileLayerDef, x: usize, y: usize) -> Option<char> {
    layer.rows.get(y)?.chars().nth(x)
}

fn set_layer_symbol_at(layer: &mut TileLayerDef, x: usize, y: usize, symbol: char) -> bool {
    let Some(row) = layer.rows.get_mut(y) else {
        return false;
    };
    let mut chars = row.chars().collect::<Vec<_>>();
    let Some(cell) = chars.get_mut(x) else {
        return false;
    };
    if *cell == symbol {
        return false;
    }
    *cell = symbol;
    *row = chars.into_iter().collect();
    true
}

fn allocate_layer_symbol(layer: &TileLayerDef) -> Option<char> {
    let used = layer
        .legend
        .iter()
        .filter_map(|entry| entry.symbol.chars().next())
        .collect::<std::collections::HashSet<_>>();
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@$%^&*+=?:;~"
        .chars()
        .find(|candidate| *candidate != '.' && !used.contains(candidate))
}

fn layer_dimensions(layers: &MapLayersDef) -> (u32, u32) {
    let width = layers
        .layers
        .iter()
        .flat_map(|layer| layer.rows.iter())
        .map(|row| row.chars().count() as u32)
        .max()
        .unwrap_or(1)
        .max(1);
    let height = layers
        .layers
        .iter()
        .map(|layer| layer.rows.len() as u32)
        .max()
        .unwrap_or(1)
        .max(1);
    (width, height)
}

fn is_empty_layer_symbol(symbol: char) -> bool {
    symbol == '.' || symbol == ' '
}

fn normalized_map_rect(start: (u32, u32), end: (u32, u32)) -> (i32, i32, i32, i32) {
    let left = start.0.min(end.0) as i32;
    let top = start.1.min(end.1) as i32;
    let right = start.0.max(end.0) as i32;
    let bottom = start.1.max(end.1) as i32;
    (left, top, right, bottom)
}

fn rects_overlap(
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    other_left: i32,
    other_top: i32,
    other_right: i32,
    other_bottom: i32,
) -> bool {
    left <= other_right && right >= other_left && top <= other_bottom && bottom >= other_top
}

fn pixel_editor_texture_path(
    project_root: &std::path::Path,
    registry: &ContentRegistry,
    active_map_id: &str,
) -> PathBuf {
    let texture = registry
        .maps
        .get(active_map_id)
        .and_then(|map| registry.tilesets.get(&map.metadata.tileset))
        .or_else(|| registry.tilesets.values().next())
        .map(|tileset| tileset.texture_path.clone())
        .unwrap_or_else(|| "assets/textures/terrain_atlas_phase17_generated.png".to_string());
    project_root.join(texture.replace('\\', "/"))
}
pub fn run_editor_egui() -> anyhow::Result<()> {
    let project_root = locate_project_root()?;
    let registry = game_data::load_registry(&project_root)
        .context("failed to load content registry for egui editor")?;

    editor_core::init_with_registry(&registry)
        .context("failed to initialize editor core for egui editor")?;

    let active_map_id = if registry.maps.contains_key("starter_farm") {
        "starter_farm".to_string()
    } else if registry.maps.contains_key("autotile_test_coast") {
        "autotile_test_coast".to_string()
    } else {
        registry
            .maps
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "starter_farm".to_string())
    };

    write_editor_live_preview_manifest(&project_root, &active_map_id)
        .context("failed to write egui editor live-preview manifest")?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Starlight Ridge Editor")
            .with_inner_size([1500.0, 900.0])
            .with_min_inner_size([1100.0, 700.0]),
        ..Default::default()
    };

    let app = StarlightRidgeEguiEditor::new(project_root, registry, active_map_id)?;
    eframe::run_native(
        "Starlight Ridge Editor",
        options,
        Box::new(move |_cc| Ok(Box::new(app))),
    )
    .map_err(|error| anyhow::anyhow!("failed to run egui editor: {error}"))
}

struct ContentReloadJob {
    label: String,
    started_at: Instant,
    handle: JoinHandle<anyhow::Result<ContentReloadPayload>>,
}

struct ContentReloadPayload {
    active_map_id: String,
    registry: ContentRegistry,
    tile_map: Option<TileMapRenderData>,
    editor_map: Option<EditorMapState>,
    world_placements: Option<WorldPlacementState>,
    vox_assets: Vec<VoxAssetInfo>,
    vox_scan_error: Option<String>,
}

struct StarlightRidgeEguiEditor {
    project_root: PathBuf,
    registry: ContentRegistry,
    active_map_id: String,
    tile_map: Option<TileMapRenderData>,
    editor_map: Option<EditorMapState>,
    world_placements: Option<WorldPlacementState>,
    map_brush_size: u32,
    selected_tool: usize,
    selected_asset_index: usize,
    selected_tile_id: String,
    selected_cell: (u32, u32),
    role_state: TileRoleState,
    selected_map_cell: Option<(u32, u32)>,
    world_marquee_start: Option<(u32, u32)>,
    world_marquee_end: Option<(u32, u32)>,
    world_resize_active: bool,
    editor_selection: EditorSelection,
    left_tab: LeftTab,
    right_tab: RightTab,
    bottom_tab: BottomTab,
    workspace_tab: WorkspaceTab,
    asset_subtab: AssetSubTab,
    world_subtab: WorldSubTab,
    world_object_filter: WorldObjectFilter,
    voxel_panel_left_tab: VoxelPanelLeftTab,
    voxel_panel_right_tab: VoxelPanelRightTab,
    logic_subtab: LogicSubTab,
    project_subtab: ProjectSubTab,
    animation_subtab: AnimationSubTab,
    character_subtab: CharacterSubTab,
    data_subtab: DataSubTab,
    playtest_subtab: PlaytestSubTab,
    settings_subtab: SettingsSubTab,
    tile_filter: String,
    status: String,
    log_lines: Vec<String>,
    show_grid: bool,
    show_transitions: bool,
    preview_zoom: f32,
    shell_render_depth: u8,
    vox_assets: Vec<VoxAssetInfo>,
    selected_vox_index: usize,
    loaded_vox_cache: Option<(String, VoxModel)>,
    generator_log: Vec<String>,
    pixel_editor: PixelEditorState,
    voxel_panel_designer: VoxelPanelDesignerState,
    worldgen_bake_preview: Option<WorldgenBakePreviewState>,
    content_reload_job: Option<ContentReloadJob>,
}

impl StarlightRidgeEguiEditor {
    fn new(
        project_root: PathBuf,
        registry: ContentRegistry,
        active_map_id: String,
    ) -> anyhow::Result<Self> {
        let tile_map = build_tile_map_render_data(&project_root, &registry, &active_map_id)
            .with_context(|| {
                format!("failed to build egui editor preview for map '{active_map_id}'")
            })?;
        let selected_tile_id =
            default_selected_tile(&registry).unwrap_or_else(|| "grass_bright".to_string());
        let selected_cell = atlas_cell_for_tile(&registry, &selected_tile_id).unwrap_or((0, 0));
        let role_state = load_tile_role_state(&project_root, &selected_tile_id);
        let editor_map = match EditorMapState::load(&project_root, &active_map_id) {
            Ok(state) => Some(state),
            Err(error) => {
                log::warn!("failed to load mutable egui map state: {error:#}");
                None
            }
        };
        let world_placements = match WorldPlacementState::load(&project_root, &active_map_id) {
            Ok(state) => Some(state),
            Err(error) => {
                log::warn!("failed to load editable world placement state: {error:#}");
                None
            }
        };
        let pixel_editor =
            PixelEditorState::load_for_active_tileset(&project_root, &registry, &active_map_id);
        let voxel_panel_designer = VoxelPanelDesignerState::load(&project_root);
        let vox_assets = match scan_vox_files(&project_root) {
            Ok(assets) => assets,
            Err(error) => {
                log::warn!("failed to scan .vox assets: {error:#}");
                Vec::new()
            }
        };

        let mut editor = Self {
            project_root,
            registry,
            active_map_id,
            tile_map,
            editor_map,
            world_placements,
            map_brush_size: 1,
            selected_tool: 0,
            selected_asset_index: 0,
            selected_tile_id,
            selected_cell,
            role_state,
            selected_map_cell: None,
            world_marquee_start: None,
            world_marquee_end: None,
            world_resize_active: false,
            editor_selection: EditorSelection::Tile,
            left_tab: LeftTab::Project,
            right_tab: RightTab::Tile,
            bottom_tab: BottomTab::Console,
            workspace_tab: WorkspaceTab::Assets,
            asset_subtab: AssetSubTab::TerrainAtlas,
            world_subtab: WorldSubTab::MapPaint,
            world_object_filter: WorldObjectFilter::All,
            voxel_panel_left_tab: VoxelPanelLeftTab::Panels,
            voxel_panel_right_tab: VoxelPanelRightTab::Slice,
            logic_subtab: LogicSubTab::Graphs,
            project_subtab: ProjectSubTab::Overview,
            animation_subtab: AnimationSubTab::Clips,
            character_subtab: CharacterSubTab::Bases,
            data_subtab: DataSubTab::Items,
            playtest_subtab: PlaytestSubTab::Launch,
            settings_subtab: SettingsSubTab::Preferences,
            tile_filter: String::new(),
            status: "egui editor ready. Native GL overlay is no longer the editor UI path."
                .to_string(),
            log_lines: Vec::new(),
            show_grid: true,
            show_transitions: true,
            preview_zoom: 1.0,
            shell_render_depth: 0,
            vox_assets,
            selected_vox_index: 0,
            loaded_vox_cache: None,
            generator_log: Vec::new(),
            pixel_editor,
            voxel_panel_designer,
            worldgen_bake_preview: None,
            content_reload_job: None,
        };
        editor.log("Phase 36 egui editor shell initialized.");
        editor.log(format!("Loaded content: {}", editor.registry.summary()));
        editor.log(format!("Active map: {}", editor.active_map_id));
        Ok(editor)
    }

    fn log(&mut self, message: impl Into<String>) {
        let message = message.into();
        log::info!("{message}");
        self.log_lines.push(message);
        if self.log_lines.len() > 200 {
            let drop_count = self.log_lines.len() - 200;
            self.log_lines.drain(0..drop_count);
        }
    }

    fn begin_shell_render(&mut self) {
        self.shell_render_depth = self.shell_render_depth.saturating_add(1);
        if self.shell_render_depth > 1 {
            let message = "EGUI SHELL NESTING REGRESSION: attempted to render the full editor shell inside itself.";
            self.status = message.to_string();
            log::error!("{message}");
            debug_assert!(self.shell_render_depth <= 1, "{message}");
        }
    }

    fn end_shell_render(&mut self) {
        self.shell_render_depth = self.shell_render_depth.saturating_sub(1);
    }

    fn active_tileset(&self) -> Option<&TilesetDef> {
        self.registry
            .maps
            .get(&self.active_map_id)
            .and_then(|map| self.registry.tilesets.get(&map.metadata.tileset))
            .or_else(|| self.registry.tilesets.values().next())
    }

    fn load_editor_map_state(
        &mut self,
        preferred_layer_id: Option<&str>,
        preferred_symbol: Option<char>,
    ) {
        match EditorMapState::load(&self.project_root, &self.active_map_id) {
            Ok(mut state) => {
                if let Some(layer_id) = preferred_layer_id {
                    state.select_layer_by_id(layer_id);
                }
                if let Some(symbol) = preferred_symbol {
                    state.selected_symbol = symbol;
                }
                self.editor_map = Some(state);
                self.sync_selected_symbol_to_tile();
            }
            Err(error) => {
                self.editor_map = None;
                self.log(format!("Editable map layer load error: {error:#}"));
            }
        }
    }

    fn load_world_placement_state(&mut self) {
        match WorldPlacementState::load(&self.project_root, &self.active_map_id) {
            Ok(state) => {
                self.world_placements = Some(state);
            }
            Err(error) => {
                self.world_placements = None;
                self.log(format!("World placement load error: {error:#}"));
            }
        }
    }

    fn sync_selected_symbol_to_tile(&mut self) {
        let tile_id = self.selected_tile_id.clone();
        if let Some(state) = self.editor_map.as_mut() {
            if let Some(layer) = state.selected_layer() {
                if let Some(symbol) = layer_symbol_for_tile(layer, &tile_id) {
                    state.selected_symbol = symbol;
                }
            }
        }
    }

    fn ensure_selected_symbol_for_paint(&mut self) -> Option<char> {
        let tile_id = self.selected_tile_id.clone();
        let before_layers = self.editor_map.as_ref().map(|state| state.layers.clone());
        let result = {
            let state = self.editor_map.as_mut()?;
            let layer = state.selected_layer_mut()?;
            if layer.locked {
                self.status = format!("Layer '{}' is locked.", layer.id);
                return None;
            }
            if let Some(symbol) = layer_symbol_for_tile(layer, &tile_id) {
                Some((symbol, false))
            } else if let Some(symbol) = allocate_layer_symbol(layer) {
                layer.legend.push(LayerLegendEntry {
                    symbol: symbol.to_string(),
                    tile_id: tile_id.clone(),
                });
                Some((symbol, true))
            } else {
                None
            }
        }?;

        if let Some(state) = self.editor_map.as_mut() {
            state.selected_symbol = result.0;
            if result.1 {
                if let Some(layers) = before_layers {
                    state.push_history_value("add layer legend symbol", layers);
                }
                state.dirty = true;
                self.status = format!(
                    "Added layer legend symbol '{}' for tile {} on layer {}.",
                    result.0,
                    tile_id,
                    state.selected_layer_id()
                );
            }
        }

        Some(result.0)
    }

    fn save_active_map_layers(&mut self) {
        if let Some(state) = self.editor_map.as_mut() {
            state.finish_stroke();
        }
        let Some(state) = self.editor_map.as_ref() else {
            self.status = "No editable map layers are loaded.".to_string();
            self.log(self.status.clone());
            return;
        };

        let path = state.layers_path.clone();
        let layers = state.layers.clone();
        match game_data::loader::save_map_layers_with_backup(&path, &layers) {
            Ok(backup_path) => {
                if let Some(state) = self.editor_map.as_mut() {
                    state.dirty = false;
                }
                let backup_note = backup_path
                    .as_ref()
                    .map(|path| format!(" Backup: {}.", path.display()))
                    .unwrap_or_else(|| {
                        " No previous layers.ron existed, so no backup was written.".to_string()
                    });
                let _ = write_editor_live_preview_manifest(&self.project_root, &self.active_map_id);
                self.reload_content();
                self.status = format!(
                    "Saved editable map layers to {}.{}",
                    path.display(),
                    backup_note
                );
                self.log(self.status.clone());
            }
            Err(error) => {
                self.status = "Failed to save editable map layers.".to_string();
                self.log(format!("Map layer save error: {error:#}"));
            }
        }
    }

    fn active_map_dimensions(&self) -> (u32, u32) {
        let metadata_dims = self
            .registry
            .maps
            .get(&self.active_map_id)
            .map(|map| (map.metadata.width.max(1), map.metadata.height.max(1)))
            .unwrap_or((1, 1));
        if let Some(state) = &self.editor_map {
            let layer_dims = layer_dimensions(&state.layers);
            (
                metadata_dims.0.max(layer_dims.0),
                metadata_dims.1.max(layer_dims.1),
            )
        } else if let Some(tile_map) = &self.tile_map {
            (
                metadata_dims.0.max(tile_map.map_width),
                metadata_dims.1.max(tile_map.map_height),
            )
        } else {
            metadata_dims
        }
    }

    fn tile_color_from_id(&self, tile_id: &str) -> egui::Color32 {
        let resolved_tile_id = self
            .registry
            .terrain_types
            .get(tile_id)
            .map(|terrain| terrain.fallback_tile_id.as_str())
            .unwrap_or(tile_id);

        if let Some((atlas_x, atlas_y)) = self.active_tileset().and_then(|tileset| {
            tileset
                .named_tiles
                .iter()
                .find(|entry| entry.id == resolved_tile_id)
                .map(|entry| (entry.x, entry.y))
        }) {
            color_for_tile(&TileInstance {
                x: 0,
                y: 0,
                atlas_x,
                atlas_y,
            })
        } else {
            egui::Color32::from_rgb(205, 74, 120)
        }
    }

    fn paint_symbol_at_current_layer(&mut self, x: u32, y: u32, symbol: char) -> usize {
        let size = self.map_brush_size.max(1) as i32;
        let start = -(size / 2);
        let mut changed = 0usize;
        if let Some(state) = self.editor_map.as_mut() {
            let before_layers = state.layers.clone();
            if let Some(layer) = state.selected_layer_mut() {
                if layer.locked {
                    self.status = format!("Layer '{}' is locked.", layer.id);
                    return 0;
                }
                for dy in 0..size {
                    for dx in 0..size {
                        let px = x as i32 + start + dx;
                        let py = y as i32 + start + dy;
                        if px < 0 || py < 0 {
                            continue;
                        }
                        if set_layer_symbol_at(layer, px as usize, py as usize, symbol) {
                            changed += 1;
                        }
                    }
                }
            }
            if changed > 0 {
                let label = if is_empty_layer_symbol(symbol) {
                    "erase map cells"
                } else {
                    "paint map cells"
                };
                if !state.mark_stroke_changed() {
                    state.push_history_value(label, before_layers);
                }
                state.dirty = true;
            }
        }
        changed
    }

    fn fill_current_layer(&mut self, x: u32, y: u32, replacement: char) -> usize {
        let mut changed = 0usize;
        if let Some(state) = self.editor_map.as_mut() {
            let before_layers = state.layers.clone();
            if let Some(layer) = state.selected_layer_mut() {
                if layer.locked {
                    self.status = format!("Layer '{}' is locked.", layer.id);
                    return 0;
                }
                let Some(target) = layer_symbol_at(layer, x as usize, y as usize) else {
                    return 0;
                };
                if target == replacement {
                    return 0;
                }
                let width = layer
                    .rows
                    .iter()
                    .map(|row| row.chars().count())
                    .max()
                    .unwrap_or(0);
                let height = layer.rows.len();
                let mut queue = std::collections::VecDeque::new();
                let mut visited = std::collections::HashSet::new();
                queue.push_back((x as usize, y as usize));

                while let Some((cx, cy)) = queue.pop_front() {
                    if cx >= width || cy >= height || !visited.insert((cx, cy)) {
                        continue;
                    }
                    if layer_symbol_at(layer, cx, cy) != Some(target) {
                        continue;
                    }
                    if set_layer_symbol_at(layer, cx, cy, replacement) {
                        changed += 1;
                    }
                    if cx > 0 {
                        queue.push_back((cx - 1, cy));
                    }
                    if cy > 0 {
                        queue.push_back((cx, cy - 1));
                    }
                    queue.push_back((cx + 1, cy));
                    queue.push_back((cx, cy + 1));
                }
            }
            if changed > 0 {
                state.push_history_value("fill map layer", before_layers);
                state.dirty = true;
            }
        }
        changed
    }

    fn undo_map_layer_edit(&mut self) -> bool {
        let Some(state) = self.editor_map.as_mut() else {
            return false;
        };
        if let Some(label) = state.undo_layers() {
            self.status = format!("Undid {label}.");
            true
        } else {
            self.status = "No map layer edit to undo.".to_string();
            false
        }
    }

    fn redo_map_layer_edit(&mut self) -> bool {
        let Some(state) = self.editor_map.as_mut() else {
            return false;
        };
        if let Some(label) = state.redo_layers() {
            self.status = format!("Redid {label}.");
            true
        } else {
            self.status = "No map layer edit to redo.".to_string();
            false
        }
    }

    fn save_world_props(&mut self) {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable prop placement state is loaded.".to_string();
            self.log(self.status.clone());
            return;
        };
        match save_ron_with_backup(&state.props_path, &state.props, "editor_props") {
            Ok(backup) => {
                state.props_dirty = false;
                self.status = format!(
                    "Saved props.ron for {}{}.",
                    state.map_id,
                    backup
                        .as_ref()
                        .map(|path| format!(" Backup: {}", path.display()))
                        .unwrap_or_default()
                );
                self.log(self.status.clone());
                self.reload_content();
            }
            Err(error) => {
                self.status = "Failed to save props.ron.".to_string();
                self.log(format!("Prop save error: {error:#}"));
            }
        }
    }

    fn save_world_spawns(&mut self) {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable spawn state is loaded.".to_string();
            self.log(self.status.clone());
            return;
        };
        match save_ron_with_backup(&state.spawns_path, &state.spawns, "editor_spawns") {
            Ok(backup) => {
                state.spawns_dirty = false;
                self.status = format!(
                    "Saved spawns.ron for {}{}.",
                    state.map_id,
                    backup
                        .as_ref()
                        .map(|path| format!(" Backup: {}", path.display()))
                        .unwrap_or_default()
                );
                self.log(self.status.clone());
                self.reload_content();
            }
            Err(error) => {
                self.status = "Failed to save spawns.ron.".to_string();
                self.log(format!("Spawn save error: {error:#}"));
            }
        }
    }

    fn save_world_triggers(&mut self) {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable trigger state is loaded.".to_string();
            self.log(self.status.clone());
            return;
        };
        match save_ron_with_backup(&state.triggers_path, &state.triggers, "editor_triggers") {
            Ok(backup) => {
                state.triggers_dirty = false;
                self.status = format!(
                    "Saved triggers.ron for {}{}.",
                    state.map_id,
                    backup
                        .as_ref()
                        .map(|path| format!(" Backup: {}", path.display()))
                        .unwrap_or_default()
                );
                self.log(self.status.clone());
                self.reload_content();
            }
            Err(error) => {
                self.status = "Failed to save triggers.ron.".to_string();
                self.log(format!("Trigger save error: {error:#}"));
            }
        }
    }

    fn save_world_voxel_objects(&mut self) {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable voxel object state is loaded.".to_string();
            self.log(self.status.clone());
            return;
        };
        state.voxel_objects.map_id = state.map_id.clone();
        match save_ron_with_backup(
            &state.voxel_objects_path,
            &state.voxel_objects,
            "editor_voxel_objects",
        ) {
            Ok(backup) => {
                state.voxel_objects_dirty = false;
                self.status = format!(
                    "Saved voxel_objects.ron for {}{}.",
                    state.map_id,
                    backup
                        .as_ref()
                        .map(|path| format!(" Backup: {}", path.display()))
                        .unwrap_or_default()
                );
                self.log(self.status.clone());
                self.reload_content();
            }
            Err(error) => {
                self.status = "Failed to save voxel_objects.ron.".to_string();
                self.log(format!("Voxel object save error: {error:#}"));
            }
        }
    }

    fn save_context(&mut self) {
        match self.workspace_tab {
            WorkspaceTab::World => self.save_world_dirty_assets(),
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::VoxelPanels => {
                self.save_voxel_panel_kit()
            }
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::PixelEditor => {
                match self.pixel_editor.save_png_with_backup() {
                    Ok(path) => {
                        self.status = format!("Saved edited atlas PNG: {}", path.display());
                        self.log(self.status.clone());
                        self.reload_content();
                    }
                    Err(error) => {
                        self.status = "Failed to save edited atlas PNG.".to_string();
                        self.log(format!("Pixel editor save error: {error:#}"));
                    }
                }
            }
            _ => self.write_selection_manifest(),
        }
    }

    fn save_all_dirty(&mut self) {
        let mut saved_any = false;
        if self
            .editor_map
            .as_ref()
            .map(|state| state.dirty)
            .unwrap_or(false)
        {
            self.save_active_map_layers();
            saved_any = true;
        }

        let dirty_placements = self
            .world_placements
            .as_ref()
            .map(|state| {
                (
                    state.props_dirty,
                    state.spawns_dirty,
                    state.triggers_dirty,
                    state.voxel_objects_dirty,
                )
            })
            .unwrap_or((false, false, false, false));
        if dirty_placements.0 {
            self.save_world_props();
            saved_any = true;
        }
        if dirty_placements.1 {
            self.save_world_spawns();
            saved_any = true;
        }
        if dirty_placements.2 {
            self.save_world_triggers();
            saved_any = true;
        }
        if dirty_placements.3 {
            self.save_world_voxel_objects();
            saved_any = true;
        }
        if self.workspace_tab == WorkspaceTab::Assets
            && self.asset_subtab == AssetSubTab::VoxelPanels
            && self.voxel_panel_designer.dirty
        {
            self.save_voxel_panel_kit();
            saved_any = true;
        }
        if self.workspace_tab == WorkspaceTab::Assets
            && self.asset_subtab == AssetSubTab::PixelEditor
            && self.pixel_editor.dirty
        {
            match self.pixel_editor.save_png_with_backup() {
                Ok(path) => {
                    self.status = format!("Saved edited atlas PNG: {}", path.display());
                    self.log(self.status.clone());
                    self.reload_content();
                    saved_any = true;
                }
                Err(error) => {
                    self.status = "Failed to save edited atlas PNG.".to_string();
                    self.log(format!("Pixel editor save error: {error:#}"));
                }
            }
        }

        if !saved_any {
            self.status = "No dirty editor assets to save.".to_string();
        }
    }

    fn save_world_dirty_assets(&mut self) {
        let layers_dirty = self
            .editor_map
            .as_ref()
            .map(|state| state.dirty)
            .unwrap_or(false);
        let placement_dirty = self
            .world_placements
            .as_ref()
            .map(|state| {
                state.props_dirty
                    || state.spawns_dirty
                    || state.triggers_dirty
                    || state.voxel_objects_dirty
            })
            .unwrap_or(false);

        if layers_dirty {
            self.save_active_map_layers();
        }
        if let Some(state) = self.world_placements.as_ref() {
            let props_dirty = state.props_dirty;
            let spawns_dirty = state.spawns_dirty;
            let triggers_dirty = state.triggers_dirty;
            let voxel_objects_dirty = state.voxel_objects_dirty;
            if props_dirty {
                self.save_world_props();
            }
            if spawns_dirty {
                self.save_world_spawns();
            }
            if triggers_dirty {
                self.save_world_triggers();
            }
            if voxel_objects_dirty {
                self.save_world_voxel_objects();
            }
        }
        if !layers_dirty && !placement_dirty {
            self.status = "No dirty world assets to save.".to_string();
        }
    }

    fn context_can_undo(&self) -> bool {
        match self.workspace_tab {
            WorkspaceTab::World => self
                .editor_map
                .as_ref()
                .map(|state| state.history.can_undo())
                .unwrap_or(false),
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::PixelEditor => {
                !self.pixel_editor.undo_stack.is_empty()
            }
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::VoxelPanels => {
                self.voxel_panel_designer.panel_undo.can_undo()
            }
            _ => false,
        }
    }

    fn context_can_redo(&self) -> bool {
        match self.workspace_tab {
            WorkspaceTab::World => self
                .editor_map
                .as_ref()
                .map(|state| state.history.can_redo())
                .unwrap_or(false),
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::PixelEditor => {
                !self.pixel_editor.redo_stack.is_empty()
            }
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::VoxelPanels => {
                self.voxel_panel_designer.panel_undo.can_redo()
            }
            _ => false,
        }
    }

    fn undo_context(&mut self) {
        match self.workspace_tab {
            WorkspaceTab::World => {
                self.undo_map_layer_edit();
            }
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::PixelEditor => {
                if let Some(label) = self.pixel_editor.undo() {
                    self.status = format!("Undid {label}.");
                } else {
                    self.status = "No pixel edit to undo.".to_string();
                }
            }
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::VoxelPanels => {
                if let Some(label) = self.voxel_panel_designer.undo_panels() {
                    self.status = format!("Undid voxel panel edit: {label}.");
                } else {
                    self.status = "No voxel panel edit to undo.".to_string();
                }
            }
            _ => {
                self.status = "No undo action for this workspace yet.".to_string();
            }
        }
    }

    fn redo_context(&mut self) {
        match self.workspace_tab {
            WorkspaceTab::World => {
                self.redo_map_layer_edit();
            }
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::PixelEditor => {
                if self.pixel_editor.redo().is_some() {
                    self.status = "Redid pixel edit.".to_string();
                } else {
                    self.status = "No pixel edit to redo.".to_string();
                }
            }
            WorkspaceTab::Assets if self.asset_subtab == AssetSubTab::VoxelPanels => {
                if let Some(label) = self.voxel_panel_designer.redo_panels() {
                    self.status = format!("Redid voxel panel edit: {label}.");
                } else {
                    self.status = "No voxel panel edit to redo.".to_string();
                }
            }
            _ => {
                self.status = "No redo action for this workspace yet.".to_string();
            }
        }
    }

    fn pick_from_current_layer(&mut self, x: u32, y: u32) {
        let picked = self
            .editor_map
            .as_ref()
            .and_then(|state| state.selected_layer())
            .and_then(|layer| {
                let symbol = layer_symbol_at(layer, x as usize, y as usize)?;
                if is_empty_layer_symbol(symbol) {
                    return None;
                }
                layer_tile_for_symbol(layer, symbol).map(|tile_id| (symbol, tile_id))
            });

        if let Some((symbol, tile_id)) = picked {
            if let Some(state) = self.editor_map.as_mut() {
                state.selected_symbol = symbol;
            }
            self.select_tile(tile_id, "Map pick");
            self.status = format!(
                "Picked '{}' / {} from {},{}.",
                symbol, self.selected_tile_id, x, y
            );
        } else {
            self.status = format!("No mapped tile on selected layer at {},{}.", x, y);
        }
    }

    fn apply_map_tool_at_cell(&mut self, x: u32, y: u32, clicked: bool) {
        self.selected_map_cell = Some((x, y));
        match self.selected_tool {
            2 => {
                if let Some(symbol) = self.ensure_selected_symbol_for_paint() {
                    let changed = self.paint_symbol_at_current_layer(x, y, symbol);
                    if changed > 0 {
                        self.status = format!(
                            "Painted {} cell(s) with '{}' / {} on layer {}.",
                            changed,
                            symbol,
                            self.selected_tile_id,
                            self.editor_map
                                .as_ref()
                                .map(|state| state.selected_layer_id())
                                .unwrap_or_else(|| "<none>".to_string())
                        );
                    }
                } else {
                    self.status = "Could not allocate a layer legend symbol for the selected tile."
                        .to_string();
                }
            }
            3 => {
                let changed = self.paint_symbol_at_current_layer(x, y, '.');
                if changed > 0 {
                    self.status = format!(
                        "Erased {} cell(s) on layer {}.",
                        changed,
                        self.editor_map
                            .as_ref()
                            .map(|state| state.selected_layer_id())
                            .unwrap_or_else(|| "<none>".to_string())
                    );
                }
            }
            4 if clicked => {
                if let Some(symbol) = self.ensure_selected_symbol_for_paint() {
                    let changed = self.fill_current_layer(x, y, symbol);
                    self.status = format!(
                        "Filled {} cell(s) with '{}' / {}.",
                        changed, symbol, self.selected_tile_id
                    );
                }
            }
            5 if clicked => self.pick_from_current_layer(x, y),
            _ if clicked => {
                if self.select_world_placement_at_cell(x, y) {
                    return;
                }
                self.editor_selection = EditorSelection::MapCell;
                if let Some(tile_id) = self.tile_id_at_map_cell(x, y) {
                    self.select_tile(tile_id, "World preview");
                } else {
                    self.status = format!("Selected empty map cell {x},{y}.");
                }
            }
            _ => {}
        }
    }

    fn select_world_placement_at_cell(&mut self, x: u32, y: u32) -> bool {
        let Some(state) = self.world_placements.as_mut() else {
            return false;
        };
        let cell_x = x as i64;
        let cell_y = y as i64;

        if let Some((index, id)) = state
            .voxel_objects
            .objects
            .iter()
            .enumerate()
            .rev()
            .find(|(_, object)| {
                let left = object.x.floor() as i64;
                let top = object.y.floor() as i64;
                let right = left + object.footprint_width.max(1.0).ceil() as i64;
                let bottom = top + object.footprint_height.max(1.0).ceil() as i64;
                cell_x >= left && cell_y >= top && cell_x < right && cell_y < bottom
            })
            .map(|(index, object)| (index, object.id.clone()))
        {
            state.selected_voxel_object_index = index;
            state.active_selection = WorldPlacementKind::VoxelObject;
            self.editor_selection = EditorSelection::VoxelObject;
            self.status = format!("Selected voxel object '{id}' at {x},{y}.");
            return true;
        }

        if let Some((index, id)) = state
            .props
            .iter()
            .enumerate()
            .rev()
            .find(|(_, prop)| prop.x as i64 == cell_x && prop.y as i64 == cell_y)
            .map(|(index, prop)| (index, prop.id.clone()))
        {
            state.selected_prop_index = index;
            state.active_selection = WorldPlacementKind::Prop;
            self.editor_selection = EditorSelection::Prop;
            self.status = format!("Selected prop '{id}' at {x},{y}.");
            return true;
        }

        if let Some((index, id)) = state
            .spawns
            .iter()
            .enumerate()
            .rev()
            .find(|(_, spawn)| spawn.x as i64 == cell_x && spawn.y as i64 == cell_y)
            .map(|(index, spawn)| (index, spawn.id.clone()))
        {
            state.selected_spawn_index = index;
            state.active_selection = WorldPlacementKind::Spawn;
            self.editor_selection = EditorSelection::Spawn;
            self.status = format!("Selected spawn '{id}' at {x},{y}.");
            return true;
        }

        if let Some((index, id)) = state
            .triggers
            .iter()
            .enumerate()
            .rev()
            .find(|(_, trigger)| {
                let left = trigger.x as i64;
                let top = trigger.y as i64;
                let right = left + trigger.w.max(1) as i64;
                let bottom = top + trigger.h.max(1) as i64;
                cell_x >= left && cell_y >= top && cell_x < right && cell_y < bottom
            })
            .map(|(index, trigger)| (index, trigger.id.clone()))
        {
            state.selected_trigger_index = index;
            state.active_selection = WorldPlacementKind::Trigger;
            self.editor_selection = EditorSelection::Trigger;
            self.status = format!("Selected trigger '{id}' at {x},{y}.");
            return true;
        }

        false
    }

    fn select_world_placement_in_rect(&mut self, start: (u32, u32), end: (u32, u32)) -> bool {
        let Some(state) = self.world_placements.as_mut() else {
            return false;
        };
        let (left, top, right, bottom) = normalized_map_rect(start, end);

        if let Some((index, id)) = state
            .voxel_objects
            .objects
            .iter()
            .enumerate()
            .rev()
            .find(|(_, object)| {
                let obj_left = object.x.floor() as i32;
                let obj_top = object.y.floor() as i32;
                let obj_right = obj_left + object.footprint_width.max(1.0).ceil() as i32 - 1;
                let obj_bottom = obj_top + object.footprint_height.max(1.0).ceil() as i32 - 1;
                rects_overlap(
                    left, top, right, bottom, obj_left, obj_top, obj_right, obj_bottom,
                )
            })
            .map(|(index, object)| (index, object.id.clone()))
        {
            state.selected_voxel_object_index = index;
            state.active_selection = WorldPlacementKind::VoxelObject;
            self.editor_selection = EditorSelection::VoxelObject;
            self.status = format!("Marquee selected voxel object '{id}'.");
            return true;
        }

        if let Some((index, id)) = state
            .triggers
            .iter()
            .enumerate()
            .rev()
            .find(|(_, trigger)| {
                let trigger_left = trigger.x;
                let trigger_top = trigger.y;
                let trigger_right = trigger_left + trigger.w.max(1) as i32 - 1;
                let trigger_bottom = trigger_top + trigger.h.max(1) as i32 - 1;
                rects_overlap(
                    left,
                    top,
                    right,
                    bottom,
                    trigger_left,
                    trigger_top,
                    trigger_right,
                    trigger_bottom,
                )
            })
            .map(|(index, trigger)| (index, trigger.id.clone()))
        {
            state.selected_trigger_index = index;
            state.active_selection = WorldPlacementKind::Trigger;
            self.editor_selection = EditorSelection::Trigger;
            self.status = format!("Marquee selected trigger '{id}'.");
            return true;
        }

        if let Some((index, id)) = state
            .props
            .iter()
            .enumerate()
            .rev()
            .find(|(_, prop)| {
                prop.x >= left && prop.x <= right && prop.y >= top && prop.y <= bottom
            })
            .map(|(index, prop)| (index, prop.id.clone()))
        {
            state.selected_prop_index = index;
            state.active_selection = WorldPlacementKind::Prop;
            self.editor_selection = EditorSelection::Prop;
            self.status = format!("Marquee selected prop '{id}'.");
            return true;
        }

        if let Some((index, id)) = state
            .spawns
            .iter()
            .enumerate()
            .rev()
            .find(|(_, spawn)| {
                spawn.x >= left && spawn.x <= right && spawn.y >= top && spawn.y <= bottom
            })
            .map(|(index, spawn)| (index, spawn.id.clone()))
        {
            state.selected_spawn_index = index;
            state.active_selection = WorldPlacementKind::Spawn;
            self.editor_selection = EditorSelection::Spawn;
            self.status = format!("Marquee selected spawn '{id}'.");
            return true;
        }

        self.selected_map_cell = Some(start);
        self.editor_selection = EditorSelection::MapCell;
        self.status = format!("No world object inside marquee {left},{top} to {right},{bottom}.");
        false
    }

    fn move_active_world_placement_to_cell(&mut self, x: u32, y: u32) -> bool {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable world placement state is loaded.".to_string();
            return false;
        };
        match state.active_selection {
            WorldPlacementKind::Prop => {
                let Some(prop) = state.props.get_mut(state.selected_prop_index) else {
                    return false;
                };
                let x = x as i32;
                let y = y as i32;
                if prop.x == x && prop.y == y {
                    return false;
                }
                prop.x = x;
                prop.y = y;
                state.props_dirty = true;
                self.status = format!("Moved prop '{}' to {},{}.", prop.id, prop.x, prop.y);
            }
            WorldPlacementKind::Spawn => {
                let Some(spawn) = state.spawns.get_mut(state.selected_spawn_index) else {
                    return false;
                };
                let x = x as i32;
                let y = y as i32;
                if spawn.x == x && spawn.y == y {
                    return false;
                }
                spawn.x = x;
                spawn.y = y;
                state.spawns_dirty = true;
                self.status = format!("Moved spawn '{}' to {},{}.", spawn.id, spawn.x, spawn.y);
            }
            WorldPlacementKind::Trigger => {
                let Some(trigger) = state.triggers.get_mut(state.selected_trigger_index) else {
                    return false;
                };
                let x = x as i32;
                let y = y as i32;
                if trigger.x == x && trigger.y == y {
                    return false;
                }
                trigger.x = x;
                trigger.y = y;
                state.triggers_dirty = true;
                self.status = format!(
                    "Moved trigger '{}' to {},{}.",
                    trigger.id, trigger.x, trigger.y
                );
            }
            WorldPlacementKind::VoxelObject => {
                let Some(object) = state
                    .voxel_objects
                    .objects
                    .get_mut(state.selected_voxel_object_index)
                else {
                    return false;
                };
                let x = x as f32;
                let y = y as f32;
                if (object.x - x).abs() < f32::EPSILON && (object.y - y).abs() < f32::EPSILON {
                    return false;
                }
                object.x = x;
                object.y = y;
                state.voxel_objects_dirty = true;
                self.status = format!(
                    "Moved voxel object '{}' to {:.1},{:.1}.",
                    object.id, object.x, object.y
                );
            }
        }
        true
    }

    fn resize_active_world_placement_to_cell(&mut self, x: u32, y: u32) -> bool {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable world placement state is loaded.".to_string();
            return false;
        };

        match state.active_selection {
            WorldPlacementKind::Trigger => {
                let Some(trigger) = state.triggers.get_mut(state.selected_trigger_index) else {
                    return false;
                };
                let width = (x as i32 - trigger.x + 1).max(1);
                let height = (y as i32 - trigger.y + 1).max(1);
                if trigger.w == width && trigger.h == height {
                    return false;
                }
                trigger.w = width;
                trigger.h = height;
                state.triggers_dirty = true;
                self.status = format!(
                    "Resized trigger '{}' to {}x{}.",
                    trigger.id, trigger.w, trigger.h
                );
            }
            WorldPlacementKind::VoxelObject => {
                let Some(object) = state
                    .voxel_objects
                    .objects
                    .get_mut(state.selected_voxel_object_index)
                else {
                    return false;
                };
                let width = (x as f32 - object.x + 1.0).max(0.25);
                let height = (y as f32 - object.y + 1.0).max(0.25);
                if (object.footprint_width - width).abs() < f32::EPSILON
                    && (object.footprint_height - height).abs() < f32::EPSILON
                {
                    return false;
                }
                object.footprint_width = width;
                object.footprint_height = height;
                state.voxel_objects_dirty = true;
                self.status = format!(
                    "Resized voxel object '{}' footprint to {:.1}x{:.1}.",
                    object.id, object.footprint_width, object.footprint_height
                );
            }
            WorldPlacementKind::Prop | WorldPlacementKind::Spawn => {
                self.status = "Selected placement uses a point marker, not a resizable footprint."
                    .to_string();
                return false;
            }
        }
        true
    }

    fn nudge_active_world_placement(&mut self, dx: i32, dy: i32) -> bool {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable world placement state is loaded.".to_string();
            return false;
        };
        match state.active_selection {
            WorldPlacementKind::Prop => {
                let Some(prop) = state.props.get_mut(state.selected_prop_index) else {
                    self.status = "No selected prop to nudge.".to_string();
                    return false;
                };
                prop.x += dx;
                prop.y += dy;
                state.props_dirty = true;
                self.status = format!("Nudged prop '{}' to {},{}.", prop.id, prop.x, prop.y);
            }
            WorldPlacementKind::Spawn => {
                let Some(spawn) = state.spawns.get_mut(state.selected_spawn_index) else {
                    self.status = "No selected spawn to nudge.".to_string();
                    return false;
                };
                spawn.x += dx;
                spawn.y += dy;
                state.spawns_dirty = true;
                self.status = format!("Nudged spawn '{}' to {},{}.", spawn.id, spawn.x, spawn.y);
            }
            WorldPlacementKind::Trigger => {
                let Some(trigger) = state.triggers.get_mut(state.selected_trigger_index) else {
                    self.status = "No selected trigger to nudge.".to_string();
                    return false;
                };
                trigger.x += dx;
                trigger.y += dy;
                state.triggers_dirty = true;
                self.status = format!(
                    "Nudged trigger '{}' to {},{}.",
                    trigger.id, trigger.x, trigger.y
                );
            }
            WorldPlacementKind::VoxelObject => {
                let Some(object) = state
                    .voxel_objects
                    .objects
                    .get_mut(state.selected_voxel_object_index)
                else {
                    self.status = "No selected voxel object to nudge.".to_string();
                    return false;
                };
                object.x += dx as f32;
                object.y += dy as f32;
                state.voxel_objects_dirty = true;
                self.status = format!(
                    "Nudged voxel object '{}' to {:.1},{:.1}.",
                    object.id, object.x, object.y
                );
            }
        }
        true
    }

    fn duplicate_active_world_placement(&mut self) -> bool {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable world placement state is loaded.".to_string();
            return false;
        };
        let duplicated = match state.active_selection {
            WorldPlacementKind::Prop => state.duplicate_selected_prop(),
            WorldPlacementKind::Spawn => state.duplicate_selected_spawn(),
            WorldPlacementKind::Trigger => state.duplicate_selected_trigger(),
            WorldPlacementKind::VoxelObject => state.duplicate_selected_voxel_object(),
        };
        if duplicated {
            self.status = "Duplicated selected world placement.".to_string();
        } else {
            self.status = "No selected world placement to duplicate.".to_string();
        }
        duplicated
    }

    fn delete_active_world_placement(&mut self) -> bool {
        let Some(state) = self.world_placements.as_mut() else {
            self.status = "No editable world placement state is loaded.".to_string();
            return false;
        };
        let deleted = match state.active_selection {
            WorldPlacementKind::Prop => state.remove_selected_prop(),
            WorldPlacementKind::Spawn => state.remove_selected_spawn(),
            WorldPlacementKind::Trigger => state.remove_selected_trigger(),
            WorldPlacementKind::VoxelObject => state.remove_selected_voxel_object(),
        };
        if deleted {
            self.status = "Deleted selected world placement.".to_string();
        } else {
            self.status = "No selected world placement to delete.".to_string();
        }
        deleted
    }

    fn map_layer_validation_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let Some(state) = &self.editor_map else {
            issues.push(format!(
                "map '{}' has no editable layers.ron loaded",
                self.active_map_id
            ));
            return issues;
        };

        if state.layers.map_id != self.active_map_id {
            issues.push(format!(
                "layers.ron map_id '{}' does not match active map '{}'",
                state.layers.map_id, self.active_map_id
            ));
        }
        if state.layers.tile_width == 0 || state.layers.tile_height == 0 {
            issues.push(format!(
                "layers.ron has invalid tile size {}x{}",
                state.layers.tile_width, state.layers.tile_height
            ));
        }
        if state.layers.layers.is_empty() {
            issues.push("layers.ron contains no layers".to_string());
        }

        let Some(map) = self.registry.maps.get(&self.active_map_id) else {
            issues.push(format!(
                "active map '{}' is missing map.ron metadata",
                self.active_map_id
            ));
            return issues;
        };
        let expected_width = map.metadata.width as usize;
        let expected_height = map.metadata.height as usize;
        let known_tiles = self
            .active_tileset()
            .map(|tileset| {
                tileset
                    .named_tiles
                    .iter()
                    .map(|entry| entry.id.as_str())
                    .collect::<std::collections::HashSet<_>>()
            })
            .unwrap_or_default();

        for layer in &state.layers.layers {
            if layer.rows.len() != expected_height {
                issues.push(format!(
                    "layer '{}' has {} rows but map height is {}",
                    layer.id,
                    layer.rows.len(),
                    expected_height
                ));
            }

            let mut legend_symbols = std::collections::HashSet::new();
            for legend in &layer.legend {
                let chars = legend.symbol.chars().collect::<Vec<_>>();
                if chars.len() != 1 {
                    issues.push(format!(
                        "layer '{}' legend symbol '{}' must be exactly one character",
                        layer.id, legend.symbol
                    ));
                    continue;
                }
                if !legend_symbols.insert(chars[0]) {
                    issues.push(format!(
                        "layer '{}' duplicates legend symbol '{}'",
                        layer.id, chars[0]
                    ));
                }
                if !known_tiles.contains(legend.tile_id.as_str())
                    && !self.registry.terrain_types.contains_key(&legend.tile_id)
                {
                    issues.push(format!(
                        "layer '{}' symbol '{}' references missing tile/terrain '{}'",
                        layer.id, legend.symbol, legend.tile_id
                    ));
                }
            }

            for (row_index, row) in layer.rows.iter().enumerate() {
                let row_width = row.chars().count();
                if row_width != expected_width {
                    issues.push(format!(
                        "layer '{}' row {} width is {} but map width is {}",
                        layer.id, row_index, row_width, expected_width
                    ));
                }
                for (x, symbol) in row.chars().enumerate() {
                    if is_empty_layer_symbol(symbol) {
                        continue;
                    }
                    if !legend_symbols.contains(&symbol) {
                        issues.push(format!(
                            "layer '{}' uses unmapped symbol '{}' at {},{}",
                            layer.id, symbol, x, row_index
                        ));
                    }
                }
            }
        }

        issues
    }

    fn generate_worldgen_bake_preview(&mut self) {
        match self.build_worldgen_bake_preview() {
            Ok(preview) => {
                self.status = format!(
                    "Generated bake preview for {}: {}",
                    preview.target_map_id,
                    preview.report.summary()
                );
                self.log(self.status.clone());
                self.worldgen_bake_preview = Some(preview);
            }
            Err(error) => {
                self.status = format!("WorldGen bake preview failed: {error:#}");
                self.log(self.status.clone());
            }
        }
    }

    fn build_worldgen_bake_preview(&self) -> anyhow::Result<WorldgenBakePreviewState> {
        let draft = self
            .registry
            .generated_scene_drafts
            .values()
            .find(|draft| draft.scene_id == self.active_map_id)
            .or_else(|| self.registry.generated_scene_drafts.values().next())
            .context("no generated scene draft is loaded in the content registry")?;
        let contract = self
            .registry
            .scene_bake_contracts
            .get(&draft.bake_contract_id)
            .or_else(|| self.registry.scene_bake_contracts.values().next())
            .context("no scene bake contract is loaded in the content registry")?;
        let (target_width, target_height) = self
            .editor_map
            .as_ref()
            .map(|state| layer_dimensions(&state.layers))
            .or_else(|| {
                self.registry
                    .maps
                    .get(&self.active_map_id)
                    .map(|bundle| (bundle.metadata.width, bundle.metadata.height))
            })
            .unwrap_or((draft.width, draft.height));

        let request = SceneGenRequest {
            scene_id: draft.scene_id.clone(),
            kind: worldgen_scene_kind_from_shared(draft.scene_kind),
            seed: draft.seed,
            width: target_width.max(1),
            height: target_height.max(1),
            tile_size: draft.tile_size,
            template_id: draft.template_id.clone(),
            allow_coast: true,
            allow_forest: true,
            allow_rocks: true,
            allow_farmable_clearings: true,
        };
        let scene = game_worldgen::generate_scene(&request)?;
        let target_layer_id =
            select_bake_target_layer(self.editor_map.as_ref(), &contract.generated_layers)
                .unwrap_or_else(|| "ground".to_string());
        let target_object_layer_id = select_object_layer(self.editor_map.as_ref());
        let report = self.preview_worldgen_bake_report(
            &scene,
            &target_layer_id,
            target_object_layer_id.as_deref(),
            &contract.protected_layers,
            draft.width,
            draft.height,
        );

        Ok(WorldgenBakePreviewState {
            scene,
            draft_id: draft.id.clone(),
            contract_id: contract.id.clone(),
            target_map_id: self.active_map_id.clone(),
            target_layer_id,
            target_object_layer_id,
            report,
        })
    }

    fn preview_worldgen_bake_report(
        &self,
        scene: &GeneratedScene,
        target_layer_id: &str,
        target_object_layer_id: Option<&str>,
        protected_layers: &[String],
        draft_width: u32,
        draft_height: u32,
    ) -> WorldgenBakeReport {
        let mut report = WorldgenBakeReport::default();
        let validation = scene.validate();
        report.warnings.extend(validation.warnings);
        report.terrain_counts = validation.semantic_counts;
        report.object_cells = scene.object_spawns.len();

        let Some(state) = self.editor_map.as_ref() else {
            report
                .warnings
                .push("No editable layers.ron is loaded; commit is disabled.".to_string());
            return report;
        };

        if draft_width != scene.width || draft_height != scene.height {
            report.warnings.push(format!(
                "Draft contract is {}x{}, but target editable layer stack is {}x{}; preview uses target dimensions.",
                draft_width, draft_height, scene.width, scene.height
            ));
        }

        let Some(layer) = state
            .layers
            .layers
            .iter()
            .find(|layer| layer.id == target_layer_id)
        else {
            report
                .warnings
                .push(format!("Target layer '{}' is missing.", target_layer_id));
            return report;
        };

        for y in 0..scene.height as usize {
            for x in 0..scene.width as usize {
                let Some(terrain) = scene.terrain_at(x as u32, y as u32) else {
                    continue;
                };
                let tile_id = terrain_to_ground_tile_id(terrain);
                let target_symbol = layer_symbol_for_tile(layer, tile_id).unwrap_or('?');
                let current_symbol = layer_symbol_at(layer, x, y).unwrap_or('.');
                if current_symbol == target_symbol {
                    report.unchanged_cells += 1;
                } else {
                    report.changed_cells += 1;
                }
            }
        }

        if let Some(object_layer_id) = target_object_layer_id {
            if state
                .layers
                .layers
                .iter()
                .all(|layer| layer.id != object_layer_id)
            {
                report.warnings.push(format!(
                    "Object bake layer '{}' is missing; natural object markers will be skipped.",
                    object_layer_id
                ));
            }
        }

        if protected_layers
            .iter()
            .any(|protected| protected == target_layer_id)
        {
            report.skipped_protected_cells =
                (scene.width as usize).saturating_mul(scene.height as usize);
            report.warnings.push(format!(
                "Target terrain layer '{}' is protected by the bake contract; commit will refuse to write it.",
                target_layer_id
            ));
        } else {
            for layer in &state.layers.layers {
                if layer.id != target_layer_id
                    && protected_layers
                        .iter()
                        .any(|protected| protected == &layer.id)
                {
                    report.skipped_protected_cells += layer
                        .rows
                        .iter()
                        .map(|row| row.chars().count())
                        .sum::<usize>();
                }
            }
        }

        report
    }

    fn commit_worldgen_bake_preview(&mut self) {
        match self.apply_worldgen_bake_preview() {
            Ok(report) => {
                self.status = format!("WorldGen bake committed: {}", report.summary());
                self.log(self.status.clone());
                self.reload_content();
            }
            Err(error) => {
                self.status = format!("WorldGen bake commit failed: {error:#}");
                self.log(self.status.clone());
            }
        }
    }

    fn apply_worldgen_bake_preview(&mut self) -> anyhow::Result<WorldgenBakeReport> {
        if self.worldgen_bake_preview.is_none() {
            self.generate_worldgen_bake_preview();
        }
        let preview = self
            .worldgen_bake_preview
            .clone()
            .context("no generated bake preview is available")?;
        anyhow::ensure!(
            preview.target_map_id == self.active_map_id,
            "preview target map does not match active map"
        );
        if let Some(contract) = self.registry.scene_bake_contracts.get(&preview.contract_id) {
            anyhow::ensure!(
                !contract
                    .protected_layers
                    .iter()
                    .any(|layer| layer == &preview.target_layer_id),
                "refusing to commit generated terrain into protected layer '{}'",
                preview.target_layer_id
            );
        }

        let layers_path = map_layers_path(&self.project_root, &self.active_map_id);
        let mut layers = game_data::loader::load_map_layers(&layers_path)?;
        let ground_index = layers
            .layers
            .iter()
            .position(|layer| layer.id == preview.target_layer_id)
            .or_else(|| layers.layers.iter().position(|layer| layer.id == "ground"))
            .context("no generated terrain target layer exists")?;
        normalize_layer_rows(
            &mut layers.layers[ground_index],
            preview.scene.width as usize,
            preview.scene.height as usize,
        );

        let mut report = preview.report.clone();
        report.changed_cells = 0;
        report.unchanged_cells = 0;
        {
            let ground_layer = &mut layers.layers[ground_index];
            for y in 0..preview.scene.height as usize {
                for x in 0..preview.scene.width as usize {
                    let Some(terrain) = preview.scene.terrain_at(x as u32, y as u32) else {
                        continue;
                    };
                    let tile_id = terrain_to_ground_tile_id(terrain);
                    let symbol = ensure_layer_symbol_for_tile(ground_layer, tile_id)?;
                    if set_layer_symbol_at(ground_layer, x, y, symbol) {
                        report.changed_cells += 1;
                    } else {
                        report.unchanged_cells += 1;
                    }
                }
            }
        }

        if let Some(object_layer_id) = preview.target_object_layer_id.as_deref() {
            if let Some(object_index) = layers
                .layers
                .iter()
                .position(|layer| layer.id == object_layer_id)
            {
                let object_layer = &mut layers.layers[object_index];
                normalize_layer_rows(
                    object_layer,
                    preview.scene.width as usize,
                    preview.scene.height as usize,
                );
                for spawn in &preview.scene.object_spawns {
                    let tile_id = object_spawn_to_tile_id(&spawn.object_id);
                    let symbol = ensure_layer_symbol_for_tile(object_layer, tile_id)?;
                    let x = spawn.x as usize;
                    let y = spawn.y as usize;
                    if layer_symbol_at(object_layer, x, y)
                        .map(is_empty_layer_symbol)
                        .unwrap_or(false)
                    {
                        set_layer_symbol_at(object_layer, x, y, symbol);
                    }
                }
            }
        }

        let backup = game_data::loader::save_map_layers_with_phase_backup(
            &layers_path,
            &layers,
            "phase52g",
        )?;
        report.backup_path = backup;
        report.committed = true;
        self.worldgen_bake_preview = Some(WorldgenBakePreviewState {
            report: report.clone(),
            ..preview
        });
        Ok(report)
    }
    fn reload_content(&mut self) {
        self.request_content_reload("manual reload");
    }

    fn request_content_reload(&mut self, label: impl Into<String>) {
        let label = label.into();
        if self.content_reload_job.is_some() {
            self.status =
                "Reload already in progress. The editor remains usable while it finishes."
                    .to_string();
            return;
        }

        let selected_layer_id = self
            .editor_map
            .as_ref()
            .map(|state| state.selected_layer_id());
        let selected_symbol = self.editor_map.as_ref().map(|state| state.selected_symbol);
        let project_root = self.project_root.clone();
        let active_map_id = self.active_map_id.clone();
        let thread_label = label.clone();

        self.status = format!("Reload queued: {label}.");
        self.log(format!("Reload started in background: {label}."));
        self.content_reload_job = Some(ContentReloadJob {
            label,
            started_at: Instant::now(),
            handle: thread::spawn(move || {
                StarlightRidgeEguiEditor::load_content_reload_payload(
                    project_root,
                    active_map_id,
                    selected_layer_id,
                    selected_symbol,
                )
                .with_context(|| format!("background reload failed for {thread_label}"))
            }),
        });
    }

    fn poll_content_reload_job(&mut self, ctx: &egui::Context) {
        let Some(job) = self.content_reload_job.as_ref() else {
            return;
        };

        if !job.handle.is_finished() {
            let elapsed = job.started_at.elapsed().as_secs_f32();
            if elapsed > 0.25 {
                self.status = format!("Reloading content in background... {elapsed:.1}s");
            }
            ctx.request_repaint_after(Duration::from_millis(100));
            return;
        }

        let job = self.content_reload_job.take().expect("reload job exists");
        let label = job.label;
        let started_at = job.started_at;
        match job.handle.join() {
            Ok(Ok(payload)) => {
                self.apply_content_reload_payload(payload, &label, started_at);
            }
            Ok(Err(error)) => {
                self.status = "Reload failed while rebuilding editor content.".to_string();
                self.log(format!("Reload error: {error:#}"));
            }
            Err(_) => {
                self.status = "Reload worker panicked; editor shell stayed alive.".to_string();
                self.log("Reload worker panicked before returning content.".to_string());
            }
        }
    }

    fn load_content_reload_payload(
        project_root: PathBuf,
        active_map_id: String,
        preferred_layer_id: Option<String>,
        preferred_symbol: Option<char>,
    ) -> anyhow::Result<ContentReloadPayload> {
        let (vox_assets, vox_scan_error) = match scan_vox_files(&project_root) {
            Ok(assets) => (assets, None),
            Err(error) => (Vec::new(), Some(format!("{error:#}"))),
        };

        let registry =
            game_data::load_registry(&project_root).context("failed to reload content registry")?;

        let mut editor_map = match EditorMapState::load(&project_root, &active_map_id) {
            Ok(state) => Some(state),
            Err(_) => None,
        };
        if let Some(state) = editor_map.as_mut() {
            if let Some(layer_id) = preferred_layer_id.as_deref() {
                state.select_layer_by_id(layer_id);
            }
            if let Some(symbol) = preferred_symbol {
                state.selected_symbol = symbol;
            }
        }

        let world_placements = WorldPlacementState::load(&project_root, &active_map_id).ok();
        let tile_map = build_tile_map_render_data(&project_root, &registry, &active_map_id)
            .context("failed to rebuild egui tile preview")?;

        Ok(ContentReloadPayload {
            active_map_id,
            registry,
            tile_map,
            editor_map,
            world_placements,
            vox_assets,
            vox_scan_error,
        })
    }

    fn apply_content_reload_payload(
        &mut self,
        payload: ContentReloadPayload,
        label: &str,
        started_at: Instant,
    ) {
        if payload.active_map_id != self.active_map_id {
            self.status = format!(
                "Discarded stale reload for map {}; active map is {}.",
                payload.active_map_id, self.active_map_id
            );
            self.log(self.status.clone());
            return;
        }

        self.registry = payload.registry;
        self.tile_map = payload.tile_map;
        self.editor_map = payload.editor_map;
        self.world_placements = payload.world_placements;
        self.vox_assets = payload.vox_assets;
        if self.selected_vox_index >= self.vox_assets.len() {
            self.selected_vox_index = self.vox_assets.len().saturating_sub(1);
        }
        self.sync_selected_symbol_to_tile();

        if let Some(error) = payload.vox_scan_error {
            self.log(format!("VOX scan warning during reload: {error}"));
        }
        let elapsed = started_at.elapsed().as_secs_f32();
        self.status = format!("Reloaded content in background ({label}, {elapsed:.2}s).");
        self.log(self.status.clone());
    }

    fn switch_map(&mut self, map_id: String) {
        if self.active_map_id == map_id {
            return;
        }

        self.active_map_id = map_id;
        self.selected_map_cell = None;
        self.world_marquee_start = None;
        self.world_marquee_end = None;
        self.world_resize_active = false;
        self.load_editor_map_state(None, None);
        self.load_world_placement_state();
        match build_tile_map_render_data(&self.project_root, &self.registry, &self.active_map_id) {
            Ok(tile_map) => {
                self.tile_map = tile_map;
                let _ = write_editor_live_preview_manifest(&self.project_root, &self.active_map_id);
                self.pixel_editor = PixelEditorState::load_for_active_tileset(
                    &self.project_root,
                    &self.registry,
                    &self.active_map_id,
                );
                self.status = format!("Switched map to {}.", self.active_map_id);
                self.log(self.status.clone());
            }
            Err(error) => {
                self.status = format!("Map switch failed for {}.", self.active_map_id);
                self.log(format!("Map switch error: {error:#}"));
            }
        }
    }

    fn select_tile(&mut self, tile_id: String, source: &str) {
        self.selected_tile_id = tile_id;
        self.selected_cell = atlas_cell_for_tile(&self.registry, &self.selected_tile_id)
            .unwrap_or(self.selected_cell);
        self.role_state = load_tile_role_state(&self.project_root, &self.selected_tile_id);
        self.sync_selected_symbol_to_tile();
        self.editor_selection = EditorSelection::Tile;
        self.status = format!(
            "{source} selected tile {} at atlas {},{}.",
            self.selected_tile_id, self.selected_cell.0, self.selected_cell.1
        );
        self.log(self.status.clone());
    }

    fn select_next_tile(&mut self, step: isize) {
        let Some(tileset) = self.active_tileset() else {
            self.status = "No active tileset available.".to_string();
            return;
        };
        if tileset.named_tiles.is_empty() {
            self.status = "Active tileset has no named tiles.".to_string();
            return;
        }
        let current = tileset
            .named_tiles
            .iter()
            .position(|tile| tile.id == self.selected_tile_id)
            .unwrap_or(0) as isize;
        let len = tileset.named_tiles.len() as isize;
        let next = (current + step).rem_euclid(len) as usize;
        let tile_id = tileset.named_tiles[next].id.clone();
        self.select_tile(tile_id, "Keyboard/preview");
    }

    fn save_role_state(&mut self) {
        match save_tile_role_state(&self.project_root, &self.selected_tile_id, &self.role_state) {
            Ok(()) => {
                self.status = format!("Saved role metadata for {}.", self.selected_tile_id);
                self.log(self.status.clone());
                self.reload_content();
            }
            Err(error) => {
                self.status = "Failed to save selected tile metadata.".to_string();
                self.log(format!("Role metadata save error: {error:#}"));
            }
        }
    }

    fn write_selection_manifest(&mut self) {
        let path = self
            .project_root
            .join("artifacts")
            .join("egui_asset_studio_selection.ron");
        let body = format!(
            "(\n    selected_asset_index: {},\n    selected_tile: \"{}\",\n    selected_cell: ({}, {}),\n    role: \"{}\",\n    collision: \"{}\",\n    active_map: \"{}\",\n    selected_map_cell: {:?},\n    note: \"Phase 36 egui editor checkpoint\",\n)\n",
            self.selected_asset_index,
            self.selected_tile_id,
            self.selected_cell.0,
            self.selected_cell.1,
            self.role_state.role,
            self.role_state.collision,
            self.active_map_id,
            self.selected_map_cell,
        );

        let result = path
            .parent()
            .map(std::fs::create_dir_all)
            .transpose()
            .and_then(|_| std::fs::write(&path, body));

        match result {
            Ok(()) => {
                self.status = format!("Wrote egui editor checkpoint: {}", path.display());
                self.log(self.status.clone());
            }
            Err(error) => {
                self.status = "Failed to write egui editor checkpoint.".to_string();
                self.log(format!("Checkpoint write error: {error:#}"));
            }
        }
    }

    fn open_web_asset_lab(&mut self) {
        let script = self.project_root.join("RUN_ASSET_LAB.bat");
        let result = if cfg!(target_os = "windows") && script.exists() {
            Command::new("cmd").arg("/C").arg(script).spawn()
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "RUN_ASSET_LAB.bat is only launched automatically on Windows",
            ))
        };

        match result {
            Ok(_) => {
                self.status = "Launched web Asset Lab helper.".to_string();
                self.log(self.status.clone());
            }
            Err(error) => {
                self.status = "Could not launch web Asset Lab helper from egui editor.".to_string();
                self.log(format!("Asset Lab launch note: {error}"));
            }
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            if input.key_pressed(egui::Key::F5) {
                self.reload_content();
            }
            let ctrl = input.modifiers.ctrl;
            if ctrl {
                if input.key_pressed(egui::Key::Z) {
                    if self.workspace_tab == WorkspaceTab::World && self.undo_map_layer_edit() {
                        return;
                    }
                    if let Some(label) = self.pixel_editor.undo() {
                        self.status = format!("Undid {label}.");
                    }
                    return;
                }
                if input.key_pressed(egui::Key::Y) {
                    if self.workspace_tab == WorkspaceTab::World && self.redo_map_layer_edit() {
                        return;
                    }
                    if self.pixel_editor.redo().is_some() {
                        self.status = "Redid pixel edit.".to_string();
                    }
                    return;
                }
                if input.key_pressed(egui::Key::C) {
                    if self.pixel_editor.copy_selection() {
                        self.pixel_editor.tool = PixelTool::Paste;
                        self.status = "Copied pixel selection.".to_string();
                    } else {
                        let tile_size = self.active_tile_size();
                        if self.pixel_editor.copy_tile(self.selected_cell, tile_size) {
                            self.pixel_editor.tool = PixelTool::Paste;
                            self.status = format!(
                                "Copied selected atlas tile {},{}.",
                                self.selected_cell.0, self.selected_cell.1
                            );
                        }
                    }
                    return;
                }
                if input.key_pressed(egui::Key::V) {
                    self.pixel_editor.tool = PixelTool::Paste;
                    self.asset_subtab = AssetSubTab::PixelEditor;
                    self.workspace_tab = WorkspaceTab::Assets;
                    self.status = "Paste preview armed for pixel editor.".to_string();
                    return;
                }
                if input.key_pressed(egui::Key::S) {
                    self.save_context();
                    return;
                }
                if input.key_pressed(egui::Key::D)
                    && self.workspace_tab == WorkspaceTab::World
                    && self.world_subtab == WorldSubTab::MapPaint
                    && self.selected_tool == 0
                {
                    self.duplicate_active_world_placement();
                    return;
                }
            }
            if self.workspace_tab == WorkspaceTab::World
                && self.world_subtab == WorldSubTab::MapPaint
                && self.selected_tool == 0
            {
                if input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace) {
                    self.delete_active_world_placement();
                    return;
                }
                if input.key_pressed(egui::Key::ArrowRight) {
                    self.nudge_active_world_placement(1, 0);
                    return;
                }
                if input.key_pressed(egui::Key::ArrowLeft) {
                    self.nudge_active_world_placement(-1, 0);
                    return;
                }
                if input.key_pressed(egui::Key::ArrowDown) {
                    self.nudge_active_world_placement(0, 1);
                    return;
                }
                if input.key_pressed(egui::Key::ArrowUp) {
                    self.nudge_active_world_placement(0, -1);
                    return;
                }
            }
            if input.key_pressed(egui::Key::V) {
                self.selected_tool = 0;
            }
            if input.key_pressed(egui::Key::B) {
                self.selected_tool = 2;
            }
            if input.key_pressed(egui::Key::E) {
                self.selected_tool = 3;
            }
            if input.key_pressed(egui::Key::G) {
                self.selected_tool = 4;
            }
            if input.key_pressed(egui::Key::I) {
                self.selected_tool = 5;
            }
            if input.key_pressed(egui::Key::T) {
                self.selected_tool = 6;
            }
            if input.key_pressed(egui::Key::C) {
                self.selected_tool = 7;
            }
            if input.key_pressed(egui::Key::A) {
                self.selected_tool = 8;
            }
            if input.key_pressed(egui::Key::P) {
                self.selected_tool = 9;
            }
            if input.key_pressed(egui::Key::ArrowRight) {
                self.select_next_tile(1);
            }
            if input.key_pressed(egui::Key::ArrowLeft) {
                self.select_next_tile(-1);
            }
        });
    }

    fn draw_top_bar(&mut self, root_ui: &mut egui::Ui) {
        egui::Panel::top("editor_top_bar").show_inside(root_ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Starlight Ridge Editor");
                ui.separator();
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Project, "Project");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::World, "World");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Assets, "Assets");
                ui.selectable_value(
                    &mut self.workspace_tab,
                    WorkspaceTab::Animation,
                    "Animation",
                );
                ui.selectable_value(
                    &mut self.workspace_tab,
                    WorkspaceTab::Character,
                    "Character",
                );
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Logic, "Logic");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Data, "Data");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Playtest, "Playtest");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Settings, "Settings");
            });

            ui.separator();
            self.draw_workspace_tab_strip(ui);

            ui.separator();
            self.draw_context_command_strip(ui);
            self.draw_workspace_tool_strip(ui);
        });
    }

    fn draw_workspace_tab_strip(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| match self.workspace_tab {
            WorkspaceTab::Project => {
                ui.label("Project:");
                ui.selectable_value(
                    &mut self.project_subtab,
                    ProjectSubTab::Overview,
                    "Overview",
                );
                ui.selectable_value(
                    &mut self.project_subtab,
                    ProjectSubTab::Validation,
                    "Validation",
                );
                ui.selectable_value(&mut self.project_subtab, ProjectSubTab::Build, "Build");
                ui.selectable_value(&mut self.project_subtab, ProjectSubTab::Export, "Export");
                ui.selectable_value(
                    &mut self.project_subtab,
                    ProjectSubTab::Diagnostics,
                    "Diagnostics",
                );
            }
            WorkspaceTab::World => {
                ui.label("World:");
                ui.selectable_value(&mut self.world_subtab, WorldSubTab::MapPaint, "Map Paint");
                ui.selectable_value(&mut self.world_subtab, WorldSubTab::Layers, "Layers");
                ui.selectable_value(&mut self.world_subtab, WorldSubTab::Objects, "Objects");
                ui.selectable_value(
                    &mut self.world_subtab,
                    WorldSubTab::TerrainRules,
                    "Terrain Rules",
                );
            }
            WorkspaceTab::Assets => {
                ui.label("Assets:");
                ui.selectable_value(
                    &mut self.asset_subtab,
                    AssetSubTab::TerrainAtlas,
                    "Terrain Atlas",
                );
                ui.selectable_value(
                    &mut self.asset_subtab,
                    AssetSubTab::AtlasCompare,
                    "Atlas Import",
                );
                ui.selectable_value(
                    &mut self.asset_subtab,
                    AssetSubTab::PixelEditor,
                    "Pixel Editor",
                );
                ui.selectable_value(
                    &mut self.asset_subtab,
                    AssetSubTab::VoxelPanels,
                    "Voxel Panels",
                );
                ui.selectable_value(&mut self.asset_subtab, AssetSubTab::Voxels, "VOX Models");
                ui.selectable_value(
                    &mut self.asset_subtab,
                    AssetSubTab::VoxelGenerator,
                    "Voxel Generator",
                );
                ui.selectable_value(&mut self.asset_subtab, AssetSubTab::Props, "Props");
                ui.selectable_value(&mut self.asset_subtab, AssetSubTab::Seasons, "Seasons");
            }
            WorkspaceTab::Animation => {
                ui.label("Animation:");
                ui.selectable_value(&mut self.animation_subtab, AnimationSubTab::Clips, "Clips");
                ui.selectable_value(
                    &mut self.animation_subtab,
                    AnimationSubTab::Timeline,
                    "Timeline",
                );
                ui.selectable_value(
                    &mut self.animation_subtab,
                    AnimationSubTab::Events,
                    "Events",
                );
                ui.selectable_value(
                    &mut self.animation_subtab,
                    AnimationSubTab::Sockets,
                    "Sockets",
                );
                ui.selectable_value(
                    &mut self.animation_subtab,
                    AnimationSubTab::Hitboxes,
                    "Hitboxes",
                );
                ui.selectable_value(
                    &mut self.animation_subtab,
                    AnimationSubTab::SeasonalVariants,
                    "Seasonal",
                );
            }
            WorkspaceTab::Character => {
                ui.label("Character:");
                ui.selectable_value(&mut self.character_subtab, CharacterSubTab::Bases, "Bases");
                ui.selectable_value(
                    &mut self.character_subtab,
                    CharacterSubTab::Outfits,
                    "Outfits",
                );
                ui.selectable_value(&mut self.character_subtab, CharacterSubTab::Tools, "Tools");
                ui.selectable_value(
                    &mut self.character_subtab,
                    CharacterSubTab::DirectionSets,
                    "Directions",
                );
                ui.selectable_value(
                    &mut self.character_subtab,
                    CharacterSubTab::Preview,
                    "Preview",
                );
            }
            WorkspaceTab::Logic => {
                ui.label("Logic:");
                ui.selectable_value(&mut self.logic_subtab, LogicSubTab::Graphs, "Graphs");
                ui.selectable_value(
                    &mut self.logic_subtab,
                    LogicSubTab::EventBindings,
                    "Event Bindings",
                );
                ui.selectable_value(&mut self.logic_subtab, LogicSubTab::Tools, "Tools");
                ui.selectable_value(
                    &mut self.logic_subtab,
                    LogicSubTab::Blocks,
                    "Blocks / Tiles",
                );
                ui.selectable_value(
                    &mut self.logic_subtab,
                    LogicSubTab::Validation,
                    "Validation",
                );
            }
            WorkspaceTab::Data => {
                ui.label("Data:");
                ui.selectable_value(&mut self.data_subtab, DataSubTab::Items, "Items");
                ui.selectable_value(&mut self.data_subtab, DataSubTab::Crops, "Crops");
                ui.selectable_value(&mut self.data_subtab, DataSubTab::Npcs, "NPCs");
                ui.selectable_value(&mut self.data_subtab, DataSubTab::Dialogue, "Dialogue");
                ui.selectable_value(&mut self.data_subtab, DataSubTab::Quests, "Quests");
                ui.selectable_value(&mut self.data_subtab, DataSubTab::Shops, "Shops");
                ui.selectable_value(&mut self.data_subtab, DataSubTab::Schedules, "Schedules");
            }
            WorkspaceTab::Playtest => {
                ui.label("Playtest:");
                ui.selectable_value(&mut self.playtest_subtab, PlaytestSubTab::Launch, "Launch");
                ui.selectable_value(
                    &mut self.playtest_subtab,
                    PlaytestSubTab::Runtime,
                    "Runtime",
                );
                ui.selectable_value(&mut self.playtest_subtab, PlaytestSubTab::Logs, "Logs");
            }
            WorkspaceTab::Settings => {
                ui.label("Settings:");
                ui.selectable_value(
                    &mut self.settings_subtab,
                    SettingsSubTab::Preferences,
                    "Preferences",
                );
                ui.selectable_value(
                    &mut self.settings_subtab,
                    SettingsSubTab::Keybinds,
                    "Keybinds",
                );
                ui.selectable_value(&mut self.settings_subtab, SettingsSubTab::Paths, "Paths");
                ui.selectable_value(
                    &mut self.settings_subtab,
                    SettingsSubTab::WebCompanion,
                    "Web Companion",
                );
            }
        });
    }

    fn draw_context_command_strip(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Commands:");
            if ui.button("Save Ctrl+S").clicked() {
                self.save_context();
            }
            if ui.button("Save All Dirty").clicked() {
                self.save_all_dirty();
            }
            if ui
                .add_enabled(
                    self.content_reload_job.is_none(),
                    egui::Button::new("Reload F5"),
                )
                .clicked()
            {
                self.reload_content();
            }
            if ui
                .add_enabled(self.context_can_undo(), egui::Button::new("Undo Ctrl+Z"))
                .clicked()
            {
                self.undo_context();
            }
            if ui
                .add_enabled(self.context_can_redo(), egui::Button::new("Redo Ctrl+Y"))
                .clicked()
            {
                self.redo_context();
            }
            if ui.button("Validate").clicked() {
                self.bottom_tab = BottomTab::Validation;
                self.status = "Focused validation results.".to_string();
            }
            if ui.button("Checkpoint").clicked() {
                self.write_selection_manifest();
            }
        });
    }

    fn draw_workspace_tool_strip(&mut self, ui: &mut egui::Ui) {
        match self.workspace_tab {
            WorkspaceTab::World => {
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    ui.label("World tools:");
                    for (index, label) in TOOL_NAMES.iter().enumerate().take(8) {
                        if ui
                            .selectable_label(self.selected_tool == index, *label)
                            .on_hover_text(format!("{} tool", label))
                            .clicked()
                        {
                            self.selected_tool = index;
                            self.status = format!("Active world tool: {label}");
                        }
                    }
                });
            }
            _ => {}
        }
    }
    fn draw_left_panel(&mut self, root_ui: &mut egui::Ui) {
        egui::Panel::left("editor_left_panel")
            .resizable(true)
            .default_size(240.0)
            .min_size(190.0)
            .max_size(380.0)
            .show_inside(root_ui, |ui| {
                ui.heading("Project");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.left_tab, LeftTab::Project, "Project");
                    ui.selectable_value(&mut self.left_tab, LeftTab::Textures, "Textures");
                    ui.selectable_value(&mut self.left_tab, LeftTab::Maps, "Maps");
                });
                ui.separator();

                match self.left_tab {
                    LeftTab::Project => self.draw_project_tab(ui),
                    LeftTab::Textures => self.draw_textures_tab(ui),
                    LeftTab::Maps => self.draw_maps_tab(ui),
                }
            });
    }

    fn draw_project_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Root: {}", self.project_root.display()));
        ui.label(format!("Items: {}", self.registry.items.len()));
        ui.label(format!("Maps: {}", self.registry.maps.len()));
        ui.label(format!("Tilesets: {}", self.registry.tilesets.len()));
        ui.label(format!(
            "Sprite sheets: {}",
            self.registry.sprite_sheets.len()
        ));
        ui.label(format!(
            "Terrain rulesets: {}",
            self.registry.terrain_rulesets.len()
        ));
        ui.separator();

        ui.heading("Main assets");
        let assets = [
            ("Terrain Atlas", "Texture atlas and tile roles"),
            ("Player Walk", "Character animation sheet"),
            ("Ocean Props", "Static props and objects"),
            ("Animation Timeline", "Events, sockets, hitboxes"),
        ];

        for (index, (name, description)) in assets.iter().enumerate() {
            let selected = self.selected_asset_index == index;
            if ui.selectable_label(selected, *name).clicked() {
                self.selected_asset_index = index;
                self.status = format!("Selected asset workflow: {name}");
            }
            ui.small(*description);
        }
    }

    fn draw_textures_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Tile filter");
            ui.text_edit_singleline(&mut self.tile_filter);
        });
        ui.small("Native egui Asset Lab index. Web template stamping is intentionally removed from the editor path.");
        ui.separator();

        let Some(tileset) = self.active_tileset() else {
            ui.label("No active tileset loaded.");
            return;
        };

        ui.label(format!(
            "{} — {}x{} tiles, {} columns, {} rows",
            tileset.display_name,
            tileset.tile_width,
            tileset.tile_height,
            tileset.columns,
            tileset.rows
        ));

        let filter = self.tile_filter.to_lowercase();
        let tiles = tileset
            .named_tiles
            .iter()
            .filter(|tile| filter.is_empty() || tile.id.to_lowercase().contains(&filter))
            .map(|tile| (tile.id.clone(), tile.x, tile.y))
            .collect::<Vec<_>>();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (tile_id, x, y) in tiles {
                let selected = self.selected_tile_id == tile_id;
                if ui
                    .selectable_label(selected, format!("{tile_id}  [{x},{y}]"))
                    .clicked()
                {
                    self.select_tile(tile_id, "Tileset list");
                }
            }
        });
    }

    fn draw_maps_tab(&mut self, ui: &mut egui::Ui) {
        let mut maps = self.registry.maps.keys().cloned().collect::<Vec<_>>();
        maps.sort();

        ui.label("Active map");
        let mut next_map = self.active_map_id.clone();
        egui::ComboBox::from_id_salt("active_map_selector")
            .selected_text(&next_map)
            .show_ui(ui, |ui| {
                for map_id in &maps {
                    ui.selectable_value(&mut next_map, map_id.clone(), map_id);
                }
            });

        if next_map != self.active_map_id {
            self.switch_map(next_map);
        }

        ui.separator();
        if let Some(map) = self.registry.maps.get(&self.active_map_id) {
            ui.label(format!(
                "Size: {} x {}",
                map.metadata.width, map.metadata.height
            ));
            ui.label(format!("Tileset: {}", map.metadata.tileset));
            ui.label(format!("Props: {}", map.props.len()));
            ui.label(format!("Spawns: {}", map.spawns.len()));
            ui.label(format!("Triggers: {}", map.triggers.len()));
        }

        ui.separator();
        if let Some(state) = &self.editor_map {
            ui.label(format!("Editable map state: {}", state.map_id));
            ui.label(format!("Layers file: {}", state.layers_path.display()));
            ui.label(format!("Editable layers: {}", state.layers.layers.len()));
            ui.label(format!("Selected layer: {}", state.selected_layer_id()));
            ui.label(format!("Selected symbol: '{}'", state.selected_symbol));
            ui.label(if state.dirty {
                "Map edits: dirty"
            } else {
                "Map edits: clean"
            });
        } else {
            ui.label("No editable layers.ron loaded for this map.");
        }

        ui.small("Use the command strip for save, reload, undo, redo, and validation.");
        ui.separator();
        ui.checkbox(&mut self.show_grid, "Show map grid");
        ui.checkbox(&mut self.show_transitions, "Show transition overlays");
        ui.add(egui::Slider::new(&mut self.preview_zoom, 0.5..=3.0).text("Preview zoom"));
        ui.add(egui::Slider::new(&mut self.map_brush_size, 1..=9).text("Map brush"));
    }

    fn draw_right_panel(&mut self, root_ui: &mut egui::Ui) {
        egui::Panel::right("editor_right_panel")
            .resizable(true)
            .default_size(280.0)
            .min_size(220.0)
            .max_size(420.0)
            .show_inside(root_ui, |ui| {
                ui.heading("Inspector");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.right_tab, RightTab::Tile, "Selection");
                    ui.selectable_value(&mut self.right_tab, RightTab::Seams, "Seams");
                    ui.selectable_value(&mut self.right_tab, RightTab::Export, "Export");
                });
                ui.separator();

                match self.right_tab {
                    RightTab::Tile => self.draw_tile_inspector(ui),
                    RightTab::Seams => self.draw_seam_inspector(ui),
                    RightTab::Export => self.draw_export_inspector(ui),
                }
            });
    }

    fn draw_tile_inspector(&mut self, ui: &mut egui::Ui) {
        match self.editor_selection {
            EditorSelection::Prop => {
                ui.heading("Selected prop");
                let Some(state) = self.world_placements.as_mut() else {
                    ui.label("No prop placement state loaded.");
                    return;
                };
                if let Some(prop) = state.props.get_mut(state.selected_prop_index) {
                    if ui_text_row(ui, "ID", &mut prop.id) | ui_text_row(ui, "Kind", &mut prop.kind)
                    {
                        state.props_dirty = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Position");
                        if ui.add(egui::DragValue::new(&mut prop.x)).changed()
                            | ui.add(egui::DragValue::new(&mut prop.y)).changed()
                        {
                            state.props_dirty = true;
                        }
                    });
                    ui.label(if state.props_dirty { "Dirty" } else { "Clean" });
                } else {
                    ui.label("No prop selected.");
                }
                return;
            }
            EditorSelection::Spawn => {
                ui.heading("Selected spawn");
                let Some(state) = self.world_placements.as_mut() else {
                    ui.label("No spawn state loaded.");
                    return;
                };
                if let Some(spawn) = state.spawns.get_mut(state.selected_spawn_index) {
                    if ui_text_row(ui, "ID", &mut spawn.id)
                        | ui_text_row(ui, "Kind", &mut spawn.kind)
                    {
                        state.spawns_dirty = true;
                    }
                    let mut ref_id = spawn.ref_id.clone().unwrap_or_default();
                    if ui_text_row(ui, "Ref ID", &mut ref_id) {
                        let trimmed = ref_id.trim();
                        spawn.ref_id = if trimmed.is_empty() {
                            None
                        } else {
                            Some(trimmed.to_string())
                        };
                        state.spawns_dirty = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Position");
                        if ui.add(egui::DragValue::new(&mut spawn.x)).changed()
                            | ui.add(egui::DragValue::new(&mut spawn.y)).changed()
                        {
                            state.spawns_dirty = true;
                        }
                    });
                    ui.label(if state.spawns_dirty { "Dirty" } else { "Clean" });
                } else {
                    ui.label("No spawn selected.");
                }
                return;
            }
            EditorSelection::Trigger => {
                ui.heading("Selected trigger");
                let Some(state) = self.world_placements.as_mut() else {
                    ui.label("No trigger state loaded.");
                    return;
                };
                if let Some(trigger) = state.triggers.get_mut(state.selected_trigger_index) {
                    if ui_text_row(ui, "ID", &mut trigger.id)
                        | ui_text_row(ui, "Kind", &mut trigger.kind)
                        | ui_text_row(ui, "Target map", &mut trigger.target_map)
                    {
                        state.triggers_dirty = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Bounds");
                        let changed = ui.add(egui::DragValue::new(&mut trigger.x)).changed()
                            | ui.add(egui::DragValue::new(&mut trigger.y)).changed()
                            | ui.add(egui::DragValue::new(&mut trigger.w).range(1..=999))
                                .changed()
                            | ui.add(egui::DragValue::new(&mut trigger.h).range(1..=999))
                                .changed();
                        if changed {
                            state.triggers_dirty = true;
                        }
                    });
                    ui.label(if state.triggers_dirty {
                        "Dirty"
                    } else {
                        "Clean"
                    });
                } else {
                    ui.label("No trigger selected.");
                }
                return;
            }
            EditorSelection::VoxelObject => {
                ui.heading("Selected voxel object");
                let Some(state) = self.world_placements.as_mut() else {
                    ui.label("No voxel object state loaded.");
                    return;
                };
                if let Some(object) = state
                    .voxel_objects
                    .objects
                    .get_mut(state.selected_voxel_object_index)
                {
                    if ui_text_row(ui, "ID", &mut object.id)
                        | ui_text_row(ui, "Display", &mut object.display_name)
                        | ui_text_row(ui, "Source kind", &mut object.source_kind)
                        | ui_text_row(ui, "Source ID", &mut object.source_id)
                        | ui_text_row(ui, "Source path", &mut object.source_path)
                    {
                        state.voxel_objects_dirty = true;
                    }
                    ui.horizontal(|ui| {
                        ui.label("Position");
                        let changed = ui.add(egui::DragValue::new(&mut object.x)).changed()
                            | ui.add(egui::DragValue::new(&mut object.y)).changed()
                            | ui.add(egui::DragValue::new(&mut object.z)).changed();
                        if changed {
                            state.voxel_objects_dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Shape");
                        let changed = ui
                            .add(egui::DragValue::new(&mut object.yaw_degrees).speed(1.0))
                            .changed()
                            | ui.add(
                                egui::DragValue::new(&mut object.footprint_width)
                                    .range(0.1..=999.0),
                            )
                            .changed()
                            | ui.add(
                                egui::DragValue::new(&mut object.footprint_height)
                                    .range(0.1..=999.0),
                            )
                            .changed()
                            | ui.add(egui::DragValue::new(&mut object.height).range(0.1..=999.0))
                                .changed();
                        if changed {
                            state.voxel_objects_dirty = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Anchor");
                        let changed = ui.add(egui::DragValue::new(&mut object.anchor_x)).changed()
                            | ui.add(egui::DragValue::new(&mut object.anchor_y)).changed();
                        if changed {
                            state.voxel_objects_dirty = true;
                        }
                    });
                    if ui_text_row(ui, "Collision", &mut object.collision_kind)
                        | ui.checkbox(&mut object.locked, "Locked").changed()
                        | ui_text_row(ui, "Notes", &mut object.notes)
                    {
                        state.voxel_objects_dirty = true;
                    }
                    ui.label(if state.voxel_objects_dirty {
                        "Dirty"
                    } else {
                        "Clean"
                    });
                } else {
                    ui.label("No voxel object selected.");
                }
                return;
            }
            EditorSelection::PixelSelection => {
                ui.heading("Pixel selection");
                ui.label(format!(
                    "Texture: {}",
                    self.pixel_editor.image_path.display()
                ));
                if let Some((x, y, width, height)) = self.pixel_editor.normalized_selection() {
                    ui.label(format!("Origin: {x},{y}"));
                    ui.label(format!("Size: {width} x {height}"));
                    ui.label(format!("Area: {} px", width.saturating_mul(height)));
                    if let Some((hover_x, hover_y)) = self.pixel_editor.hover_pixel {
                        ui.label(format!("Hover: {hover_x},{hover_y}"));
                    }
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Copy selection").clicked() {
                            if self.pixel_editor.copy_selection() {
                                self.pixel_editor.tool = PixelTool::Paste;
                                self.status = "Copied pixel selection.".to_string();
                            }
                        }
                        if ui.button("Clear selection").clicked() {
                            self.pixel_editor.selection_start = None;
                            self.pixel_editor.selection_end = None;
                            self.editor_selection = EditorSelection::Tile;
                            self.status = "Cleared pixel selection.".to_string();
                        }
                    });
                    ui.label(if self.pixel_editor.dirty {
                        "Dirty"
                    } else {
                        "Clean"
                    });
                } else {
                    ui.label("No pixel region is selected.");
                }
                return;
            }
            EditorSelection::VoxelPanelSelection => {
                ui.heading("Voxel panel selection");
                ui.label(format!(
                    "Kit: {}",
                    self.voxel_panel_designer.kit_path.display()
                ));
                ui.label(format!(
                    "Mode: {}",
                    self.voxel_panel_designer.workspace_mode.label()
                ));

                if let Some(panel) = self.voxel_panel_designer.selected_panel() {
                    ui.separator();
                    ui.label(format!("Panel: {}", panel.display_name));
                    ui.label(format!("ID: {}", panel.id));
                    ui.label(format!("Kind: {}", panel.panel_kind));
                    ui.label(format!(
                        "Size: {} x {} x {}",
                        panel.width, panel.height, panel.depth
                    ));
                    ui.label(format!("Cells: {}", panel.cells.len()));
                    ui.label(format!("Sockets: {}", panel.sockets.len()));
                    if let Some(socket) = panel
                        .sockets
                        .get(self.voxel_panel_designer.selected_socket_index)
                    {
                        ui.label(format!(
                            "Socket: {} @ {},{},{}",
                            socket.id, socket.x, socket.y, socket.z
                        ));
                    }
                }

                if let Some((x, y)) = self.voxel_panel_designer.hover_cell {
                    ui.label(format!(
                        "Hover cell: {x},{y},{}",
                        self.voxel_panel_designer.active_depth
                    ));
                }
                ui.label(format!(
                    "Material: {}",
                    self.voxel_panel_designer.selected_material_id
                ));

                if let Some(composition) = self.voxel_panel_designer.selected_composition() {
                    ui.separator();
                    ui.label(format!("Composition: {}", composition.display_name));
                    ui.label(format!("ID: {}", composition.id));
                    ui.label(format!(
                        "Canvas: {} x {} x {}",
                        composition.canvas_width,
                        composition.canvas_height,
                        composition.canvas_depth
                    ));
                    ui.label(format!("Instances: {}", composition.instances.len()));
                    ui.label(format!("Connections: {}", composition.connections.len()));
                }

                if let Some(instance) = self.voxel_panel_designer.selected_composition_instance() {
                    ui.label(format!("Instance: {}", instance.id));
                    ui.label(format!("Panel source: {}", instance.panel_id));
                    ui.label(format!(
                        "Position: {},{},{}",
                        instance.x, instance.y, instance.z
                    ));
                }

                if let Some(path) = &self.voxel_panel_designer.last_mesh_export_path {
                    ui.separator();
                    ui.label(format!("Last export: {}", path.display()));
                }
                if let Some(summary) = &self.voxel_panel_designer.last_mesh_export_summary {
                    ui.small(summary);
                }

                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    for mode in VoxelPanelWorkspaceMode::ALL {
                        if ui.button(mode.label()).clicked() {
                            self.voxel_panel_designer.workspace_mode = mode;
                        }
                    }
                });
                ui.label(if self.voxel_panel_designer.dirty {
                    "Dirty"
                } else {
                    "Clean"
                });
                return;
            }
            EditorSelection::Layer => {
                ui.heading("Selected layer");
                let Some(state) = self.editor_map.as_mut() else {
                    ui.label("No editable map state loaded.");
                    return;
                };
                let index = state.selected_layer_index;
                let dirty = state.dirty;
                if let Some(layer) = state.layers.layers.get_mut(index) {
                    ui.label(format!("ID: {}", layer.id));
                    let mut visible = layer.visible;
                    if ui.checkbox(&mut visible, "Visible").changed() {
                        layer.visible = visible;
                        state.dirty = true;
                    }
                    let mut locked = layer.locked;
                    if ui.checkbox(&mut locked, "Locked").changed() {
                        layer.locked = locked;
                        state.dirty = true;
                    }
                    if ui
                        .add(egui::Slider::new(&mut layer.opacity, 0.0..=1.0).text("Opacity"))
                        .changed()
                    {
                        layer.opacity = layer.opacity.clamp(0.0, 1.0);
                        state.dirty = true;
                    }
                    ui.label(format!("Rows: {}", layer.rows.len()));
                    ui.label(format!("Legend entries: {}", layer.legend.len()));
                    ui.label(if dirty { "Dirty" } else { "Clean" });
                } else {
                    ui.label("No layer selected.");
                }
                return;
            }
            EditorSelection::MapCell | EditorSelection::Tile => {}
        }

        ui.label("Selected tile");
        ui.monospace(&self.selected_tile_id);
        ui.label(format!(
            "Atlas cell: {},{}",
            self.selected_cell.0, self.selected_cell.1
        ));

        if let Some((x, y)) = self.selected_map_cell {
            ui.label(format!("Selected map cell: {x},{y}"));
        } else {
            ui.label("Selected map cell: none");
        }

        ui.separator();

        let mut next_role = self.role_state.role.clone();
        egui::ComboBox::from_id_salt("tile_role_combo")
            .selected_text(&next_role)
            .show_ui(ui, |ui| {
                for role in EDITOR_ROLE_CYCLE {
                    ui.selectable_value(&mut next_role, (*role).to_string(), *role);
                }
            });

        if next_role != self.role_state.role {
            self.role_state = TileRoleState::from_role(&next_role);
            self.status = format!(
                "Changed role for {} to {}.",
                self.selected_tile_id, self.role_state.role
            );
        }

        let mut next_collision = self.role_state.collision.clone();
        egui::ComboBox::from_id_salt("tile_collision_combo")
            .selected_text(&next_collision)
            .show_ui(ui, |ui| {
                for collision in EDITOR_COLLISION_CYCLE {
                    ui.selectable_value(&mut next_collision, (*collision).to_string(), *collision);
                }
            });

        if next_collision != self.role_state.collision {
            self.role_state = TileRoleState::from_collision(&self.role_state.role, &next_collision);
            self.status = format!(
                "Changed collision for {} to {}.",
                self.selected_tile_id, self.role_state.collision
            );
        }

        ui.separator();
        ui.checkbox(&mut self.role_state.walkable, "Walkable");
        ui.checkbox(&mut self.role_state.blocks_movement, "Blocks movement");
        ui.checkbox(&mut self.role_state.water, "Water");
        ui.checkbox(&mut self.role_state.interactable, "Interactable");
        ui.checkbox(&mut self.role_state.crop_soil, "Crop soil");
        ui.checkbox(&mut self.role_state.door, "Door");

        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Save metadata").clicked() {
                self.save_role_state();
            }
            if ui.button("Previous tile").clicked() {
                self.select_next_tile(-1);
            }
            if ui.button("Next tile").clicked() {
                self.select_next_tile(1);
            }
        });
    }

    fn draw_seam_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label("Native egui seam and transition tools");
        ui.separator();
        ui.label("This panel replaces the old GL placeholder buttons and no longer routes editor.exe back into the web Asset Lab.");
        ui.label("Still missing here: pixel seam painting, 4-season variant preview, water animation preview, and atlas compare/merge.");
        ui.add_space(8.0);

        if ui.button("Mark selected tile for seam cleanup").clicked() {
            self.status = format!("Queued seam cleanup marker for {}.", self.selected_tile_id);
            self.log(self.status.clone());
        }
        if ui.button("Focus native Asset Lab").clicked() {
            self.left_tab = LeftTab::Textures;
            self.status = format!(
                "Focused native egui Asset Lab for {}.",
                self.selected_tile_id
            );
            self.log(self.status.clone());
        }
    }

    fn draw_export_inspector(&mut self, ui: &mut egui::Ui) {
        ui.label("Export and validation");
        ui.separator();
        ui.label("Checkpoint output lets the next patch wire real atlas import/export without the GL overlay hit-test layer.");
        if ui.button("Write egui checkpoint").clicked() {
            self.write_selection_manifest();
        }
        if ui.button("Rewrite live preview manifest").clicked() {
            match write_editor_live_preview_manifest(&self.project_root, &self.active_map_id) {
                Ok(()) => {
                    self.status = "Rewrote editor_live_preview.ron.".to_string();
                    self.log(self.status.clone());
                }
                Err(error) => {
                    self.status = "Failed to rewrite live preview manifest.".to_string();
                    self.log(format!("Live preview manifest error: {error:#}"));
                }
            }
        }

        ui.separator();
        ui.label("Current validation coverage");
        ui.monospace(format!(
            "editor atlas pipelines: {}\nexport pipelines: {}\nanimation pipelines: {}\nterrain rulesets: {}",
            self.registry.editor_atlas_pipelines.len(),
            self.registry.editor_export_pipelines.len(),
            self.registry.editor_animation_pipelines.len(),
            self.registry.terrain_rulesets.len(),
        ));
    }

    fn draw_bottom_panel(&mut self, root_ui: &mut egui::Ui) {
        egui::Panel::bottom("editor_bottom_panel")
            .resizable(true)
            .default_size(140.0)
            .min_size(96.0)
            .max_size(320.0)
            .show_inside(root_ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Console, "Console");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Validation, "Validation");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::HotReload, "Hot Reload");
                    ui.selectable_value(&mut self.bottom_tab, BottomTab::Runtime, "Runtime");
                });
                ui.separator();

                let body_height = (ui.available_height() - 4.0).max(0.0);
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(16, 20, 28))
                    .inner_margin(egui::Margin::symmetric(8, 6))
                    .show(ui, |ui| {
                        ui.set_min_height(body_height);
                        ui.set_max_height(body_height);
                        match self.bottom_tab {
                            BottomTab::Console => self.draw_console_tab(ui),
                            BottomTab::Validation => self.draw_validation_tab(ui),
                            BottomTab::HotReload => self.draw_hot_reload_tab(ui),
                            BottomTab::Runtime => self.draw_runtime_tab(ui),
                        }
                    });
            });
    }

    fn draw_status_bar(&mut self, root_ui: &mut egui::Ui) {
        egui::Panel::bottom("editor_static_status_bar")
            .resizable(false)
            .exact_size(28.0)
            .show_inside(root_ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.strong("Status");
                    ui.separator();
                    ui.label(&self.status);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("tile: {}", self.selected_tile_id));
                        ui.separator();
                        ui.label(format!("map: {}", self.active_map_id));
                        ui.separator();
                        ui.label(format!("tool: {}", TOOL_NAMES[self.selected_tool]));
                        ui.separator();
                        let map_state = self
                            .editor_map
                            .as_ref()
                            .map(|state| {
                                format!(
                                    "layer: {} · {}",
                                    state.selected_layer_id(),
                                    if state.dirty { "dirty" } else { "clean" }
                                )
                            })
                            .unwrap_or_else(|| "layer: none".to_string());
                        ui.label(map_state);
                    });
                });
            });
    }

    fn draw_console_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &self.log_lines {
                    ui.label(line);
                }
            });
    }

    fn draw_validation_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Map layer validation");
        let issues = self.map_layer_validation_issues();
        if issues.is_empty() {
            ui.label(format!(
                "No layer issues detected for '{}'.",
                self.active_map_id
            ));
        } else {
            ui.label(format!(
                "{} issue(s) detected for '{}':",
                issues.len(),
                self.active_map_id
            ));
            egui::ScrollArea::vertical().show(ui, |ui| {
                for issue in issues.iter().take(80) {
                    ui.label(format!("• {issue}"));
                }
                if issues.len() > 80 {
                    ui.label(format!("…and {} more.", issues.len() - 80));
                }
            });
        }

        ui.separator();
        ui.label("Other queued validation targets:");
        ui.label("• external atlas import must validate size, tile grid, role tags, and collisions before merging");
        ui.label("• animation timeline events need socket/hitbox preview and save validation");
        ui.label("• seasonal tile sets need parity checks across spring/summer/autumn/winter");
    }

    fn draw_hot_reload_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Manual hot reload is active through F5.");
        ui.label("The old editor watched PNG/RON/TOML files from the GL loop. The egui path currently reloads explicitly so the UI stays deterministic.");
        if ui
            .add_enabled(
                self.content_reload_job.is_none(),
                egui::Button::new("Reload now"),
            )
            .clicked()
        {
            self.reload_content();
        }
    }

    fn draw_runtime_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Active map: {}", self.active_map_id));
        ui.label(format!(
            "Tile render instances: {}",
            self.tile_map
                .as_ref()
                .map(|map| map.tiles.len())
                .unwrap_or(0)
        ));
        ui.label(format!("Selected tool: {}", TOOL_NAMES[self.selected_tool]));
        ui.label(format!("Selected tile: {}", self.selected_tile_id));
        if let Some(state) = &self.editor_map {
            ui.label(format!("Editable layer: {}", state.selected_layer_id()));
            ui.label(format!("Layer dirty: {}", state.dirty));
            ui.label(format!("Layers path: {}", state.layers_path.display()));
        }
    }

    fn draw_center_panel(&mut self, root_ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(12, 15, 22)))
            .show_inside(root_ui, |ui| match self.workspace_tab {
                WorkspaceTab::Project => self.draw_project_workspace(ui),
                WorkspaceTab::World => self.draw_world_workspace(ui),
                WorkspaceTab::Assets => self.draw_assets_workspace(ui),
                WorkspaceTab::Animation => self.draw_animation_workspace(ui),
                WorkspaceTab::Character => self.draw_character_workspace(ui),
                WorkspaceTab::Logic => self.draw_logic_workspace(ui),
                WorkspaceTab::Data => self.draw_data_workspace(ui),
                WorkspaceTab::Playtest => self.draw_playtest_workspace(ui),
                WorkspaceTab::Settings => self.draw_settings_workspace(ui),
            });
    }

    fn draw_project_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Project Overview",
            "Project health, content counts, and build/readiness checks.",
        );
        match self.project_subtab {
            ProjectSubTab::Overview => {
                ui.columns(3, |columns| {
                    columns[0].heading("Content");
                    columns[0].label(format!("Maps: {}", self.registry.maps.len()));
                    columns[0].label(format!("Tilesets: {}", self.registry.tilesets.len()));
                    columns[0].label(format!(
                        "Sprite sheets: {}",
                        self.registry.sprite_sheets.len()
                    ));
                    columns[1].heading("Gameplay data");
                    columns[1].label(format!("Items: {}", self.registry.items.len()));
                    columns[1].label(format!("Crops: {}", self.registry.crops.len()));
                    columns[1].label(format!("NPCs: {}", self.registry.npcs.len()));
                    columns[2].heading("Editor contracts");
                    columns[2].label(format!(
                        "Atlas pipelines: {}",
                        self.registry.editor_atlas_pipelines.len()
                    ));
                    columns[2].label(format!(
                        "Export pipelines: {}",
                        self.registry.editor_export_pipelines.len()
                    ));
                    columns[2].label(format!(
                        "Animation pipelines: {}",
                        self.registry.editor_animation_pipelines.len()
                    ));
                });
            }
            ProjectSubTab::Validation => self.draw_validation_tab(ui),
            ProjectSubTab::Diagnostics => self.draw_runtime_tab(ui),
            ProjectSubTab::Build => self.draw_workspace_notes(
                ui,
                "Build",
                &[
                    "Build profiles",
                    "Content smoke checks",
                    "Release diagnostics bundle",
                ],
            ),
            ProjectSubTab::Export => self.draw_export_inspector(ui),
        }
    }

    fn draw_world_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "World Editor",
            "Real editable map layers, direct painting, save/reload, and layer validation.",
        );
        match self.world_subtab {
            WorldSubTab::MapPaint => self.draw_world_preview_workspace(ui),
            WorldSubTab::Layers => self.draw_world_layers_workspace(ui),
            WorldSubTab::Objects => self.draw_world_objects_workspace(ui),
            WorldSubTab::TerrainRules => self.draw_worldgen_bake_workspace(ui),
        }
    }

    fn draw_world_objects_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "World Objects",
            "Unified prop, spawn, trigger, and voxel object editing for the active map.",
        );

        let mut duplicate_requested = false;
        let mut remove_requested = false;
        let Some(state) = self.world_placements.as_mut() else {
            ui.label("No editable object placement files loaded for this map.");
            if ui.button("Reload placements").clicked() {
                self.load_world_placement_state();
            }
            return;
        };

        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Map: {}", state.map_id));
            ui.label(format!("{} prop(s)", state.props.len()));
            ui.label(format!("{} spawn(s)", state.spawns.len()));
            ui.label(format!("{} trigger(s)", state.triggers.len()));
            ui.label(format!(
                "{} voxel object(s)",
                state.voxel_objects.objects.len()
            ));
            let dirty = state.props_dirty
                || state.spawns_dirty
                || state.triggers_dirty
                || state.voxel_objects_dirty;
            ui.label(if dirty { "Dirty" } else { "Clean" });
            ui.separator();
            ui.label("Show:");
            ui.selectable_value(&mut self.world_object_filter, WorldObjectFilter::All, "All");
            ui.selectable_value(
                &mut self.world_object_filter,
                WorldObjectFilter::Props,
                "Props",
            );
            ui.selectable_value(
                &mut self.world_object_filter,
                WorldObjectFilter::Spawns,
                "Spawns",
            );
            ui.selectable_value(
                &mut self.world_object_filter,
                WorldObjectFilter::Triggers,
                "Triggers",
            );
            ui.selectable_value(
                &mut self.world_object_filter,
                WorldObjectFilter::VoxelObjects,
                "Voxel Objects",
            );
        });

        ui.horizontal_wrapped(|ui| {
            if ui.button("Add prop").clicked() {
                state.add_prop();
                self.editor_selection = EditorSelection::Prop;
                self.world_object_filter = WorldObjectFilter::Props;
            }
            if ui.button("Add spawn").clicked() {
                state.add_spawn();
                self.editor_selection = EditorSelection::Spawn;
                self.world_object_filter = WorldObjectFilter::Spawns;
            }
            if ui.button("Add trigger").clicked() {
                state.add_trigger();
                self.editor_selection = EditorSelection::Trigger;
                self.world_object_filter = WorldObjectFilter::Triggers;
            }
            if ui.button("Add voxel object").clicked() {
                state.add_voxel_object();
                self.editor_selection = EditorSelection::VoxelObject;
                self.world_object_filter = WorldObjectFilter::VoxelObjects;
            }
            if ui.button("Duplicate selected").clicked() {
                duplicate_requested = true;
            }
            if ui.button("Remove selected").clicked() {
                remove_requested = true;
            }
            ui.small("Edit details in the right inspector. Save/reload in command strip.");
        });

        ui.separator();
        ui.columns(2, |columns| {
            columns[0].heading("Object list");
            egui::ScrollArea::vertical()
                .id_salt("world_object_list")
                .show(&mut columns[0], |ui| {
                    if matches!(
                        self.world_object_filter,
                        WorldObjectFilter::All | WorldObjectFilter::Props
                    ) {
                        ui.strong("Props");
                        for (index, prop) in state.props.iter().enumerate() {
                            let selected = self.editor_selection == EditorSelection::Prop
                                && state.selected_prop_index == index;
                            if ui
                                .selectable_label(
                                    selected,
                                    format!("{} - {} @ {},{}", prop.id, prop.kind, prop.x, prop.y),
                                )
                                .clicked()
                            {
                                state.selected_prop_index = index;
                                state.active_selection = WorldPlacementKind::Prop;
                                self.editor_selection = EditorSelection::Prop;
                            }
                        }
                        ui.add_space(6.0);
                    }

                    if matches!(
                        self.world_object_filter,
                        WorldObjectFilter::All | WorldObjectFilter::Spawns
                    ) {
                        ui.strong("Spawns");
                        for (index, spawn) in state.spawns.iter().enumerate() {
                            let selected = self.editor_selection == EditorSelection::Spawn
                                && state.selected_spawn_index == index;
                            if ui
                                .selectable_label(
                                    selected,
                                    format!(
                                        "{} - {} @ {},{}",
                                        spawn.id, spawn.kind, spawn.x, spawn.y
                                    ),
                                )
                                .clicked()
                            {
                                state.selected_spawn_index = index;
                                state.active_selection = WorldPlacementKind::Spawn;
                                self.editor_selection = EditorSelection::Spawn;
                            }
                        }
                        ui.add_space(6.0);
                    }

                    if matches!(
                        self.world_object_filter,
                        WorldObjectFilter::All | WorldObjectFilter::Triggers
                    ) {
                        ui.strong("Triggers");
                        for (index, trigger) in state.triggers.iter().enumerate() {
                            let selected = self.editor_selection == EditorSelection::Trigger
                                && state.selected_trigger_index == index;
                            if ui
                                .selectable_label(
                                    selected,
                                    format!(
                                        "{} - {} -> {} @ {},{} {}x{}",
                                        trigger.id,
                                        trigger.kind,
                                        trigger.target_map,
                                        trigger.x,
                                        trigger.y,
                                        trigger.w,
                                        trigger.h
                                    ),
                                )
                                .clicked()
                            {
                                state.selected_trigger_index = index;
                                state.active_selection = WorldPlacementKind::Trigger;
                                self.editor_selection = EditorSelection::Trigger;
                            }
                        }
                        ui.add_space(6.0);
                    }

                    if matches!(
                        self.world_object_filter,
                        WorldObjectFilter::All | WorldObjectFilter::VoxelObjects
                    ) {
                        ui.strong("Voxel Objects");
                        for (index, object) in state.voxel_objects.objects.iter().enumerate() {
                            let selected = self.editor_selection == EditorSelection::VoxelObject
                                && state.selected_voxel_object_index == index;
                            if ui
                                .selectable_label(
                                    selected,
                                    format!(
                                        "{} - {} @ {:.1},{:.1},{:.1}",
                                        object.id, object.source_id, object.x, object.y, object.z
                                    ),
                                )
                                .clicked()
                            {
                                state.selected_voxel_object_index = index;
                                state.active_selection = WorldPlacementKind::VoxelObject;
                                self.editor_selection = EditorSelection::VoxelObject;
                            }
                        }
                    }
                });

            columns[1].heading("Selected object");
            match self.editor_selection {
                EditorSelection::Prop => {
                    if let Some(prop) = state.props.get(state.selected_prop_index) {
                        columns[1].label("Type: Prop");
                        columns[1].label(format!("ID: {}", prop.id));
                        columns[1].label(format!("Kind: {}", prop.kind));
                        columns[1].label(format!("Position: {},{}", prop.x, prop.y));
                    } else {
                        columns[1].label("No prop selected.");
                    }
                }
                EditorSelection::Spawn => {
                    if let Some(spawn) = state.spawns.get(state.selected_spawn_index) {
                        columns[1].label("Type: Spawn");
                        columns[1].label(format!("ID: {}", spawn.id));
                        columns[1].label(format!("Kind: {}", spawn.kind));
                        columns[1].label(format!("Position: {},{}", spawn.x, spawn.y));
                        columns[1].label(format!(
                            "Ref: {}",
                            spawn.ref_id.as_deref().unwrap_or("<none>")
                        ));
                    } else {
                        columns[1].label("No spawn selected.");
                    }
                }
                EditorSelection::Trigger => {
                    if let Some(trigger) = state.triggers.get(state.selected_trigger_index) {
                        columns[1].label("Type: Trigger");
                        columns[1].label(format!("ID: {}", trigger.id));
                        columns[1].label(format!("Kind: {}", trigger.kind));
                        columns[1].label(format!("Target: {}", trigger.target_map));
                        columns[1].label(format!(
                            "Bounds: {},{} {}x{}",
                            trigger.x, trigger.y, trigger.w, trigger.h
                        ));
                    } else {
                        columns[1].label("No trigger selected.");
                    }
                }
                EditorSelection::VoxelObject => {
                    if let Some(object) = state
                        .voxel_objects
                        .objects
                        .get(state.selected_voxel_object_index)
                    {
                        columns[1].label("Type: Voxel Object");
                        columns[1].label(format!("ID: {}", object.id));
                        columns[1].label(format!("Source: {}", object.source_id));
                        columns[1].label(format!(
                            "Position: {:.1},{:.1},{:.1}",
                            object.x, object.y, object.z
                        ));
                        columns[1].label(format!(
                            "Footprint: {:.1}x{:.1}, h {:.1}",
                            object.footprint_width, object.footprint_height, object.height
                        ));
                    } else {
                        columns[1].label("No voxel object selected.");
                    }
                }
                _ => {
                    columns[1].label(
                        "Select a prop, spawn, trigger, or voxel object from the list or map.",
                    );
                }
            }
        });

        if duplicate_requested {
            self.duplicate_active_world_placement();
        }
        if remove_requested {
            self.delete_active_world_placement();
        }
    }

    fn draw_worldgen_bake_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "WorldGen Bake Commit",
            "Generate a semantic scene preview, validate it, back up layers.ron, and commit into editable map layers.",
        );

        ui.horizontal(|ui| {
            if ui.button("Generate / Refresh Preview").clicked() {
                self.generate_worldgen_bake_preview();
            }
            let commit_enabled = self.worldgen_bake_preview.is_some();
            if ui
                .add_enabled(
                    commit_enabled,
                    egui::Button::new("Backup + Commit to Editable Layers"),
                )
                .clicked()
            {
                self.commit_worldgen_bake_preview();
            }
            if ui.button("Reload Content").clicked() {
                self.reload_content();
            }
        });

        ui.separator();
        ui.columns(3, |columns| {
            columns[0].heading("Draft");
            if let Some(draft) = self
                .registry
                .generated_scene_drafts
                .values()
                .find(|draft| draft.scene_id == self.active_map_id)
                .or_else(|| self.registry.generated_scene_drafts.values().next())
            {
                columns[0].label(format!("id: {}", draft.id));
                columns[0].label(format!("scene: {}", draft.scene_id));
                columns[0].label(format!("seed: {}", draft.seed));
                columns[0].label(format!("template: {}", draft.template_id));
                columns[0].label(format!("contract size: {}x{}", draft.width, draft.height));
            } else {
                columns[0].label("No generated scene draft loaded.");
            }

            columns[1].heading("Bake target");
            columns[1].label(format!("active map: {}", self.active_map_id));
            if let Some(state) = &self.editor_map {
                let (width, height) = layer_dimensions(&state.layers);
                columns[1].label(format!("editable map state: {}", state.map_id));
                columns[1].label(format!("editable size: {}x{}", width, height));
                columns[1].label(format!("layers file: {}", state.layers_path.display()));
                columns[1].label(format!("selected layer: {}", state.selected_layer_id()));
            } else {
                columns[1].label("No editable map state loaded.");
            }

            columns[2].heading("Contract");
            if let Some(preview) = &self.worldgen_bake_preview {
                columns[2].label(format!("draft: {}", preview.draft_id));
                columns[2].label(format!("contract: {}", preview.contract_id));
                columns[2].label(format!("terrain layer: {}", preview.target_layer_id));
                columns[2].label(format!(
                    "object layer: {}",
                    preview
                        .target_object_layer_id
                        .as_deref()
                        .unwrap_or("<none>")
                ));
                columns[2].label(format!("committed: {}", preview.report.committed));
            } else {
                columns[2].label("Generate a preview to inspect the active contract.");
            }
        });

        ui.separator();
        if let Some(preview) = &self.worldgen_bake_preview {
            ui.heading("Bake report");
            ui.label(preview.report.summary());
            if let Some(path) = &preview.report.backup_path {
                ui.label(format!("backup: {}", path.display()));
            }
            ui.collapsing("Terrain families", |ui| {
                for (terrain, count) in &preview.report.terrain_counts {
                    ui.label(format!("{:?}: {}", terrain, count));
                }
            });
            ui.collapsing("Warnings", |ui| {
                if preview.report.warnings.is_empty() {
                    ui.label("No warnings.");
                } else {
                    for warning in &preview.report.warnings {
                        ui.label(format!("• {}", warning));
                    }
                }
            });
        } else {
            ui.label("No bake preview generated yet. Use Generate / Refresh Preview first.");
        }
    }
    fn draw_map_layer_controls(&mut self, ui: &mut egui::Ui) {
        let layer_options = self
            .editor_map
            .as_ref()
            .map(|state| {
                state
                    .layers
                    .layers
                    .iter()
                    .enumerate()
                    .map(|(index, layer)| (index, layer.id.clone(), layer.visible))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if layer_options.is_empty() {
            ui.label("No editable layers are loaded.");
            return;
        }

        let mut selected_layer_index = self
            .editor_map
            .as_ref()
            .map(|state| state.selected_layer_index)
            .unwrap_or(0);
        ui.horizontal(|ui| {
            ui.label("Layer");
            egui::ComboBox::from_id_salt("world_selected_layer_combo")
                .selected_text(
                    layer_options
                        .iter()
                        .find(|(index, _, _)| *index == selected_layer_index)
                        .map(|(_, id, visible)| {
                            format!("{}{}", id, if *visible { "" } else { " (hidden)" })
                        })
                        .unwrap_or_else(|| "<none>".to_string()),
                )
                .show_ui(ui, |ui| {
                    for (index, id, visible) in &layer_options {
                        ui.selectable_value(
                            &mut selected_layer_index,
                            *index,
                            format!("{}{}", id, if *visible { "" } else { " (hidden)" }),
                        );
                    }
                });

            let mut layer_changed = false;
            if let Some(state) = self.editor_map.as_mut() {
                if selected_layer_index < state.layers.layers.len()
                    && selected_layer_index != state.selected_layer_index
                {
                    state.selected_layer_index = selected_layer_index;
                    self.editor_selection = EditorSelection::Layer;
                    if let Some(symbol) = state
                        .selected_layer()
                        .and_then(|layer| layer.legend.first())
                        .and_then(|entry| entry.symbol.chars().next())
                    {
                        state.selected_symbol = symbol;
                    }
                    layer_changed = true;
                }
            }
            if layer_changed {
                self.sync_selected_symbol_to_tile();
            }

            ui.label(
                if self
                    .editor_map
                    .as_ref()
                    .map(|state| state.dirty)
                    .unwrap_or(false)
                {
                    "Layers dirty"
                } else {
                    "Layers clean"
                },
            );
        });

        let mut visible_change = None;
        let mut locked_change = None;
        let mut opacity_change = None;
        if let Some(state) = self.editor_map.as_ref() {
            if let Some(layer) = state.selected_layer() {
                let mut visible = layer.visible;
                if ui
                    .checkbox(&mut visible, "Selected layer visible")
                    .changed()
                {
                    visible_change = Some(visible);
                }
                let mut locked = layer.locked;
                if ui.checkbox(&mut locked, "Locked").changed() {
                    locked_change = Some(locked);
                }
                let mut opacity = layer.opacity.clamp(0.0, 1.0);
                if ui
                    .add(egui::Slider::new(&mut opacity, 0.0..=1.0).text("Opacity"))
                    .changed()
                {
                    opacity_change = Some(opacity);
                }
            }
        }
        if let Some(visible) = visible_change {
            if let Some(state) = self.editor_map.as_mut() {
                let index = state.selected_layer_index;
                let before_layers = state.layers.clone();
                if let Some(layer) = state.layers.layers.get_mut(index) {
                    layer.visible = visible;
                    state.push_history_value("toggle layer visibility", before_layers);
                    state.dirty = true;
                }
            }
        }
        if let Some(locked) = locked_change {
            if let Some(state) = self.editor_map.as_mut() {
                let index = state.selected_layer_index;
                let before_layers = state.layers.clone();
                if let Some(layer) = state.layers.layers.get_mut(index) {
                    layer.locked = locked;
                    state.push_history_value("toggle layer lock", before_layers);
                    state.dirty = true;
                }
            }
        }
        if let Some(opacity) = opacity_change {
            if let Some(state) = self.editor_map.as_mut() {
                let index = state.selected_layer_index;
                let before_layers = state.layers.clone();
                if let Some(layer) = state.layers.layers.get_mut(index) {
                    layer.opacity = opacity.clamp(0.0, 1.0);
                    state.push_history_value("change layer opacity", before_layers);
                    state.dirty = true;
                }
            }
        }

        let symbol_options = self
            .editor_map
            .as_ref()
            .and_then(|state| state.selected_layer())
            .map(|layer| {
                layer
                    .legend
                    .iter()
                    .filter_map(|entry| {
                        entry
                            .symbol
                            .chars()
                            .next()
                            .map(|symbol| (symbol, entry.tile_id.clone()))
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut selected_symbol = self
            .editor_map
            .as_ref()
            .map(|state| state.selected_symbol)
            .unwrap_or('.');
        ui.horizontal(|ui| {
            ui.label("Symbol / tile");
            egui::ComboBox::from_id_salt("world_selected_symbol_combo")
                .selected_text(format!("'{}' / {}", selected_symbol, self.selected_tile_id))
                .show_ui(ui, |ui| {
                    for (symbol, tile_id) in &symbol_options {
                        ui.selectable_value(
                            &mut selected_symbol,
                            *symbol,
                            format!("'{}'  {}", symbol, tile_id),
                        );
                    }
                });
        });

        let selected_tile_from_symbol = symbol_options
            .iter()
            .find(|(symbol, _)| *symbol == selected_symbol)
            .map(|(_, tile_id)| tile_id.clone());
        if let Some(state) = self.editor_map.as_mut() {
            state.selected_symbol = selected_symbol;
        }
        if let Some(tile_id) = selected_tile_from_symbol {
            if tile_id != self.selected_tile_id {
                self.select_tile(tile_id, "Layer symbol");
            }
        }

        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.map_brush_size, 1..=9).text("Brush size"));
            ui.label("B brush - E erase - G fill - I pick");
        });
    }

    fn draw_world_preview_workspace(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("World Map Paint");
            ui.label(format!("Map: {}", self.active_map_id));
            ui.label(format!("Tool: {}", TOOL_NAMES[self.selected_tool]));
            if let Some(state) = &self.editor_map {
                ui.label(format!("Layer: {}", state.selected_layer_id()));
                ui.label(if state.dirty { "Dirty" } else { "Clean" });
            }
        });
        ui.separator();
        self.draw_map_layer_controls(ui);
        ui.separator();

        let available = ui.available_size();
        let desired = egui::vec2(available.x.max(320.0), available.y.max(260.0));
        let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 8.0, egui::Color32::from_rgb(18, 23, 32));
        painter.rect_stroke(
            rect,
            8.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(58, 72, 96)),
            egui::StrokeKind::Inside,
        );

        self.paint_world_preview(rect, &painter);

        if !ui.input(|input| input.pointer.primary_down()) {
            if self.selected_tool == 0 {
                if let (Some(start), Some(end)) = (self.world_marquee_start, self.world_marquee_end)
                {
                    if start != end {
                        self.select_world_placement_in_rect(start, end);
                    }
                }
            }
            self.world_marquee_start = None;
            self.world_marquee_end = None;
            self.world_resize_active = false;
            if let Some(state) = self.editor_map.as_mut() {
                state.finish_stroke();
                state.last_painted_cell = None;
            }
        }

        let should_process = response.clicked()
            || (response.dragged() && matches!(self.selected_tool, 0 | 2 | 3))
            || (response.drag_started() && self.selected_tool == 0);
        if should_process {
            let stroke_tool = matches!(self.selected_tool, 2 | 3);
            if stroke_tool && (response.drag_started() || response.clicked()) {
                let label = if self.selected_tool == 3 {
                    "erase map stroke"
                } else {
                    "paint map stroke"
                };
                if let Some(state) = self.editor_map.as_mut() {
                    state.begin_stroke(label);
                }
            }
            if let Some(pointer) = response.interact_pointer_pos() {
                if let Some((map_x, map_y)) = self.pos_to_map_cell(rect, pointer) {
                    if self.selected_tool == 0 {
                        if response.drag_started()
                            && self.selected_world_resize_handle_contains(rect, pointer)
                        {
                            self.world_resize_active = true;
                            self.resize_active_world_placement_to_cell(map_x, map_y);
                        } else if response.dragged() && self.world_resize_active {
                            self.resize_active_world_placement_to_cell(map_x, map_y);
                        } else if response.drag_started() {
                            if !self.select_world_placement_at_cell(map_x, map_y) {
                                self.world_marquee_start = Some((map_x, map_y));
                                self.world_marquee_end = Some((map_x, map_y));
                                self.editor_selection = EditorSelection::MapCell;
                                self.selected_map_cell = Some((map_x, map_y));
                                self.status =
                                    format!("Started marquee selection at {map_x},{map_y}.");
                            }
                        } else if response.clicked() {
                            if !self.select_world_placement_at_cell(map_x, map_y) {
                                self.editor_selection = EditorSelection::MapCell;
                                self.selected_map_cell = Some((map_x, map_y));
                                self.status = format!("Selected map cell {map_x},{map_y}.");
                            }
                        } else if response.dragged() && self.world_marquee_start.is_some() {
                            self.world_marquee_end = Some((map_x, map_y));
                        } else if response.dragged()
                            && matches!(
                                self.editor_selection,
                                EditorSelection::Prop
                                    | EditorSelection::Spawn
                                    | EditorSelection::Trigger
                                    | EditorSelection::VoxelObject
                            )
                        {
                            self.move_active_world_placement_to_cell(map_x, map_y);
                        }
                        return;
                    }
                    let clicked = response.clicked();
                    let should_skip = self
                        .editor_map
                        .as_ref()
                        .and_then(|state| state.last_painted_cell)
                        == Some((map_x, map_y))
                        && !clicked;
                    if !should_skip {
                        if let Some(state) = self.editor_map.as_mut() {
                            state.last_painted_cell = Some((map_x, map_y));
                        }
                        self.apply_map_tool_at_cell(map_x, map_y, clicked);
                    }
                }
            }
            if stroke_tool && response.clicked() {
                if let Some(state) = self.editor_map.as_mut() {
                    state.finish_stroke();
                }
            }
        }
    }

    fn draw_world_layers_workspace(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Editable Layers");
            if let Some(state) = &self.editor_map {
                ui.label(format!("{} layer(s)", state.layers.layers.len()));
                ui.label(if state.dirty { "Dirty" } else { "Clean" });
                ui.label(format!(
                    "Undo: {} | Redo: {}",
                    state.history.undo_len(),
                    state.history.redo_len()
                ));
            }
            if ui.button("Add blank layer").clicked() {
                let (width, height) = self.active_map_dimensions();
                if let Some(state) = self.editor_map.as_mut() {
                    state.add_blank_layer(width as usize, height as usize);
                    self.status = "Added blank map layer.".to_string();
                }
            }
        });
        ui.separator();
        self.draw_map_layer_controls(ui);
        ui.separator();

        let layer_rows = self
            .editor_map
            .as_ref()
            .map(|state| {
                state
                    .layers
                    .layers
                    .iter()
                    .enumerate()
                    .map(|(index, layer)| {
                        (
                            index,
                            layer.id.clone(),
                            layer.visible,
                            layer.locked,
                            layer.opacity.clamp(0.0, 1.0),
                            layer.rows.len(),
                            layer
                                .rows
                                .iter()
                                .map(|row| row.chars().count())
                                .max()
                                .unwrap_or(0),
                            layer.legend.len(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut layer_action: Option<(&'static str, usize)> = None;
        egui::Grid::new("world_layers_grid")
            .striped(true)
            .show(ui, |ui| {
                ui.strong("Active");
                ui.strong("Visible");
                ui.strong("Locked");
                ui.strong("Opacity");
                ui.strong("Layer");
                ui.strong("Rows");
                ui.strong("Width");
                ui.strong("Legend");
                ui.strong("Actions");
                ui.end_row();

                for (index, id, visible, locked, opacity, rows, width, legend_count) in layer_rows {
                    let selected = self
                        .editor_map
                        .as_ref()
                        .map(|state| state.selected_layer_index == index)
                        .unwrap_or(false);
                    if ui.selectable_label(selected, "edit").clicked() {
                        if let Some(state) = self.editor_map.as_mut() {
                            state.selected_layer_index = index;
                            self.editor_selection = EditorSelection::Layer;
                        }
                    }

                    let mut next_visible = visible;
                    if ui.checkbox(&mut next_visible, "").changed() {
                        if let Some(state) = self.editor_map.as_mut() {
                            let before_layers = state.layers.clone();
                            if let Some(layer) = state.layers.layers.get_mut(index) {
                                layer.visible = next_visible;
                                state.push_history_value("toggle layer visibility", before_layers);
                                state.dirty = true;
                            }
                        }
                    }
                    let mut next_locked = locked;
                    if ui.checkbox(&mut next_locked, "").changed() {
                        if let Some(state) = self.editor_map.as_mut() {
                            let before_layers = state.layers.clone();
                            if let Some(layer) = state.layers.layers.get_mut(index) {
                                layer.locked = next_locked;
                                state.push_history_value("toggle layer lock", before_layers);
                                state.dirty = true;
                            }
                        }
                    }
                    ui.label(format!("{:.0}%", opacity * 100.0));
                    ui.label(id);
                    ui.label(rows.to_string());
                    ui.label(width.to_string());
                    ui.label(legend_count.to_string());
                    ui.horizontal(|ui| {
                        if ui.small_button("Up").clicked() {
                            layer_action = Some(("up", index));
                        }
                        if ui.small_button("Down").clicked() {
                            layer_action = Some(("down", index));
                        }
                        if ui.small_button("Duplicate").clicked() {
                            layer_action = Some(("duplicate", index));
                        }
                        if ui.small_button("Remove").clicked() {
                            layer_action = Some(("remove", index));
                        }
                    });
                    ui.end_row();
                }
            });
        if let Some((action, index)) = layer_action {
            if let Some(state) = self.editor_map.as_mut() {
                let changed = match action {
                    "up" => state.move_layer(index, -1),
                    "down" => state.move_layer(index, 1),
                    "duplicate" => state.duplicate_layer(index),
                    "remove" => state.remove_layer(index),
                    _ => false,
                };
                if changed {
                    self.status = match action {
                        "up" | "down" => "Reordered map layer.".to_string(),
                        "duplicate" => "Duplicated map layer.".to_string(),
                        "remove" => "Removed map layer.".to_string(),
                        _ => self.status.clone(),
                    };
                    self.sync_selected_symbol_to_tile();
                } else if action == "remove" {
                    self.status = "Cannot remove the final map layer.".to_string();
                }
            }
        }

        ui.separator();
        ui.heading("Selected layer legend");
        let legend_entries =
            self.editor_map
                .as_ref()
                .and_then(|state| {
                    state.selected_layer().map(|layer| {
                        layer
                            .legend
                            .iter()
                            .filter_map(|entry| {
                                entry.symbol.chars().next().map(|symbol| {
                                    (symbol, entry.symbol.clone(), entry.tile_id.clone())
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .unwrap_or_default();
        let selected_symbol = self
            .editor_map
            .as_ref()
            .map(|state| state.selected_symbol)
            .unwrap_or('.');
        egui::ScrollArea::vertical()
            .max_height(260.0)
            .show(ui, |ui| {
                for (symbol, symbol_text, tile_id) in legend_entries {
                    let selected = selected_symbol == symbol;
                    if ui
                        .selectable_label(selected, format!("'{}'  →  {}", symbol_text, tile_id))
                        .clicked()
                    {
                        if let Some(state) = self.editor_map.as_mut() {
                            state.selected_symbol = symbol;
                        }
                        self.select_tile(tile_id, "Layer legend");
                    }
                }
            });
    }

    fn draw_assets_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Asset Studio",
            "Terrain atlas, compare/import, pixel editing, props, and seasonal asset readiness.",
        );
        ui.horizontal_wrapped(|ui| {
            if ui.button("Open web Asset Lab helper").clicked() {
                self.open_web_asset_lab();
            }
            ui.small("Optional external helper; native egui remains the primary editor path.");
        });
        ui.separator();
        match self.asset_subtab {
            AssetSubTab::TerrainAtlas => self.draw_workspace_notes(
                ui,
                "Terrain Atlas",
                &[
                    "Current patch keeps the selectable metadata list stable",
                    "Phase 42 should replace color-cell preview with the real atlas texture",
                    "Tile role and collision metadata can be saved from the inspector",
                ],
            ),
            AssetSubTab::AtlasCompare => self.draw_workspace_notes(
                ui,
                "Atlas Compare / Import",
                &[
                    "Side-by-side source/project tilesheet preview",
                    "Drag tile transfer",
                    "Overwrite/append modes",
                    "Mirror-aware paste",
                    "Metadata rewrite and validation",
                ],
            ),
            AssetSubTab::PixelEditor => self.draw_pixel_editor_workspace(ui),
            AssetSubTab::VoxelPanels => self.draw_voxel_panel_designer_workspace(ui),
            AssetSubTab::Voxels => self.draw_voxels_workspace(ui),
            AssetSubTab::VoxelGenerator => self.draw_voxel_generator_workspace(ui),
            AssetSubTab::Props => self.draw_world_objects_workspace(ui),
            AssetSubTab::Seasons => self.draw_workspace_notes(
                ui,
                "Season Variants",
                &[
                    "Spring/summer/autumn/winter parity",
                    "Season-specific atlas selection",
                    "Water animation preview",
                    "Missing variant validation",
                ],
            ),
        }
    }

    fn save_voxel_panel_kit(&mut self) {
        match self.voxel_panel_designer.save_with_backup() {
            Ok(Some(backup_path)) => {
                self.status = format!(
                    "Saved voxel panel kit with backup: {}",
                    backup_path.display()
                );
                self.log(self.status.clone());
            }
            Ok(None) => {
                self.status = "Saved voxel panel kit.".to_string();
                self.log(self.status.clone());
            }
            Err(error) => {
                self.status = "Failed to save voxel panel kit.".to_string();
                self.log(format!("Voxel panel save error: {error:#}"));
            }
        }
    }

    fn draw_voxel_panel_designer_workspace(&mut self, ui: &mut egui::Ui) {
        self.editor_selection = EditorSelection::VoxelPanelSelection;
        self.draw_workspace_header(
            ui,
            "Voxel Pixel Panel Designer",
            "Phase 53k.2: focused workbench layout for panel editing, composition, 3D preview, and diagnostics without stacking every panel into one crowded view.",
        );

        self.draw_voxel_panel_designer_toolbar(ui);
        self.draw_voxel_panel_designer_mode_bar(ui);
        ui.separator();

        match self.voxel_panel_designer.workspace_mode {
            VoxelPanelWorkspaceMode::PanelEditor => {
                ui.columns(2, |columns| {
                    egui::ScrollArea::vertical()
                        .id_salt("voxel_panel_editor_left_scroll")
                        .auto_shrink([false, false])
                        .show(&mut columns[0], |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Sections:");
                                for tab in VoxelPanelLeftTab::ALL {
                                    ui.selectable_value(
                                        &mut self.voxel_panel_left_tab,
                                        tab,
                                        tab.label(),
                                    );
                                }
                            });
                            ui.separator();
                            self.draw_voxel_panel_kit_panel(ui);
                        });
                    egui::ScrollArea::both()
                        .id_salt("voxel_panel_editor_slice_scroll")
                        .auto_shrink([false, false])
                        .show(&mut columns[1], |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Canvas:");
                                for tab in VoxelPanelRightTab::ALL {
                                    ui.selectable_value(
                                        &mut self.voxel_panel_right_tab,
                                        tab,
                                        tab.label(),
                                    );
                                }
                            });
                            ui.separator();
                            match self.voxel_panel_right_tab {
                                VoxelPanelRightTab::Slice => {
                                    self.draw_voxel_panel_canvas_panel(ui);
                                }
                                VoxelPanelRightTab::PaletteValidation => {
                                    self.draw_voxel_panel_validation_panel(ui);
                                }
                            }
                        });
                });
            }
            VoxelPanelWorkspaceMode::Composition => {
                ui.columns(2, |columns| {
                    egui::ScrollArea::both()
                        .id_salt("voxel_panel_composition_main_scroll")
                        .auto_shrink([false, false])
                        .show(&mut columns[0], |ui| {
                            self.draw_voxel_panel_composition_canvas_panel(ui);
                        });
                    egui::ScrollArea::vertical()
                        .id_salt("voxel_panel_composition_side_scroll")
                        .auto_shrink([false, false])
                        .show(&mut columns[1], |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label("Sections:");
                                for tab in VoxelPanelLeftTab::ALL {
                                    ui.selectable_value(
                                        &mut self.voxel_panel_left_tab,
                                        tab,
                                        tab.label(),
                                    );
                                }
                                ui.separator();
                                ui.selectable_value(
                                    &mut self.voxel_panel_right_tab,
                                    VoxelPanelRightTab::Slice,
                                    "Data",
                                );
                                ui.selectable_value(
                                    &mut self.voxel_panel_right_tab,
                                    VoxelPanelRightTab::PaletteValidation,
                                    "Palette / Validation",
                                );
                            });
                            ui.separator();
                            self.draw_voxel_panel_kit_panel(ui);
                            if self.voxel_panel_right_tab == VoxelPanelRightTab::PaletteValidation {
                                ui.separator();
                                self.draw_voxel_panel_validation_panel(ui);
                            }
                        });
                });
            }
            VoxelPanelWorkspaceMode::Preview3d => {
                egui::ScrollArea::vertical()
                    .id_salt("voxel_panel_preview_3d_scroll")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_voxel_panel_3d_preview_panel(ui);
                    });
            }
            VoxelPanelWorkspaceMode::Diagnostics => {
                ui.columns(2, |columns| {
                    egui::ScrollArea::vertical()
                        .id_salt("voxel_panel_diagnostics_left_scroll")
                        .auto_shrink([false, false])
                        .show(&mut columns[0], |ui| {
                            self.draw_voxel_panel_validation_panel(ui);
                        });
                    egui::ScrollArea::vertical()
                        .id_salt("voxel_panel_diagnostics_preview_scroll")
                        .auto_shrink([false, false])
                        .show(&mut columns[1], |ui| {
                            self.draw_voxel_panel_3d_preview_panel(ui);
                        });
                });
            }
        }
    }

    fn draw_voxel_panel_designer_mode_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Workbench:");
            for mode in VoxelPanelWorkspaceMode::ALL {
                ui.selectable_value(
                    &mut self.voxel_panel_designer.workspace_mode,
                    mode,
                    mode.label(),
                );
            }
        });
        ui.small(self.voxel_panel_designer.workspace_mode.summary());
    }

    fn draw_voxel_panel_designer_toolbar(&mut self, ui: &mut egui::Ui) {
        let mut reload_requested = false;
        ui.horizontal_wrapped(|ui| {
            if ui.button("Reload kits").clicked() {
                reload_requested = true;
            }
            ui.separator();
            ui.label("Tool:");
            for tool in VoxelPanelTool::ALL {
                ui.selectable_value(&mut self.voxel_panel_designer.tool, tool, tool.label());
            }
            ui.separator();
            let dirty = if self.voxel_panel_designer.dirty {
                "dirty"
            } else {
                "clean"
            };
            ui.label(format!("Kit state: {dirty}"));
            ui.small("Save in command strip");
        });

        if reload_requested {
            self.voxel_panel_designer.reload_paths(&self.project_root);
            self.status = format!(
                "Reloaded {} voxel panel kit path(s).",
                self.voxel_panel_designer.kit_paths.len()
            );
            self.log(self.status.clone());
        }
    }

    fn draw_voxel_panel_thumbnail(
        &self,
        ui: &mut egui::Ui,
        panel: &VoxelPanelDef,
        desired_size: egui::Vec2,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3.0, egui::Color32::from_rgb(24, 27, 33));

        let pad = 3.0;
        let inner = rect.shrink(pad);
        let cell_size = (inner.width() / panel.width.max(1) as f32)
            .min(inner.height() / panel.height.max(1) as f32)
            .floor()
            .clamp(1.0, 8.0);
        let used_size = egui::vec2(
            panel.width.max(1) as f32 * cell_size,
            panel.height.max(1) as f32 * cell_size,
        );
        let origin = egui::pos2(
            inner.center().x - used_size.x * 0.5,
            inner.center().y - used_size.y * 0.5,
        );
        let thumb_rect = egui::Rect::from_min_size(origin, used_size);

        for cell in &panel.cells {
            let x = origin.x + cell.x as f32 * cell_size;
            let y = origin.y + cell.y as f32 * cell_size;
            let mut color = self.voxel_panel_designer.material_color(&cell.material_id);
            if panel.depth > 1 && cell.z != self.voxel_panel_designer.active_depth {
                color = egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 120);
            }
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(x, y),
                    egui::vec2(cell_size.max(1.0), cell_size.max(1.0)),
                )
                .shrink(0.5),
                0.5,
                color,
            );
        }

        painter.rect_stroke(
            thumb_rect,
            2.0,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(120, 140, 160, 120),
            ),
            egui::StrokeKind::Inside,
        );
        response
    }

    fn draw_voxel_panel_kit_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Kit / Panel");
        ui.small("RON-backed modular panel source data with Phase 53i composition mesh-preview export prep.");
        ui.separator();

        let mut selected_kit_index = self.voxel_panel_designer.selected_kit_index;
        egui::ComboBox::from_id_salt("voxel_panel_kit_selector")
            .selected_text(
                self.voxel_panel_designer
                    .kit_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("<kit>"),
            )
            .show_ui(ui, |ui| {
                for (index, path) in self.voxel_panel_designer.kit_paths.iter().enumerate() {
                    let label = path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("<kit>");
                    ui.selectable_value(&mut selected_kit_index, index, label);
                }
            });

        if selected_kit_index != self.voxel_panel_designer.selected_kit_index {
            match self
                .voxel_panel_designer
                .load_selected_kit(selected_kit_index)
            {
                Ok(()) => {
                    self.status = format!(
                        "Loaded voxel panel kit {}.",
                        self.voxel_panel_designer.kit.id
                    );
                    self.log(self.status.clone());
                }
                Err(error) => {
                    self.status = "Failed to load voxel panel kit.".to_string();
                    self.log(format!("Voxel panel kit load error: {error:#}"));
                }
            }
        }

        ui.label(format!("ID: {}", self.voxel_panel_designer.kit.id));
        ui.label(format!(
            "Path: {}",
            self.voxel_panel_designer.kit_path.display()
        ));
        ui.label(format!(
            "{} panel(s), {} palette(s)",
            self.voxel_panel_designer.kit.panels.len(),
            self.voxel_panel_designer.kit.palettes.len()
        ));

        let mut kit_changed = false;
        ui.collapsing("Kit composition / future 3D preview", |ui| {
            let kit = &mut self.voxel_panel_designer.kit;
            ui.horizontal(|ui| {
                ui.label("Target view");
                kit_changed |= ui
                    .text_edit_singleline(&mut kit.composition.target_view)
                    .changed();
            });
            kit_changed |= ui
                .add(
                    egui::Slider::new(&mut kit.composition.snap_unit_px, 1..=64)
                        .text("Snap unit px"),
                )
                .changed();
            let mut kinds = kit.composition.allowed_panel_kinds.join(", ");
            ui.horizontal(|ui| {
                ui.label("Allowed kinds");
                if ui.text_edit_singleline(&mut kinds).changed() {
                    kit.composition.allowed_panel_kinds = voxel_panel_parse_list(&kinds);
                    kit_changed = true;
                }
            });
            kit_changed |= ui
                .add(
                    egui::Slider::new(&mut kit.preview_3d.voxel_unit, 0.05..=8.0)
                        .text("3D voxel unit"),
                )
                .changed();
            kit_changed |= ui
                .add(egui::Slider::new(&mut kit.preview_3d.layer_gap, 0.0..=4.0).text("Layer gap"))
                .changed();
            ui.horizontal(|ui| {
                ui.label("Camera");
                kit_changed |= ui
                    .text_edit_singleline(&mut kit.preview_3d.default_camera)
                    .changed();
            });
            kit_changed |= ui
                .checkbox(
                    &mut kit.preview_3d.show_socket_gizmos,
                    "Show socket gizmos in future 3D preview",
                )
                .changed();
            kit_changed |= ui
                .checkbox(
                    &mut kit.preview_3d.show_depth_separation,
                    "Show depth separation in future 3D preview",
                )
                .changed();
        });
        if kit_changed {
            self.voxel_panel_designer.dirty = true;
        }

        ui.separator();
        ui.heading("Panels");

        let mut next_panel = self.voxel_panel_designer.selected_panel_index;
        egui::ScrollArea::vertical()
            .id_salt("voxel_panel_list")
            .max_height(190.0)
            .show(ui, |ui| {
                for (index, panel) in self.voxel_panel_designer.kit.panels.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let response =
                            self.draw_voxel_panel_thumbnail(ui, panel, egui::vec2(72.0, 46.0));
                        if response.clicked() {
                            next_panel = index;
                        }
                        let selected = next_panel == index;
                        let label = format!(
                            "{}\n{}x{}x{} · {} cells",
                            panel.id,
                            panel.width,
                            panel.height,
                            panel.depth,
                            panel.cells.len()
                        );
                        if ui.selectable_label(selected, label).clicked() {
                            next_panel = index;
                        }
                    });
                    ui.separator();
                }
            });

        if next_panel != self.voxel_panel_designer.selected_panel_index {
            self.voxel_panel_designer.selected_panel_index = next_panel;
            self.voxel_panel_designer.active_depth = 0;
            self.voxel_panel_designer.selected_socket_index = 0;
            self.voxel_panel_designer.normalize_selection();
            if let Some(panel) = self.voxel_panel_designer.selected_panel() {
                self.status = format!("Selected voxel panel {}.", panel.id);
            }
        }

        ui.horizontal(|ui| {
            if ui.button("Add blank").clicked() {
                let index = self.voxel_panel_designer.kit.panels.len() + 1;
                self.voxel_panel_designer.kit.panels.push(VoxelPanelDef {
                    id: format!("panel_{index:02}"),
                    display_name: format!("Panel {index:02}"),
                    panel_kind: "custom".to_string(),
                    width: 16,
                    height: 16,
                    depth: 2,
                    composition: VoxelPanelCompositionDef::default(),
                    cells: Vec::new(),
                    sockets: Vec::new(),
                });
                self.voxel_panel_designer.selected_panel_index =
                    self.voxel_panel_designer.kit.panels.len() - 1;
                self.voxel_panel_designer.selected_socket_index = 0;
                self.voxel_panel_designer.dirty = true;
            }
            if ui.button("Duplicate").clicked() {
                if let Some(panel) = self.voxel_panel_designer.selected_panel().cloned() {
                    let mut clone = panel;
                    clone.id = format!("{}_copy", clone.id);
                    clone.display_name = format!("{} Copy", clone.display_name);
                    self.voxel_panel_designer.kit.panels.push(clone);
                    self.voxel_panel_designer.selected_panel_index =
                        self.voxel_panel_designer.kit.panels.len() - 1;
                    self.voxel_panel_designer.selected_socket_index = 0;
                    self.voxel_panel_designer.dirty = true;
                }
            }
        });

        ui.separator();
        let mut panel_changed = false;
        let mut add_socket = false;
        let mut remove_socket = false;
        let mut selected_socket_index = self.voxel_panel_designer.selected_socket_index;
        let active_depth = self.voxel_panel_designer.active_depth;

        match self.voxel_panel_left_tab {
            VoxelPanelLeftTab::Panels => {
                ui.heading("Selected panel");
                if let Some(panel) = self.voxel_panel_designer.selected_panel_mut() {
                    ui.horizontal(|ui| {
                        ui.label("ID");
                        panel_changed |= ui.text_edit_singleline(&mut panel.id).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Name");
                        panel_changed |= ui.text_edit_singleline(&mut panel.display_name).changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label("Kind");
                        panel_changed |= ui.text_edit_singleline(&mut panel.panel_kind).changed();
                    });
                    panel_changed |= ui
                        .add(egui::Slider::new(&mut panel.width, 1..=128).text("Width"))
                        .changed();
                    panel_changed |= ui
                        .add(egui::Slider::new(&mut panel.height, 1..=128).text("Height"))
                        .changed();
                    panel_changed |= ui
                        .add(egui::Slider::new(&mut panel.depth, 1..=32).text("Depth"))
                        .changed();

                    if panel_changed {
                        panel.cells.retain(|cell| {
                            cell.x < panel.width && cell.y < panel.height && cell.z < panel.depth
                        });
                        panel.sockets.retain(|socket| {
                            socket.x < panel.width
                                && socket.y < panel.height
                                && socket.z < panel.depth
                        });
                    }

                    ui.collapsing("Composition metadata", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Group");
                            panel_changed |= ui
                                .text_edit_singleline(&mut panel.composition.group_id)
                                .changed();
                        });
                        ui.horizontal(|ui| {
                            ui.label("Anchor");
                            panel_changed |= ui
                                .text_edit_singleline(&mut panel.composition.anchor)
                                .changed();
                        });
                        panel_changed |= ui
                            .add(
                                egui::Slider::new(&mut panel.composition.snap_priority, -100..=100)
                                    .text("Snap priority"),
                            )
                            .changed();
                        panel_changed |= ui
                            .checkbox(&mut panel.composition.allow_rotation, "Allow rotation")
                            .changed();
                        panel_changed |= ui
                            .checkbox(&mut panel.composition.allow_mirror_x, "Allow mirror X")
                            .changed();
                        panel_changed |= ui
                            .checkbox(&mut panel.composition.allow_mirror_y, "Allow mirror Y")
                            .changed();
                        let mut tags = panel.composition.tags.join(", ");
                        ui.horizontal(|ui| {
                            ui.label("Tags");
                            if ui.text_edit_singleline(&mut tags).changed() {
                                panel.composition.tags = voxel_panel_parse_list(&tags);
                                panel_changed = true;
                            }
                        });
                    });
                }
            }
            VoxelPanelLeftTab::Sockets => {
                ui.heading("Sockets");
                ui.small("Edit socket placement and compatibility without the rest of the panel metadata in the way.");
                if let Some(panel) = self.voxel_panel_designer.selected_panel_mut() {
                    ui.horizontal(|ui| {
                        if ui.button("Add socket").clicked() {
                            add_socket = true;
                        }
                        if ui.button("Remove selected").clicked() {
                            remove_socket = true;
                        }
                    });

                    selected_socket_index =
                        selected_socket_index.min(panel.sockets.len().saturating_sub(1));
                    egui::ScrollArea::vertical()
                        .id_salt("voxel_panel_socket_list")
                        .max_height(160.0)
                        .show(ui, |ui| {
                            for (index, socket) in panel.sockets.iter().enumerate() {
                                let label = format!(
                                    "{} · {} · {},{},{}",
                                    socket.id, socket.edge, socket.x, socket.y, socket.z
                                );
                                ui.selectable_value(&mut selected_socket_index, index, label);
                            }
                        });

                    if let Some(socket) = panel.sockets.get_mut(selected_socket_index) {
                        ui.separator();
                        ui.label("Selected socket");
                        ui.horizontal(|ui| {
                            ui.label("ID");
                            panel_changed |= ui.text_edit_singleline(&mut socket.id).changed();
                        });
                        egui::ComboBox::from_id_salt("voxel_panel_socket_edge")
                            .selected_text(socket.edge.as_str())
                            .show_ui(ui, |ui| {
                                for edge in
                                    ["north", "south", "east", "west", "top", "bottom", "center"]
                                {
                                    panel_changed |= ui
                                        .selectable_value(&mut socket.edge, edge.to_string(), edge)
                                        .changed();
                                }
                            });
                        panel_changed |= ui
                            .add(
                                egui::Slider::new(&mut socket.x, 0..=panel.width.saturating_sub(1))
                                    .text("Socket X"),
                            )
                            .changed();
                        panel_changed |= ui
                            .add(
                                egui::Slider::new(
                                    &mut socket.y,
                                    0..=panel.height.saturating_sub(1),
                                )
                                .text("Socket Y"),
                            )
                            .changed();
                        panel_changed |= ui
                            .add(
                                egui::Slider::new(&mut socket.z, 0..=panel.depth.saturating_sub(1))
                                    .text("Socket Z"),
                            )
                            .changed();
                        panel_changed |= ui.checkbox(&mut socket.required, "Required").changed();
                        let mut accepts = socket.accepts.join(", ");
                        ui.horizontal(|ui| {
                            ui.label("Accepts");
                            if ui.text_edit_singleline(&mut accepts).changed() {
                                socket.accepts = voxel_panel_parse_list(&accepts);
                                panel_changed = true;
                            }
                        });
                    }
                }
            }
            VoxelPanelLeftTab::Compositions => {
                self.draw_voxel_panel_composition_scene_panel(ui);
            }
        }

        self.voxel_panel_designer.selected_socket_index = selected_socket_index;

        if add_socket {
            let mut new_socket_index = None;
            if let Some(panel) = self.voxel_panel_designer.selected_panel_mut() {
                let index = panel.sockets.len() + 1;
                panel.sockets.push(VoxelPanelSocketDef {
                    id: format!("socket_{index:02}"),
                    edge: "center".to_string(),
                    x: panel.width / 2,
                    y: panel.height / 2,
                    z: active_depth.min(panel.depth.saturating_sub(1)),
                    accepts: vec![panel.panel_kind.clone()],
                    required: false,
                });
                new_socket_index = Some(panel.sockets.len().saturating_sub(1));
                panel_changed = true;
            }
            if let Some(index) = new_socket_index {
                self.voxel_panel_designer.selected_socket_index = index;
            }
        }
        if remove_socket {
            let requested_remove_index = self.voxel_panel_designer.selected_socket_index;
            let mut next_socket_index = None;
            if let Some(panel) = self.voxel_panel_designer.selected_panel_mut() {
                if !panel.sockets.is_empty() {
                    let remove_index =
                        requested_remove_index.min(panel.sockets.len().saturating_sub(1));
                    panel.sockets.remove(remove_index);
                    next_socket_index =
                        Some(remove_index.min(panel.sockets.len().saturating_sub(1)));
                    panel_changed = true;
                }
            }
            if let Some(index) = next_socket_index {
                self.voxel_panel_designer.selected_socket_index = index;
            }
        }

        if panel_changed {
            self.voxel_panel_designer.dirty = true;
            self.voxel_panel_designer.normalize_selection();
        }
    }

    fn draw_voxel_panel_composition_scene_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Compositions");
        ui.small("Place authored panels into reusable assemblies before the future 3D viewport consumes them.");

        let mut next_composition = self.voxel_panel_designer.selected_composition_index;
        egui::ScrollArea::vertical()
            .id_salt("voxel_panel_composition_scene_list")
            .max_height(90.0)
            .show(ui, |ui| {
                for (index, composition) in self
                    .voxel_panel_designer
                    .kit
                    .compositions
                    .iter()
                    .enumerate()
                {
                    let label = format!(
                        "{} · {} instance(s) · {} connection(s)",
                        composition.id,
                        composition.instances.len(),
                        composition.connections.len()
                    );
                    ui.selectable_value(&mut next_composition, index, label);
                }
            });
        if next_composition != self.voxel_panel_designer.selected_composition_index {
            self.voxel_panel_designer.selected_composition_index = next_composition;
            self.voxel_panel_designer
                .selected_composition_instance_index = 0;
            self.voxel_panel_designer
                .selected_composition_connection_index = 0;
            self.voxel_panel_designer.normalize_selection();
        }

        ui.horizontal(|ui| {
            if ui.button("Add composition").clicked() {
                let index = self.voxel_panel_designer.kit.compositions.len() + 1;
                let mut composition = default_voxel_panel_composition_scene();
                composition.id = format!("composition_{index:02}");
                composition.display_name = format!("Composition {index:02}");
                composition.instances.clear();
                self.voxel_panel_designer.kit.compositions.push(composition);
                self.voxel_panel_designer.selected_composition_index = self
                    .voxel_panel_designer
                    .kit
                    .compositions
                    .len()
                    .saturating_sub(1);
                self.voxel_panel_designer
                    .selected_composition_instance_index = 0;
                self.voxel_panel_designer
                    .selected_composition_connection_index = 0;
                self.voxel_panel_designer.dirty = true;
            }
            if ui.button("Duplicate composition").clicked() {
                if let Some(composition) = self.voxel_panel_designer.selected_composition().cloned()
                {
                    let mut clone = composition;
                    clone.id = format!("{}_copy", clone.id);
                    clone.display_name = format!("{} Copy", clone.display_name);
                    self.voxel_panel_designer.kit.compositions.push(clone);
                    self.voxel_panel_designer.selected_composition_index = self
                        .voxel_panel_designer
                        .kit
                        .compositions
                        .len()
                        .saturating_sub(1);
                    self.voxel_panel_designer.dirty = true;
                }
            }
        });

        let mut composition_changed = false;
        if let Some(composition) = self.voxel_panel_designer.selected_composition_mut() {
            ui.horizontal(|ui| {
                ui.label("ID");
                composition_changed |= ui.text_edit_singleline(&mut composition.id).changed();
            });
            ui.horizontal(|ui| {
                ui.label("Name");
                composition_changed |= ui
                    .text_edit_singleline(&mut composition.display_name)
                    .changed();
            });
            composition_changed |= ui
                .add(egui::Slider::new(&mut composition.canvas_width, 8..=256).text("Canvas W"))
                .changed();
            composition_changed |= ui
                .add(egui::Slider::new(&mut composition.canvas_height, 8..=256).text("Canvas H"))
                .changed();
            composition_changed |= ui
                .add(egui::Slider::new(&mut composition.canvas_depth, 1..=64).text("Canvas D"))
                .changed();
            composition_changed |= ui
                .add(egui::Slider::new(&mut composition.grid_unit_px, 1..=16).text("Grid unit px"))
                .changed();

            ui.collapsing("Viewport prep / Phase 53i mesh export", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Source axis");
                    composition_changed |= ui
                        .text_edit_singleline(&mut composition.viewport_prep.source_axis)
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Bake anchor");
                    composition_changed |= ui
                        .text_edit_singleline(&mut composition.viewport_prep.bake_anchor)
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Export folder");
                    composition_changed |= ui
                        .text_edit_singleline(&mut composition.viewport_prep.mesh_export_path)
                        .changed();
                });
                composition_changed |= ui
                    .add(
                        egui::Slider::new(&mut composition.viewport_prep.bake_layer_gap, 0.0..=8.0)
                            .text("Bake layer gap"),
                    )
                    .changed();
                composition_changed |= ui
                    .checkbox(&mut composition.viewport_prep.show_bounds, "Show bounds")
                    .changed();
                composition_changed |= ui
                    .checkbox(
                        &mut composition.viewport_prep.show_socket_lines,
                        "Show socket lines",
                    )
                    .changed();
                composition_changed |= ui
                    .checkbox(
                        &mut composition.viewport_prep.include_empty_bounds,
                        "Include empty instance bounds in export",
                    )
                    .changed();
                composition_changed |= ui
                    .checkbox(
                        &mut composition.viewport_prep.emit_socket_gizmos,
                        "Emit socket gizmos",
                    )
                    .changed();
            });
        }

        ui.separator();
        ui.label("Instances");
        let mut next_instance = self
            .voxel_panel_designer
            .selected_composition_instance_index;
        if let Some(composition) = self.voxel_panel_designer.selected_composition() {
            egui::ScrollArea::vertical()
                .id_salt("voxel_panel_composition_instance_list")
                .max_height(80.0)
                .show(ui, |ui| {
                    for (index, instance) in composition.instances.iter().enumerate() {
                        let label = format!(
                            "{} · {} @ {},{},{}",
                            instance.id, instance.panel_id, instance.x, instance.y, instance.z
                        );
                        ui.selectable_value(&mut next_instance, index, label);
                    }
                });
        }
        self.voxel_panel_designer
            .selected_composition_instance_index = next_instance;

        ui.horizontal(|ui| {
            if ui.button("Add selected panel").clicked() {
                if let Some(instance_id) = self
                    .voxel_panel_designer
                    .add_selected_panel_instance_to_composition()
                {
                    self.status = format!("Added composition instance {instance_id}.");
                }
            }
            if ui.button("Remove instance").clicked() {
                if let Some(instance_id) = self
                    .voxel_panel_designer
                    .remove_selected_composition_instance()
                {
                    self.status = format!("Removed composition instance {instance_id}.");
                }
            }
        });

        if let Some(instance) = self
            .voxel_panel_designer
            .selected_composition_instance_mut()
        {
            ui.collapsing("Selected instance", |ui| {
                ui.horizontal(|ui| {
                    ui.label("ID");
                    composition_changed |= ui.text_edit_singleline(&mut instance.id).changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Panel");
                    composition_changed |=
                        ui.text_edit_singleline(&mut instance.panel_id).changed();
                });
                composition_changed |= ui
                    .add(egui::Slider::new(&mut instance.x, 0..=256).text("X"))
                    .changed();
                composition_changed |= ui
                    .add(egui::Slider::new(&mut instance.y, 0..=256).text("Y"))
                    .changed();
                composition_changed |= ui
                    .add(egui::Slider::new(&mut instance.z, 0..=64).text("Z"))
                    .changed();
                composition_changed |= ui
                    .add(
                        egui::Slider::new(&mut instance.rotation_degrees, 0..=270)
                            .step_by(90.0)
                            .text("Rotation"),
                    )
                    .changed();
                composition_changed |= ui.checkbox(&mut instance.mirror_x, "Mirror X").changed();
                composition_changed |= ui.checkbox(&mut instance.mirror_y, "Mirror Y").changed();
                composition_changed |= ui.checkbox(&mut instance.locked, "Locked").changed();
            });
        }

        ui.horizontal(|ui| {
            if ui.button("Snap selected to socket").clicked() {
                match self
                    .voxel_panel_designer
                    .snap_selected_instance_to_nearest_socket()
                {
                    Ok(message) => self.status = message,
                    Err(message) => self.status = message,
                }
            }
            if ui.button("Export 3D preview RON").clicked() {
                let project_root = self.project_root.clone();
                match self
                    .voxel_panel_designer
                    .export_selected_composition_mesh_preview(&project_root)
                {
                    Ok(path) => {
                        self.status =
                            format!("Exported Phase 53i 3D preview data: {}", path.display());
                        self.log(self.status.clone());
                    }
                    Err(error) => {
                        self.status = "Failed to export Phase 53i 3D preview data.".to_string();
                        self.log(format!("Voxel panel preview export error: {error:#}"));
                    }
                }
            }
            if ui.button("Load 3D preview RON").clicked() {
                let project_root = self.project_root.clone();
                match self
                    .voxel_panel_designer
                    .load_selected_composition_mesh_preview(&project_root)
                {
                    Ok(path) => {
                        self.status =
                            format!("Loaded Phase 53i 3D preview data: {}", path.display());
                        self.log(self.status.clone());
                    }
                    Err(error) => {
                        self.status = "Failed to load Phase 53i 3D preview data.".to_string();
                        self.log(format!("Voxel panel preview load error: {error:#}"));
                    }
                }
            }
            if let Some(composition) = self.voxel_panel_designer.selected_composition() {
                ui.label(format!("Connections: {}", composition.connections.len()));
            }
        });

        if let Some(summary) = &self.voxel_panel_designer.last_mesh_export_summary {
            ui.small(summary);
        }
        if let Some(path) = &self.voxel_panel_designer.last_mesh_export_path {
            ui.small(format!("Last export: {}", path.display()));
        }

        if composition_changed {
            self.voxel_panel_designer.dirty = true;
            self.voxel_panel_designer.normalize_selection();
        }
    }

    fn draw_voxel_panel_canvas_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("2D voxel-pixel slice");
        let Some(panel) = self.voxel_panel_designer.selected_panel().cloned() else {
            ui.label("No panel selected.");
            return;
        };

        let max_depth = panel.depth.saturating_sub(1);
        ui.add(
            egui::Slider::new(&mut self.voxel_panel_designer.active_depth, 0..=max_depth)
                .text("Depth layer Z"),
        );
        self.voxel_panel_designer.active_depth =
            self.voxel_panel_designer.active_depth.min(max_depth);

        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Panel {} · {}x{}x{}",
                panel.id, panel.width, panel.height, panel.depth
            ));
            if let Some((x, y)) = self.voxel_panel_designer.hover_cell {
                ui.label(format!(
                    "Hover: {},{},{}",
                    x, y, self.voxel_panel_designer.active_depth
                ));
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(
                &mut self.voxel_panel_designer.transform_active_depth_only,
                "Transform active layer only",
            );
            if ui.button("Copy layer").clicked() {
                let count = self.voxel_panel_designer.copy_active_depth_cells();
                self.status = format!(
                    "Copied {count} voxel-pixel cell(s) from depth layer {}.",
                    self.voxel_panel_designer.active_depth
                );
            }
            if ui.button("Paste layer").clicked() {
                self.voxel_panel_designer.push_panel_undo();
                let count = self.voxel_panel_designer.paste_cells_to_active_depth();
                self.status = format!(
                    "Pasted {count} voxel-pixel cell(s) into depth layer {}.",
                    self.voxel_panel_designer.active_depth
                );
            }
            if ui.button("Clear layer").clicked() {
                self.voxel_panel_designer.push_panel_undo();
                let count = self.voxel_panel_designer.clear_active_depth_cells();
                self.status = format!(
                    "Cleared {count} voxel-pixel cell(s) from depth layer {}.",
                    self.voxel_panel_designer.active_depth
                );
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Clipboard: {} cell(s)",
                self.voxel_panel_designer.clipboard_cells.len()
            ));
            if ui.button("Mirror X").clicked() {
                self.voxel_panel_designer.push_panel_undo();
                let count = self.voxel_panel_designer.mirror_cells_x();
                self.status = format!("Mirrored {count} voxel-pixel cell(s) on X.");
            }
            if ui.button("Mirror Y").clicked() {
                self.voxel_panel_designer.push_panel_undo();
                let count = self.voxel_panel_designer.mirror_cells_y();
                self.status = format!("Mirrored {count} voxel-pixel cell(s) on Y.");
            }
            if ui.button("Rotate CW").clicked() {
                self.voxel_panel_designer.push_panel_undo();
                match self.voxel_panel_designer.rotate_cells_cw() {
                    Ok(count) => {
                        self.status = format!("Rotated {count} voxel-pixel cell(s) clockwise.")
                    }
                    Err(message) => self.status = message,
                }
            }
            if ui.button("Rotate CCW").clicked() {
                self.voxel_panel_designer.push_panel_undo();
                match self.voxel_panel_designer.rotate_cells_ccw() {
                    Ok(count) => {
                        self.status =
                            format!("Rotated {count} voxel-pixel cell(s) counter-clockwise.")
                    }
                    Err(message) => self.status = message,
                }
            }
        });

        let available = ui.available_width().clamp(240.0, 640.0);
        let cell_size = (available / panel.width.max(1) as f32)
            .floor()
            .clamp(5.0, 28.0);
        let canvas_size = egui::vec2(
            panel.width.max(1) as f32 * cell_size,
            panel.height.max(1) as f32 * cell_size,
        );
        let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(24, 27, 33));

        let z = self.voxel_panel_designer.active_depth;
        for cell in panel.cells.iter().filter(|cell| cell.z == z) {
            let cell_rect = voxel_panel_cell_rect(rect, cell_size, cell.x, cell.y);
            painter.rect_filled(
                cell_rect.shrink(1.0),
                1.5,
                self.voxel_panel_designer.material_color(&cell.material_id),
            );
        }

        let grid_stroke = egui::Stroke::new(
            0.5,
            egui::Color32::from_rgba_unmultiplied(95, 110, 125, 130),
        );
        for x in 0..=panel.width {
            let px = rect.left() + x as f32 * cell_size;
            painter.line_segment(
                [egui::pos2(px, rect.top()), egui::pos2(px, rect.bottom())],
                grid_stroke,
            );
        }
        for y in 0..=panel.height {
            let py = rect.top() + y as f32 * cell_size;
            painter.line_segment(
                [egui::pos2(rect.left(), py), egui::pos2(rect.right(), py)],
                grid_stroke,
            );
        }

        for socket in panel.sockets.iter().filter(|socket| socket.z == z) {
            let cell_rect = voxel_panel_cell_rect(rect, cell_size, socket.x, socket.y);
            painter.rect_stroke(
                cell_rect.shrink(1.0),
                1.5,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 216, 96)),
                egui::StrokeKind::Inside,
            );
        }

        let mut action_status: Option<String> = None;
        if let Some(pos) = response.interact_pointer_pos() {
            if rect.contains(pos) {
                let x = ((pos.x - rect.left()) / cell_size).floor() as u32;
                let y = ((pos.y - rect.top()) / cell_size).floor() as u32;
                if x < panel.width && y < panel.height {
                    self.voxel_panel_designer.hover_cell = Some((x, y));
                    let should_apply = response.clicked() || response.dragged();
                    let stroke_start = response.drag_started() || response.clicked();
                    if should_apply {
                        match self.voxel_panel_designer.tool {
                            VoxelPanelTool::Paint => {
                                let material_id =
                                    self.voxel_panel_designer.selected_material_id.clone();
                                if stroke_start {
                                    self.voxel_panel_designer.push_panel_undo();
                                }
                                if self
                                    .voxel_panel_designer
                                    .set_cell(x, y, z, material_id.clone())
                                {
                                    action_status = Some(format!(
                                        "Painted {},{},{} with {}.",
                                        x, y, z, material_id
                                    ));
                                }
                            }
                            VoxelPanelTool::Erase => {
                                if stroke_start {
                                    self.voxel_panel_designer.push_panel_undo();
                                }
                                if self.voxel_panel_designer.erase_cell(x, y, z) {
                                    action_status =
                                        Some(format!("Erased voxel-pixel {},{},{}.", x, y, z));
                                }
                            }
                            VoxelPanelTool::Pick => {
                                if self.voxel_panel_designer.pick_cell(x, y, z) {
                                    action_status =
                                        Some(format!("Picked material at {},{},{}.", x, y, z));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(status) = action_status {
            self.status = status;
        }

        ui.small("This is an authoring grid for one voxel depth slice. The future 3D preview/editor will consume the same RON kit data.");
    }

    fn draw_voxel_panel_composition_canvas_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Composition canvas");
        let Some(composition) = self.voxel_panel_designer.selected_composition().cloned() else {
            ui.label("No composition selected.");
            return;
        };

        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "{} · {}x{}x{}",
                composition.id,
                composition.canvas_width,
                composition.canvas_height,
                composition.canvas_depth
            ));
            ui.add(
                egui::Slider::new(
                    &mut self.voxel_panel_designer.composition_canvas_zoom,
                    0.5..=3.0,
                )
                .text("Zoom"),
            );
            if let Some(instance) = self.voxel_panel_designer.selected_composition_instance() {
                ui.label(format!("Selected: {}", instance.id));
            }
        });

        let available = ui.available_width().clamp(280.0, 760.0);
        let base_cell_size = (available / composition.canvas_width.max(1) as f32)
            .floor()
            .clamp(3.0, 14.0);
        let cell_size =
            (base_cell_size * self.voxel_panel_designer.composition_canvas_zoom).clamp(3.0, 24.0);
        let canvas_size = egui::vec2(
            composition.canvas_width.max(1) as f32 * cell_size,
            composition.canvas_height.max(1) as f32 * cell_size,
        );
        let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(19, 22, 28));

        let grid_stroke =
            egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(80, 96, 112, 100));
        let major_grid_stroke = egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(110, 130, 150, 135),
        );
        let grid_unit = composition.grid_unit_px.max(1);
        for x in 0..=composition.canvas_width {
            let px = rect.left() + x as f32 * cell_size;
            let stroke = if x % grid_unit == 0 {
                major_grid_stroke
            } else {
                grid_stroke
            };
            painter.line_segment(
                [egui::pos2(px, rect.top()), egui::pos2(px, rect.bottom())],
                stroke,
            );
        }
        for y in 0..=composition.canvas_height {
            let py = rect.top() + y as f32 * cell_size;
            let stroke = if y % grid_unit == 0 {
                major_grid_stroke
            } else {
                grid_stroke
            };
            painter.line_segment(
                [egui::pos2(rect.left(), py), egui::pos2(rect.right(), py)],
                stroke,
            );
        }

        if composition.viewport_prep.show_socket_lines {
            for connection in &composition.connections {
                let Some(from_instance) = composition
                    .instances
                    .iter()
                    .find(|instance| instance.id == connection.from_instance)
                else {
                    continue;
                };
                let Some(to_instance) = composition
                    .instances
                    .iter()
                    .find(|instance| instance.id == connection.to_instance)
                else {
                    continue;
                };
                let Some(from_panel) = self
                    .voxel_panel_designer
                    .panel_by_id(&from_instance.panel_id)
                else {
                    continue;
                };
                let Some(to_panel) = self.voxel_panel_designer.panel_by_id(&to_instance.panel_id)
                else {
                    continue;
                };
                let Some(from_socket) = from_panel
                    .sockets
                    .iter()
                    .find(|socket| socket.id == connection.from_socket)
                else {
                    continue;
                };
                let Some(to_socket) = to_panel
                    .sockets
                    .iter()
                    .find(|socket| socket.id == connection.to_socket)
                else {
                    continue;
                };
                let from_world =
                    voxel_panel_socket_world_position(from_panel, from_instance, from_socket);
                let to_world = voxel_panel_socket_world_position(to_panel, to_instance, to_socket);
                let from_pos = egui::pos2(
                    rect.left() + (from_world.0 as f32 + 0.5) * cell_size,
                    rect.top() + (from_world.1 as f32 + 0.5) * cell_size,
                );
                let to_pos = egui::pos2(
                    rect.left() + (to_world.0 as f32 + 0.5) * cell_size,
                    rect.top() + (to_world.1 as f32 + 0.5) * cell_size,
                );
                let stroke = if connection.snapped {
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(126, 196, 137))
                } else {
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(255, 180, 84))
                };
                painter.line_segment([from_pos, to_pos], stroke);
            }
        }

        let mut hit_instance_index = None;
        for (index, instance) in composition.instances.iter().enumerate() {
            let Some(panel) = self.voxel_panel_designer.panel_by_id(&instance.panel_id) else {
                continue;
            };
            let instance_rect =
                voxel_panel_composition_instance_rect(rect, cell_size, panel, instance);
            let selected = index
                == self
                    .voxel_panel_designer
                    .selected_composition_instance_index;
            painter.rect_filled(
                instance_rect,
                3.0,
                egui::Color32::from_rgba_unmultiplied(42, 48, 58, if selected { 245 } else { 210 }),
            );

            for cell in panel.cells.iter().filter(|cell| cell.z == 0) {
                let cell_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        instance_rect.left() + cell.x as f32 * cell_size,
                        instance_rect.top() + cell.y as f32 * cell_size,
                    ),
                    egui::vec2(cell_size, cell_size),
                );
                painter.rect_filled(
                    cell_rect.shrink(1.0),
                    1.0,
                    self.voxel_panel_designer.material_color(&cell.material_id),
                );
            }

            let stroke = if selected {
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 216, 96))
            } else {
                egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgba_unmultiplied(145, 165, 185, 170),
                )
            };
            if composition.viewport_prep.show_bounds {
                painter.rect_stroke(instance_rect, 3.0, stroke, egui::StrokeKind::Inside);
            }

            for socket in &panel.sockets {
                let world = voxel_panel_socket_world_position(panel, instance, socket);
                let center = egui::pos2(
                    rect.left() + (world.0 as f32 + 0.5) * cell_size,
                    rect.top() + (world.1 as f32 + 0.5) * cell_size,
                );
                painter.circle_filled(
                    center,
                    (cell_size * 0.25).clamp(2.0, 5.0),
                    egui::Color32::from_rgb(255, 216, 96),
                );
            }

            painter.text(
                instance_rect.left_top() + egui::vec2(4.0, 4.0),
                egui::Align2::LEFT_TOP,
                &instance.id,
                egui::FontId::monospace(10.0),
                egui::Color32::from_rgb(230, 236, 244),
            );

            if let Some(pos) = response.interact_pointer_pos() {
                if instance_rect.contains(pos) {
                    hit_instance_index = Some(index);
                }
            }
        }

        if response.clicked() || response.drag_started() {
            if let Some(index) = hit_instance_index {
                self.voxel_panel_designer
                    .selected_composition_instance_index = index;
            }
        }
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let x = ((pos.x - rect.left()) / cell_size).round() as i32;
                let y = ((pos.y - rect.top()) / cell_size).round() as i32;
                if self
                    .voxel_panel_designer
                    .move_selected_composition_instance_to(x, y)
                {
                    self.status = format!("Moved composition instance to {x},{y}.");
                }
            }
        }

        ui.small("Click to select an instance, drag to reposition it on the 2D composition grid, then use socket snapping to create validated connections.");
    }

    fn draw_voxel_panel_3d_preview_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("3D Preview");
        ui.small("Phase 53k read-only inspection preview: exported Phase 53i RON with grid, axis, material legend, hover labels, selection highlights, and diagnostics.");

        ui.horizontal_wrapped(|ui| {
            if ui.button("Export + load").clicked() {
                let project_root = self.project_root.clone();
                match self
                    .voxel_panel_designer
                    .export_selected_composition_mesh_preview(&project_root)
                {
                    Ok(path) => {
                        self.status =
                            format!("Exported and loaded Phase 53k preview: {}", path.display());
                        self.log(self.status.clone());
                    }
                    Err(error) => {
                        self.status = "Failed to export Phase 53k preview.".to_string();
                        self.log(format!("Voxel panel 3D preview export error: {error:#}"));
                    }
                }
            }
            if ui.button("Load expected RON").clicked() {
                let project_root = self.project_root.clone();
                match self
                    .voxel_panel_designer
                    .load_selected_composition_mesh_preview(&project_root)
                {
                    Ok(path) => {
                        self.status = format!("Loaded Phase 53i preview RON: {}", path.display());
                        self.log(self.status.clone());
                    }
                    Err(error) => {
                        self.status = "Failed to load Phase 53i preview RON.".to_string();
                        self.log(format!("Voxel panel 3D preview load error: {error:#}"));
                    }
                }
            }
            if ui.button("Refresh export list").clicked() {
                let project_root = self.project_root.clone();
                self.voxel_panel_designer
                    .refresh_preview_export_history(&project_root);
                self.status = format!(
                    "Found {} voxel preview export file(s).",
                    self.voxel_panel_designer.preview_export_paths.len()
                );
            }
            if ui.button("Reset camera").clicked() {
                self.voxel_panel_designer.preview_camera = VoxelPanelPreviewCameraState::default();
            }
        });

        let export_paths = self.voxel_panel_designer.preview_export_paths.clone();
        if !export_paths.is_empty() {
            let mut selected_index = self
                .voxel_panel_designer
                .selected_preview_export_index
                .min(export_paths.len().saturating_sub(1));
            ui.horizontal_wrapped(|ui| {
                egui::ComboBox::from_id_salt("voxel_preview_export_history")
                    .selected_text(
                        export_paths
                            .get(selected_index)
                            .and_then(|path| path.file_name())
                            .and_then(|name| name.to_str())
                            .unwrap_or("<preview export>"),
                    )
                    .show_ui(ui, |ui| {
                        for (index, path) in export_paths.iter().enumerate() {
                            let label = path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("<preview export>");
                            ui.selectable_value(&mut selected_index, index, label);
                        }
                    });
                self.voxel_panel_designer.selected_preview_export_index = selected_index;
                if ui.button("Load selected export").clicked() {
                    if let Some(path) = export_paths.get(selected_index).cloned() {
                        match self
                            .voxel_panel_designer
                            .load_mesh_preview_from_path(path.clone())
                        {
                            Ok(path) => {
                                self.status =
                                    format!("Loaded selected preview export: {}", path.display());
                                self.log(self.status.clone());
                            }
                            Err(error) => {
                                self.status = "Failed to load selected preview export.".to_string();
                                self.log(format!(
                                    "Voxel panel preview history load error: {error:#}"
                                ));
                            }
                        }
                    }
                }
            });
        }

        let camera = &mut self.voxel_panel_designer.preview_camera;
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::Slider::new(&mut camera.yaw_degrees, -180.0..=180.0).text("Orbit yaw"));
            ui.add(egui::Slider::new(&mut camera.pitch_degrees, -80.0..=80.0).text("Pitch"));
            ui.add(egui::Slider::new(&mut camera.zoom, 4.0..=56.0).text("Zoom"));
        });
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::DragValue::new(&mut camera.pan_x)
                    .speed(1.0)
                    .prefix("Pan X "),
            );
            ui.add(
                egui::DragValue::new(&mut camera.pan_y)
                    .speed(1.0)
                    .prefix("Pan Y "),
            );
        });
        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut camera.show_voxels, "Voxels");
            ui.checkbox(&mut camera.show_bounds, "Bounds");
            ui.checkbox(&mut camera.show_floor_grid, "Floor grid");
            ui.checkbox(&mut camera.show_axis_gizmo, "XYZ axis");
            ui.checkbox(&mut camera.show_socket_gizmos, "Sockets");
            ui.checkbox(&mut camera.show_connection_gizmos, "Connections");
            ui.checkbox(&mut camera.show_labels, "Labels");
            ui.checkbox(&mut camera.show_hover_labels, "Hover labels");
            ui.checkbox(&mut camera.show_material_legend, "Materials");
            ui.checkbox(&mut camera.show_diagnostics, "Diagnostics");
        });

        let Some(export) = self.voxel_panel_designer.preview_3d_export.clone() else {
            ui.label("No Phase 53i preview RON loaded yet.");
            if let Some(path) = self
                .voxel_panel_designer
                .mesh_export_path_for_selected_composition(&self.project_root)
                .ok()
            {
                ui.small(format!("Expected preview path: {}", path.display()));
            }
            return;
        };

        if let Some(path) = &self.voxel_panel_designer.preview_3d_export_path {
            ui.small(format!("Loaded: {}", path.display()));
        }
        ui.horizontal_wrapped(|ui| {
            ui.small(format!(
                "{} voxel(s) · {} instance(s) · {} socket gizmo(s) · {} connection gizmo(s)",
                export.voxel_count,
                export.instance_count,
                export.socket_gizmo_count,
                export.connection_gizmo_count
            ));
            ui.small(format!(
                "bounds {:?}..{:?} · voxel_unit {:.2} · layer_gap {:.2}",
                export.bounds_min, export.bounds_max, export.voxel_unit, export.layer_gap
            ));
        });

        let viewport_width = ui.available_width().max(260.0);
        let viewport_height = 360.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(viewport_width, viewport_height),
            egui::Sense::click_and_drag(),
        );

        if response.dragged() {
            let delta = response.drag_delta();
            let camera = &mut self.voxel_panel_designer.preview_camera;
            camera.yaw_degrees = (camera.yaw_degrees + delta.x * 0.08).clamp(-180.0, 180.0);
            camera.pitch_degrees = (camera.pitch_degrees - delta.y * 0.08).clamp(-80.0, 80.0);
        }

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 5.0, egui::Color32::from_rgb(15, 18, 23));
        painter.rect_stroke(
            rect,
            5.0,
            egui::Stroke::new(
                1.0,
                egui::Color32::from_rgba_unmultiplied(95, 112, 130, 150),
            ),
            egui::StrokeKind::Inside,
        );

        voxel_panel_draw_preview_grid(&painter, rect);
        let selected_instance_id = self
            .voxel_panel_designer
            .selected_composition_instance()
            .map(|instance| instance.id.as_str());
        let selected_socket_key = self
            .voxel_panel_designer
            .selected_composition_instance()
            .and_then(|instance| {
                self.voxel_panel_designer
                    .panel_by_id(&instance.panel_id)
                    .and_then(|panel| {
                        panel
                            .sockets
                            .get(self.voxel_panel_designer.selected_socket_index)
                    })
                    .map(|socket| (instance.id.as_str(), socket.id.as_str()))
            });
        voxel_panel_draw_3d_preview(
            &painter,
            rect,
            &export,
            &self.voxel_panel_designer.preview_camera,
            selected_instance_id,
            selected_socket_key,
        );

        let hover_pos = ui
            .input(|input| input.pointer.hover_pos())
            .filter(|position| rect.contains(*position));
        if self.voxel_panel_designer.preview_camera.show_hover_labels {
            if let Some(position) = hover_pos {
                if let Some(label) = voxel_panel_preview_hover_label(
                    &export,
                    &self.voxel_panel_designer.preview_camera,
                    rect,
                    position,
                ) {
                    let label_pos = position + egui::vec2(12.0, 12.0);
                    let label_rect = egui::Rect::from_min_size(
                        label_pos,
                        egui::vec2((label.len() as f32 * 6.2).clamp(180.0, 520.0), 24.0),
                    );
                    painter.rect_filled(
                        label_rect,
                        4.0,
                        egui::Color32::from_rgba_unmultiplied(24, 29, 36, 236),
                    );
                    painter.rect_stroke(
                        label_rect,
                        4.0,
                        egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_unmultiplied(168, 188, 212, 180),
                        ),
                        egui::StrokeKind::Inside,
                    );
                    painter.text(
                        label_pos + egui::vec2(7.0, 5.0),
                        egui::Align2::LEFT_TOP,
                        label,
                        egui::FontId::monospace(10.0),
                        egui::Color32::from_rgb(238, 244, 250),
                    );
                }
            }
        }

        painter.text(
            rect.left_top() + egui::vec2(8.0, 8.0),
            egui::Align2::LEFT_TOP,
            "drag = orbit · sliders = orbit/pan/zoom · yellow box = selected instance",
            egui::FontId::monospace(10.0),
            egui::Color32::from_rgba_unmultiplied(210, 220, 232, 190),
        );

        if self
            .voxel_panel_designer
            .preview_camera
            .show_material_legend
        {
            ui.collapsing("Material legend", |ui| {
                for (material_id, rgba, count) in
                    self.voxel_panel_designer.preview_material_counts(&export)
                {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("    ").background_color(
                            egui::Color32::from_rgba_unmultiplied(
                                rgba[0], rgba[1], rgba[2], rgba[3],
                            ),
                        ));
                        ui.label(format!("{} · {} voxel(s)", material_id, count));
                    });
                }
            });
        }

        if self.voxel_panel_designer.preview_camera.show_diagnostics {
            ui.collapsing("Preview diagnostics", |ui| {
                ui.small(format!(
                    "Stats: {} baked voxel record(s), {} baked instance record(s), {} socket gizmo(s), {} connection gizmo(s).",
                    export.voxels.len(),
                    export.instances.len(),
                    export.socket_gizmos.len(),
                    export.connection_gizmos.len()
                ));
                for message in self.voxel_panel_designer.preview_diagnostics_messages(&export) {
                    ui.label(format!("• {message}"));
                }
            });
        }
    }

    fn draw_voxel_panel_validation_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Palette / Validation");

        let mut selected_palette_index = self.voxel_panel_designer.selected_palette_index;
        egui::ComboBox::from_id_salt("voxel_panel_palette_selector")
            .selected_text(
                self.voxel_panel_designer
                    .selected_palette()
                    .map(|palette| palette.display_name.as_str())
                    .unwrap_or("<palette>"),
            )
            .show_ui(ui, |ui| {
                for (index, palette) in self.voxel_panel_designer.kit.palettes.iter().enumerate() {
                    ui.selectable_value(&mut selected_palette_index, index, &palette.display_name);
                }
            });

        if selected_palette_index != self.voxel_panel_designer.selected_palette_index {
            self.voxel_panel_designer.selected_palette_index = selected_palette_index;
            self.voxel_panel_designer.selected_material_id = self
                .voxel_panel_designer
                .selected_palette()
                .and_then(|palette| palette.materials.first())
                .map(|material| material.id.clone())
                .unwrap_or_default();
        }

        ui.separator();
        ui.label("Materials");

        let materials = self
            .voxel_panel_designer
            .selected_palette()
            .map(|palette| palette.materials.clone())
            .unwrap_or_default();

        egui::ScrollArea::vertical()
            .id_salt("voxel_panel_material_list")
            .max_height(210.0)
            .show(ui, |ui| {
                for material in materials {
                    let selected = self.voxel_panel_designer.selected_material_id == material.id;
                    ui.horizontal(|ui| {
                        let swatch = egui::RichText::new("    ").background_color(
                            egui::Color32::from_rgba_unmultiplied(
                                material.rgba[0],
                                material.rgba[1],
                                material.rgba[2],
                                material.rgba[3],
                            ),
                        );
                        ui.label(swatch);
                        if ui
                            .selectable_label(
                                selected,
                                format!("{} · {}", material.id, material.display_name),
                            )
                            .clicked()
                        {
                            self.voxel_panel_designer.selected_material_id = material.id.clone();
                        }
                    });
                    ui.small(format!("hint: {}", material.render_hint));
                }
            });

        ui.separator();
        ui.heading("Validation");

        let messages = self.voxel_panel_designer.validation_messages();
        if messages.is_empty() {
            ui.label("No voxel panel kit validation issues.");
        } else {
            egui::ScrollArea::vertical()
                .id_salt("voxel_panel_validation_messages")
                .max_height(240.0)
                .show(ui, |ui| {
                    for message in messages {
                        ui.label(format!("⚠ {message}"));
                    }
                });
        }
    }

    fn draw_voxels_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "VOX Models",
            "MagicaVoxel .vox files scanned from assets/voxels, assets/models, and content/voxels. Select a model to preview its voxel projection.",
        );

        ui.horizontal_wrapped(|ui| {
            if ui.button("Reload VOX assets").clicked() {
                self.loaded_vox_cache = None;
                match scan_vox_files(&self.project_root) {
                    Ok(assets) => {
                        self.vox_assets = assets;
                        self.selected_vox_index = self
                            .selected_vox_index
                            .min(self.vox_assets.len().saturating_sub(1));
                        self.status = format!("Scanned {} .vox asset(s).", self.vox_assets.len());
                        self.log(self.status.clone());
                    }
                    Err(error) => {
                        self.status = "VOX scan failed.".to_string();
                        self.log(format!("VOX scan error: {error:#}"));
                    }
                }
            }
            ui.label(format!("{} model(s) found", self.vox_assets.len()));
        });

        ui.separator();

        if self.vox_assets.is_empty() {
            ui.label("No .vox files found yet.");
            ui.label("Place MagicaVoxel assets in assets/voxels/, assets/models/, or content/voxels/, then click Reload VOX assets.");
            return;
        }

        ui.columns(2, |columns| {
            columns[0].heading("Discovered .vox files");
            let mut clicked_vox_index = None;
            egui::ScrollArea::vertical()
                .id_salt("vox_asset_list")
                .max_height(360.0)
                .show(&mut columns[0], |ui| {
                    for (index, asset) in self.vox_assets.iter().enumerate() {
                        let label = format!(
                            "{}  ·  {}x{}x{}  ·  {} voxels",
                            asset.id, asset.width, asset.height, asset.depth, asset.voxel_count
                        );
                        if ui
                            .selectable_label(self.selected_vox_index == index, label)
                            .clicked()
                        {
                            clicked_vox_index = Some(index);
                        }
                    }
                });
            if let Some(index) = clicked_vox_index {
                self.selected_vox_index = index;
                self.loaded_vox_cache = None;
                if let Some(asset) = self.vox_assets.get(index) {
                    self.status = format!("Selected VOX model {}.", asset.id);
                }
            }

            columns[1].heading("Selected model");
            if let Some(asset) = self.vox_assets.get(self.selected_vox_index).cloned() {
                egui::Grid::new("vox_model_details")
                    .striped(true)
                    .show(&mut columns[1], |ui| {
                        ui.label("ID");
                        ui.monospace(asset.id.as_str());
                        ui.end_row();
                        ui.label("Path");
                        ui.monospace(asset.relative_path.as_str());
                        ui.end_row();
                        ui.label("Size");
                        ui.label(format!("{} x {} x {}", asset.width, asset.height, asset.depth));
                        ui.end_row();
                        ui.label("Voxel count");
                        ui.label(asset.voxel_count.to_string());
                        ui.end_row();
                        ui.label("Palette colors");
                        ui.label(asset.palette_colors.to_string());
                        ui.end_row();
                    });

                columns[1].separator();

                let need_load = self
                    .loaded_vox_cache
                    .as_ref()
                    .map(|(id, _)| id != &asset.id)
                    .unwrap_or(true);
                if need_load {
                    match load_vox_file(&asset.absolute_path) {
                        Ok(model) => {
                            self.loaded_vox_cache = Some((asset.id.clone(), model));
                        }
                        Err(err) => {
                            columns[1].label(format!("Failed to load preview: {err:#}"));
                        }
                    }
                }

                if let Some((_, model)) = self.loaded_vox_cache.as_ref() {
                    let preview_size = egui::vec2(280.0, 280.0);
                    let (preview_rect, _) = columns[1].allocate_exact_size(
                        preview_size,
                        egui::Sense::hover(),
                    );
                    let painter = columns[1].painter_at(preview_rect);
                    painter.rect_filled(preview_rect, 4.0, egui::Color32::from_rgb(18, 22, 30));
                    draw_vox_isometric_preview(&painter, preview_rect, model);
                }
            }
        });
    }

    fn draw_voxel_generator_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Voxel Generator",
            "Generate and validate base character and tool .vox templates from built-in profiles.",
        );

        let profiles = voxel_generator::profiles::default_profiles();
        let project_root = self.project_root.clone();

        ui.horizontal_wrapped(|ui| {
            if ui.button("Generate All Templates").clicked() {
                match voxel_generator::generate_phase53b_templates(&project_root) {
                    Ok(paths) => {
                        let msg = format!("Generated {} .vox template(s).", paths.len());
                        self.status = msg.clone();
                        self.log(msg);
                        for path in &paths {
                            self.generator_log.push(format!("  ✓ {}", path.display()));
                        }
                        self.loaded_vox_cache = None;
                    }
                    Err(err) => {
                        let msg = format!("Generation failed: {err:#}");
                        self.status = msg.clone();
                        self.log(msg.clone());
                        self.generator_log.push(format!("  ✗ {msg}"));
                    }
                }
            }
            if ui.button("Clear Log").clicked() {
                self.generator_log.clear();
            }
        });

        ui.separator();

        ui.columns(2, |columns| {
            columns[0].heading("Generator Profiles");
            egui::ScrollArea::vertical()
                .id_salt("voxel_generator_profiles")
                .max_height(400.0)
                .show(&mut columns[0], |ui| {
                    for profile in &profiles {
                        let output_path = project_root.join(profile.output_path);
                        let exists = output_path.exists();
                        let file_size = if exists {
                            std::fs::metadata(&output_path)
                                .map(|m| format!("{} bytes", m.len()))
                                .unwrap_or_else(|_| "?".to_string())
                        } else {
                            "missing".to_string()
                        };
                        let status_color = if exists {
                            egui::Color32::from_rgb(100, 200, 120)
                        } else {
                            egui::Color32::from_rgb(200, 100, 80)
                        };
                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_rgb(22, 26, 35))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 60, 80)))
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.colored_label(
                                        status_color,
                                        if exists { "✓" } else { "✗" },
                                    );
                                    ui.strong(profile.id);
                                });
                                ui.small(format!(
                                    "Kind: {:?}  ·  Dims: {}×{}×{}",
                                    profile.generator_kind,
                                    profile.dimensions[0],
                                    profile.dimensions[1],
                                    profile.dimensions[2]
                                ));
                                ui.small(format!("Output: {}", profile.output_path));
                                ui.small(format!("File: {file_size}"));
                                if exists {
                                    if let Some(validation) = validate_vox_profile(
                                        &output_path,
                                        profile.dimensions,
                                    ) {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(220, 160, 60),
                                            format!("⚠ {validation}"),
                                        );
                                    } else {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(100, 200, 120),
                                            "Dimensions match profile.",
                                        );
                                    }
                                }
                                if ui.small_button("Generate this profile").clicked() {
                                    let model = voxel_generator::vox_writer::placeholder_model(
                                        profile.dimensions,
                                        profile.generator_kind,
                                    );
                                    if let Some(parent) = output_path.parent() {
                                        let _ = std::fs::create_dir_all(parent);
                                    }
                                    match std::fs::write(
                                        &output_path,
                                        voxel_generator::vox_writer::write_vox(&model)
                                            .unwrap_or_default(),
                                    ) {
                                        Ok(()) => {
                                            let msg = format!(
                                                "Generated {}.",
                                                profile.output_path
                                            );
                                            self.status = msg.clone();
                                            self.generator_log.push(format!("  ✓ {msg}"));
                                            self.loaded_vox_cache = None;
                                        }
                                        Err(err) => {
                                            let msg = format!(
                                                "Failed to write {}: {err:#}",
                                                profile.output_path
                                            );
                                            self.status = msg.clone();
                                            self.generator_log.push(format!("  ✗ {msg}"));
                                        }
                                    }
                                }
                            });
                        ui.add_space(2.0);
                    }
                });

            columns[1].heading("Generator Log");
            egui::ScrollArea::vertical()
                .id_salt("voxel_generator_log")
                .max_height(400.0)
                .stick_to_bottom(true)
                .show(&mut columns[1], |ui| {
                    if self.generator_log.is_empty() {
                        ui.label("No generator output yet. Click Generate All Templates to start.");
                    } else {
                        for line in &self.generator_log {
                            ui.small(line.as_str());
                        }
                    }
                });
        });
    }

    fn draw_pixel_editor_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Pixel Editor",
            "Phase 51e: undo/redo, selection, clipboard, paste preview, fill/line tools, brush shapes, and dither painting.",
        );
        self.draw_pixel_editor_toolbar(ui);
        ui.separator();
        self.draw_pixel_editor_canvas(ui);
    }

    fn draw_pixel_editor_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label("Tool:");
            for tool in PixelTool::ALL {
                ui.selectable_value(&mut self.pixel_editor.tool, tool, tool.label());
            }
        });

        ui.horizontal_wrapped(|ui| {
            if ui.button("Copy tile").clicked() {
                let tile_size = self.active_tile_size();
                if self.pixel_editor.copy_tile(self.selected_cell, tile_size) {
                    self.pixel_editor.tool = PixelTool::Paste;
                    self.status = format!(
                        "Copied selected atlas tile {},{}.",
                        self.selected_cell.0, self.selected_cell.1
                    );
                }
            }
            if ui.button("Copy selection Ctrl+C").clicked() {
                if self.pixel_editor.copy_selection() {
                    self.pixel_editor.tool = PixelTool::Paste;
                    self.status = "Copied pixel selection.".to_string();
                }
            }
            ui.small("Save/undo/redo in command strip");
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("Brush:");
            ui.add(egui::Slider::new(&mut self.pixel_editor.brush_size, 1..=15).text("Size"));
            ui.add(egui::Slider::new(&mut self.pixel_editor.zoom, 0.25..=16.0).text("Zoom"));
            for shape in BrushShape::ALL {
                ui.selectable_value(&mut self.pixel_editor.brush_shape, shape, shape.label());
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("RGBA:");
            ui.add(egui::Slider::new(&mut self.pixel_editor.primary_color[0], 0..=255).text("R"));
            ui.add(egui::Slider::new(&mut self.pixel_editor.primary_color[1], 0..=255).text("G"));
            ui.add(egui::Slider::new(&mut self.pixel_editor.primary_color[2], 0..=255).text("B"));
            ui.add(egui::Slider::new(&mut self.pixel_editor.primary_color[3], 0..=255).text("A"));
            let swatch = egui::RichText::new("     ").background_color(
                egui::Color32::from_rgba_unmultiplied(
                    self.pixel_editor.primary_color[0],
                    self.pixel_editor.primary_color[1],
                    self.pixel_editor.primary_color[2],
                    self.pixel_editor.primary_color[3],
                ),
            );
            ui.label(swatch);
        });

        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.pixel_editor.mirror_x, "Mirror X");
            ui.checkbox(&mut self.pixel_editor.mirror_y, "Mirror Y");
            ui.checkbox(&mut self.pixel_editor.flip_paste_x, "Flip paste X");
            ui.checkbox(&mut self.pixel_editor.flip_paste_y, "Flip paste Y");
            if ui.button("Rotate paste 90°").clicked() {
                self.pixel_editor.rotate_paste_quarters =
                    (self.pixel_editor.rotate_paste_quarters + 1) % 4;
            }
            ui.label(format!(
                "Paste rotation: {}°",
                self.pixel_editor.rotate_paste_quarters * 90
            ));
        });

        let clipboard_label = self
            .pixel_editor
            .clipboard
            .as_ref()
            .map(|clipboard| format!("Clipboard: {} x {}", clipboard.width, clipboard.height))
            .unwrap_or_else(|| "Clipboard: empty".to_string());
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Editing: {} | Image: {} x {} | {} | Undo: {} | Redo: {} | Dirty: {}",
                self.pixel_editor.image_path.display(),
                self.pixel_editor.width(),
                self.pixel_editor.height(),
                clipboard_label,
                self.pixel_editor.undo_stack.len(),
                self.pixel_editor.redo_stack.len(),
                if self.pixel_editor.dirty { "yes" } else { "no" },
            ));
        });
    }

    fn draw_pixel_editor_canvas(&mut self, ui: &mut egui::Ui) {
        self.pixel_editor.ensure_texture(ui.ctx());
        let Some(texture_id) = self
            .pixel_editor
            .texture
            .as_ref()
            .map(|texture| texture.id())
        else {
            ui.label("Pixel editor texture is not loaded.");
            return;
        };

        let image_width = self.pixel_editor.width().max(1) as f32;
        let image_height = self.pixel_editor.height().max(1) as f32;
        let available = ui.available_size_before_wrap();
        let max_width = available.x.max(360.0);
        let max_height = available.y.max(360.0);
        let fit = (max_width / image_width)
            .min(max_height / image_height)
            .max(0.25);
        let scale = (fit * self.pixel_editor.zoom).clamp(0.25, 32.0);
        let canvas_size = egui::vec2(image_width * scale, image_height * scale);

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
                self.pixel_editor.last_canvas_rect = Some(rect);
                let painter = ui.painter_at(rect);
                self.pixel_editor.paint_checkerboard(&painter, rect);
                painter.image(
                    texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                if let Some(message) = self.pixel_editor.handle_canvas_interaction(&response, rect)
                {
                    self.status = message;
                }
                if matches!(self.pixel_editor.tool, PixelTool::RectSelect)
                    && self.pixel_editor.normalized_selection().is_some()
                    && (response.drag_started() || response.dragged() || response.clicked())
                {
                    self.editor_selection = EditorSelection::PixelSelection;
                }

                self.pixel_editor.paint_overlays(
                    &painter,
                    rect,
                    self.selected_cell,
                    self.active_tile_size(),
                );

                if let Some((x, y)) = self.pixel_editor.hover_pixel {
                    response.on_hover_text(format!("Pixel {},{}", x, y));
                }
            });
    }

    fn active_tile_size(&self) -> (u32, u32) {
        self.active_tileset()
            .map(|tileset| (tileset.tile_width, tileset.tile_height))
            .unwrap_or((16, 16))
    }
    fn draw_animation_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Animation Editor",
            "Timeline, frame events, sockets, hitboxes, and directional groups.",
        );
        match self.animation_subtab {
            AnimationSubTab::Clips => self.draw_workspace_notes(
                ui,
                "Clips",
                &[
                    "Clip list",
                    "Direction groups",
                    "Clip duplicate/import/export",
                ],
            ),
            AnimationSubTab::Timeline => self.draw_workspace_notes(
                ui,
                "Timeline",
                &["Frame strip", "Playback controls", "Frame duration editing"],
            ),
            AnimationSubTab::Events => self.draw_workspace_notes(
                ui,
                "Events",
                &[
                    "Footstep events",
                    "SFX events",
                    "Tool attach and detach frames",
                ],
            ),
            AnimationSubTab::Sockets => self.draw_workspace_notes(
                ui,
                "Sockets",
                &[
                    "Tool sockets",
                    "Attachment anchors",
                    "Per-frame socket offsets",
                ],
            ),
            AnimationSubTab::Hitboxes => self.draw_workspace_notes(
                ui,
                "Hitboxes",
                &[
                    "Hitbox preview",
                    "Interaction boxes",
                    "Collision event validation",
                ],
            ),
            AnimationSubTab::SeasonalVariants => self.draw_workspace_notes(
                ui,
                "Seasonal Variants",
                &[
                    "Seasonal animation parity",
                    "Variant missing checks",
                    "Preview sets",
                ],
            ),
        }
    }

    fn draw_character_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Character Editor",
            "Base sprites, outfits, tool-use previews, and 8-direction animation contracts.",
        );
        match self.character_subtab {
            CharacterSubTab::Bases => self.draw_workspace_notes(
                ui,
                "Bases",
                &[
                    "Character base selector",
                    "Base sprite validation",
                    "Palette defaults",
                ],
            ),
            CharacterSubTab::Outfits => self.draw_workspace_notes(
                ui,
                "Outfits",
                &["Outfit layers", "Equipment overlays", "Palette swaps"],
            ),
            CharacterSubTab::Tools => self.draw_workspace_notes(
                ui,
                "Tools",
                &[
                    "Tool-use preview",
                    "Held-item anchors",
                    "Tool animation contracts",
                ],
            ),
            CharacterSubTab::DirectionSets => self.draw_workspace_notes(
                ui,
                "Direction Sets",
                &[
                    "8-direction preview",
                    "Direction parity",
                    "Missing frame checks",
                ],
            ),
            CharacterSubTab::Preview => self.draw_workspace_notes(
                ui,
                "Preview",
                &[
                    "Paperdoll preview",
                    "Animation playback",
                    "Export validation",
                ],
            ),
        }
    }

    fn draw_logic_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Logic Blueprint Editor",
            "Event graph, bindings, tool logic, block/tile actions, and graph validation.",
        );
        match self.logic_subtab {
            LogicSubTab::Graphs => self.draw_workspace_notes(
                ui,
                "Graphs",
                &[
                    "Node graph canvas",
                    "Event/condition/action nodes",
                    "Save/load graph contract",
                ],
            ),
            LogicSubTab::EventBindings => self.draw_workspace_notes(
                ui,
                "Event Bindings",
                &[
                    "OnInteract",
                    "OnToolHit",
                    "OnEnterTile",
                    "OnDayStart",
                    "OnSeasonChanged",
                ],
            ),
            LogicSubTab::Tools => self.draw_workspace_notes(
                ui,
                "Tool Logic",
                &[
                    "Hoe/water/axe/pick/sword event mapping",
                    "Required item/tool conditions",
                    "Runtime interpreter handoff",
                ],
            ),
            LogicSubTab::Blocks => self.draw_workspace_notes(
                ui,
                "Blocks / Tiles",
                &[
                    "Replace tile",
                    "Spawn prop",
                    "Remove prop",
                    "Drop item",
                    "Play sound",
                ],
            ),
            LogicSubTab::Validation => self.draw_workspace_notes(
                ui,
                "Graph Validation",
                &[
                    "Missing references",
                    "Unreachable branches",
                    "Runtime-safe payload checks",
                ],
            ),
        }
    }

    fn draw_data_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Data Editors",
            "Items, crops, NPCs, dialogue, quests, shops, and schedules.",
        );
        match self.data_subtab {
            DataSubTab::Items => self.draw_data_count_panel(ui, "Items", self.registry.items.len()),
            DataSubTab::Crops => self.draw_data_count_panel(ui, "Crops", self.registry.crops.len()),
            DataSubTab::Npcs => self.draw_data_count_panel(ui, "NPCs", self.registry.npcs.len()),
            DataSubTab::Dialogue => {
                self.draw_data_count_panel(ui, "Dialogue", self.registry.dialogues.len())
            }
            DataSubTab::Quests => {
                self.draw_data_count_panel(ui, "Quests", self.registry.quests.len())
            }
            DataSubTab::Shops => self.draw_data_count_panel(ui, "Shops", self.registry.shops.len()),
            DataSubTab::Schedules => {
                self.draw_data_count_panel(ui, "Schedules", self.registry.schedules.len())
            }
        }
    }

    fn draw_playtest_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Playtest",
            "Runtime launch, reload, selected-map manifest, and diagnostics staging.",
        );
        match self.playtest_subtab {
            PlaytestSubTab::Launch => {
                ui.label(format!("Active map: {}", self.active_map_id));
                if ui.button("Rewrite live preview manifest").clicked() {
                    match write_editor_live_preview_manifest(
                        &self.project_root,
                        &self.active_map_id,
                    ) {
                        Ok(()) => {
                            self.status = "Rewrote editor_live_preview.ron.".to_string();
                            self.log(self.status.clone());
                        }
                        Err(error) => {
                            self.status = "Failed to rewrite live preview manifest.".to_string();
                            self.log(format!("Live preview manifest error: {error:#}"));
                        }
                    }
                }
            }
            PlaytestSubTab::Runtime => self.draw_runtime_tab(ui),
            PlaytestSubTab::Logs => self.draw_console_tab(ui),
        }
    }

    fn draw_settings_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "Settings",
            "Theme, layout, web companion, and editor behavior controls.",
        );
        match self.settings_subtab {
            SettingsSubTab::Preferences => self.draw_workspace_notes(
                ui,
                "Preferences",
                &["Dark mode theme", "Layout defaults", "Autosave policy"],
            ),
            SettingsSubTab::Keybinds => self.draw_workspace_notes(
                ui,
                "Keybinds",
                &[
                    "Workspace shortcuts",
                    "Tool shortcuts",
                    "Command palette bindings",
                ],
            ),
            SettingsSubTab::Paths => self.draw_workspace_notes(
                ui,
                "Paths",
                &["Project root", "Content paths", "Artifact paths"],
            ),
            SettingsSubTab::WebCompanion => self.draw_workspace_notes(
                ui,
                "Web Companion",
                &["LAN editor controls", "Port settings", "Conflict detection"],
            ),
        }
    }

    fn draw_data_count_panel(&self, ui: &mut egui::Ui, label: &str, count: usize) {
        self.draw_workspace_notes(
            ui,
            label,
            &[
                "CRUD editor shell",
                "Schema-aware defaults",
                "Cross-reference pickers",
                "Safe duplicate and rename",
            ],
        );
        ui.separator();
        ui.label(format!("{label}: {count} loaded record(s)"));
    }

    fn draw_workspace_header(&self, ui: &mut egui::Ui, title: &str, subtitle: &str) {
        ui.heading(title);
        ui.label(egui::RichText::new(subtitle).color(egui::Color32::from_rgb(164, 176, 196)));
        ui.separator();
    }

    fn draw_workspace_notes(&self, ui: &mut egui::Ui, title: &str, notes: &[&str]) {
        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_rgb(18, 23, 32))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 68, 90)))
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.heading(title);
                ui.add_space(4.0);
                for note in notes {
                    ui.label(format!("• {note}"));
                }
            });
    }

    fn world_preview_layout(&self, rect: egui::Rect) -> (u32, u32, f32, egui::Pos2) {
        let (map_width, map_height) = self.active_map_dimensions();
        let map_w = map_width.max(1) as f32;
        let map_h = map_height.max(1) as f32;
        let base_cell = (rect.width() / map_w).min(rect.height() / map_h) * self.preview_zoom;
        let cell = base_cell.clamp(4.0, 48.0);
        let world_w = cell * map_w;
        let world_h = cell * map_h;
        let origin = egui::pos2(
            rect.center().x - world_w * 0.5,
            rect.center().y - world_h * 0.5,
        );
        (map_width, map_height, cell, origin)
    }

    fn paint_world_preview(&self, rect: egui::Rect, painter: &egui::Painter) {
        let Some(state) = &self.editor_map else {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No editable layers.ron loaded",
                egui::FontId::proportional(18.0),
                egui::Color32::from_rgb(220, 230, 240),
            );
            return;
        };

        let (map_width, map_height, cell, origin) = self.world_preview_layout(rect);
        let world_w = cell * map_width as f32;
        let world_h = cell * map_height as f32;

        for (layer_index, layer) in state.layers.layers.iter().enumerate() {
            if !layer.visible {
                continue;
            }
            let legend = layer
                .legend
                .iter()
                .filter_map(|entry| {
                    entry
                        .symbol
                        .chars()
                        .next()
                        .map(|symbol| (symbol, entry.tile_id.as_str()))
                })
                .collect::<std::collections::HashMap<_, _>>();

            for (y, row) in layer.rows.iter().enumerate() {
                if y as u32 >= map_height {
                    break;
                }
                for (x, symbol) in row.chars().enumerate() {
                    if x as u32 >= map_width || is_empty_layer_symbol(symbol) {
                        continue;
                    }
                    let Some(tile_id) = legend.get(&symbol) else {
                        continue;
                    };
                    if !self.show_transitions && tile_id.contains("transition") {
                        continue;
                    }
                    let tile_rect = egui::Rect::from_min_size(
                        egui::pos2(origin.x + x as f32 * cell, origin.y + y as f32 * cell),
                        egui::vec2(cell, cell),
                    );
                    if !tile_rect.intersects(rect) {
                        continue;
                    }
                    let base_color = self.tile_color_from_id(tile_id);
                    let alpha = (255.0 * layer.opacity.clamp(0.0, 1.0)).round() as u8;
                    let layer_color = egui::Color32::from_rgba_unmultiplied(
                        base_color.r(),
                        base_color.g(),
                        base_color.b(),
                        alpha,
                    );
                    painter.rect_filled(tile_rect, 0.0, layer_color);

                    if layer_index == state.selected_layer_index && cell >= 10.0 {
                        painter.rect_stroke(
                            tile_rect.shrink(1.0),
                            0.0,
                            egui::Stroke::new(
                                0.5,
                                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 36),
                            ),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }
        }

        if self.show_grid && cell >= 7.0 {
            let stroke = egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 96));
            for x in 0..=map_width {
                let px = origin.x + x as f32 * cell;
                painter.line_segment(
                    [egui::pos2(px, origin.y), egui::pos2(px, origin.y + world_h)],
                    stroke,
                );
            }
            for y in 0..=map_height {
                let py = origin.y + y as f32 * cell;
                painter.line_segment(
                    [egui::pos2(origin.x, py), egui::pos2(origin.x + world_w, py)],
                    stroke,
                );
            }
        }

        self.paint_world_placement_overlays(rect, painter, cell, origin);

        if let Some((x, y)) = self.selected_map_cell {
            let tile_rect = egui::Rect::from_min_size(
                egui::pos2(origin.x + x as f32 * cell, origin.y + y as f32 * cell),
                egui::vec2(cell, cell),
            );
            painter.rect_stroke(
                tile_rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::WHITE),
                egui::StrokeKind::Inside,
            );
        }

        if let (Some(start), Some(end)) = (self.world_marquee_start, self.world_marquee_end) {
            let (left, top, right, bottom) = normalized_map_rect(start, end);
            let marquee = egui::Rect::from_min_max(
                egui::pos2(origin.x + left as f32 * cell, origin.y + top as f32 * cell),
                egui::pos2(
                    origin.x + (right + 1) as f32 * cell,
                    origin.y + (bottom + 1) as f32 * cell,
                ),
            );
            painter.rect_filled(
                marquee,
                0.0,
                egui::Color32::from_rgba_unmultiplied(112, 180, 255, 34),
            );
            painter.rect_stroke(
                marquee,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(146, 205, 255)),
                egui::StrokeKind::Inside,
            );
        }
    }

    fn paint_world_placement_overlays(
        &self,
        rect: egui::Rect,
        painter: &egui::Painter,
        cell: f32,
        origin: egui::Pos2,
    ) {
        let Some(state) = &self.world_placements else {
            return;
        };

        for (index, object) in state.voxel_objects.objects.iter().enumerate() {
            let footprint = egui::Rect::from_min_size(
                egui::pos2(origin.x + object.x * cell, origin.y + object.y * cell),
                egui::vec2(
                    object.footprint_width.max(0.25) * cell,
                    object.footprint_height.max(0.25) * cell,
                ),
            );
            if !footprint.intersects(rect) {
                continue;
            }
            let selected = state.active_selection == WorldPlacementKind::VoxelObject
                && state.selected_voxel_object_index == index;
            let color = if selected {
                egui::Color32::from_rgb(255, 221, 112)
            } else {
                egui::Color32::from_rgb(175, 150, 245)
            };
            painter.rect_filled(
                footprint,
                1.0,
                egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 34),
            );
            painter.rect_stroke(
                footprint,
                1.0,
                egui::Stroke::new(if selected { 2.5 } else { 1.5 }, color),
                egui::StrokeKind::Inside,
            );
            if selected {
                paint_resize_handle(painter, footprint, cell, "↘");
            }
            let handle = egui::Rect::from_center_size(
                footprint.center(),
                egui::vec2((cell * 0.5).clamp(8.0, 18.0), (cell * 0.5).clamp(8.0, 18.0)),
            );
            painter.rect_filled(handle, 2.0, color);
            painter.rect_stroke(
                handle,
                2.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(22, 18, 34)),
                egui::StrokeKind::Inside,
            );
            if cell >= 18.0 {
                painter.text(
                    handle.center(),
                    egui::Align2::CENTER_CENTER,
                    "V",
                    egui::FontId::proportional((cell * 0.34).clamp(8.0, 13.0)),
                    egui::Color32::from_rgb(24, 18, 32),
                );
            }
        }

        for (index, trigger) in state.triggers.iter().enumerate() {
            let trigger_rect = egui::Rect::from_min_size(
                egui::pos2(
                    origin.x + trigger.x as f32 * cell,
                    origin.y + trigger.y as f32 * cell,
                ),
                egui::vec2(
                    trigger.w.max(1) as f32 * cell,
                    trigger.h.max(1) as f32 * cell,
                ),
            );
            if !trigger_rect.intersects(rect) {
                continue;
            }
            let selected = state.active_selection == WorldPlacementKind::Trigger
                && state.selected_trigger_index == index;
            let color = if selected {
                egui::Color32::from_rgb(255, 205, 96)
            } else {
                egui::Color32::from_rgb(110, 186, 255)
            };
            painter.rect_filled(
                trigger_rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28),
            );
            painter.rect_stroke(
                trigger_rect,
                0.0,
                egui::Stroke::new(if selected { 2.5 } else { 1.5 }, color),
                egui::StrokeKind::Inside,
            );
            if selected {
                paint_resize_handle(painter, trigger_rect, cell, "↘");
            }
        }

        for (index, prop) in state.props.iter().enumerate() {
            let marker = placement_marker_rect(origin, cell, prop.x, prop.y, 0.56);
            if !marker.intersects(rect) {
                continue;
            }
            let selected = state.active_selection == WorldPlacementKind::Prop
                && state.selected_prop_index == index;
            let color = if selected {
                egui::Color32::from_rgb(255, 221, 112)
            } else {
                egui::Color32::from_rgb(120, 220, 155)
            };
            painter.rect_filled(marker, 2.0, color);
            painter.rect_stroke(
                marker,
                2.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(20, 28, 24)),
                egui::StrokeKind::Inside,
            );
        }

        for (index, spawn) in state.spawns.iter().enumerate() {
            let marker = placement_marker_rect(origin, cell, spawn.x, spawn.y, 0.7);
            if !marker.intersects(rect) {
                continue;
            }
            let selected = state.active_selection == WorldPlacementKind::Spawn
                && state.selected_spawn_index == index;
            let color = if selected {
                egui::Color32::from_rgb(255, 221, 112)
            } else {
                egui::Color32::from_rgb(234, 132, 182)
            };
            painter.circle_filled(marker.center(), marker.width() * 0.5, color);
            painter.circle_stroke(
                marker.center(),
                marker.width() * 0.5,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(34, 18, 30)),
            );
            if cell >= 18.0 {
                painter.text(
                    marker.center(),
                    egui::Align2::CENTER_CENTER,
                    "S",
                    egui::FontId::proportional((cell * 0.38).clamp(8.0, 14.0)),
                    egui::Color32::from_rgb(24, 18, 28),
                );
            }
        }
    }

    fn selected_world_resize_handle_contains(&self, rect: egui::Rect, pos: egui::Pos2) -> bool {
        self.selected_world_resize_handle_rect(rect)
            .is_some_and(|handle| handle.contains(pos))
    }

    fn selected_world_resize_handle_rect(&self, rect: egui::Rect) -> Option<egui::Rect> {
        let state = self.world_placements.as_ref()?;
        let (_, _, cell, origin) = self.world_preview_layout(rect);
        match state.active_selection {
            WorldPlacementKind::Trigger => {
                let trigger = state.triggers.get(state.selected_trigger_index)?;
                let object_rect = egui::Rect::from_min_size(
                    egui::pos2(
                        origin.x + trigger.x as f32 * cell,
                        origin.y + trigger.y as f32 * cell,
                    ),
                    egui::vec2(
                        trigger.w.max(1) as f32 * cell,
                        trigger.h.max(1) as f32 * cell,
                    ),
                );
                Some(resize_handle_rect(object_rect, cell))
            }
            WorldPlacementKind::VoxelObject => {
                let object = state
                    .voxel_objects
                    .objects
                    .get(state.selected_voxel_object_index)?;
                let object_rect = egui::Rect::from_min_size(
                    egui::pos2(origin.x + object.x * cell, origin.y + object.y * cell),
                    egui::vec2(
                        object.footprint_width.max(0.25) * cell,
                        object.footprint_height.max(0.25) * cell,
                    ),
                );
                Some(resize_handle_rect(object_rect, cell))
            }
            WorldPlacementKind::Prop | WorldPlacementKind::Spawn => None,
        }
    }

    fn pos_to_map_cell(&self, rect: egui::Rect, pos: egui::Pos2) -> Option<(u32, u32)> {
        let (map_width, map_height, cell, origin) = self.world_preview_layout(rect);
        let world_w = cell * map_width as f32;
        let world_h = cell * map_height as f32;

        if pos.x < origin.x
            || pos.y < origin.y
            || pos.x >= origin.x + world_w
            || pos.y >= origin.y + world_h
        {
            return None;
        }

        let x = ((pos.x - origin.x) / cell).floor() as u32;
        let y = ((pos.y - origin.y) / cell).floor() as u32;
        if x < map_width && y < map_height {
            Some((x, y))
        } else {
            None
        }
    }

    fn tile_id_at_map_cell(&self, x: u32, y: u32) -> Option<String> {
        let state = self.editor_map.as_ref()?;
        for layer in state
            .layers
            .layers
            .iter()
            .rev()
            .filter(|layer| layer.visible)
        {
            let Some(symbol) = layer_symbol_at(layer, x as usize, y as usize) else {
                continue;
            };
            if is_empty_layer_symbol(symbol) {
                continue;
            }
            if let Some(tile_id) = layer_tile_for_symbol(layer, symbol) {
                return Some(tile_id);
            }
        }
        None
    }
}

impl eframe::App for StarlightRidgeEguiEditor {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // eframe 0.34 passes the app an already-rooted Ui. All shell panels must be
        // attached with show_inside(ui, ...) so the editor is rendered exactly once
        // instead of nesting a second full shell inside the active panel.
        let ctx = ui.ctx().clone();
        self.begin_shell_render();
        self.poll_content_reload_job(&ctx);
        apply_editor_theme(&ctx);
        self.handle_shortcuts(&ctx);
        self.draw_top_bar(ui);
        self.draw_left_panel(ui);
        self.draw_right_panel(ui);
        self.draw_status_bar(ui);
        self.draw_bottom_panel(ui);
        self.draw_center_panel(ui);
        self.end_shell_render();
    }
}

fn ui_text_row(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        changed = ui.text_edit_singleline(value).changed();
    });
    changed
}

fn placement_marker_rect(origin: egui::Pos2, cell: f32, x: i32, y: i32, scale: f32) -> egui::Rect {
    let size = (cell * scale).clamp(6.0, 22.0);
    let center = egui::pos2(
        origin.x + (x as f32 + 0.5) * cell,
        origin.y + (y as f32 + 0.5) * cell,
    );
    egui::Rect::from_center_size(center, egui::vec2(size, size))
}

fn resize_handle_rect(object_rect: egui::Rect, cell: f32) -> egui::Rect {
    let size = (cell * 0.38).clamp(9.0, 18.0);
    egui::Rect::from_center_size(object_rect.right_bottom(), egui::vec2(size, size))
}

fn paint_resize_handle(painter: &egui::Painter, object_rect: egui::Rect, cell: f32, label: &str) {
    let handle = resize_handle_rect(object_rect, cell);
    painter.rect_filled(handle, 2.0, egui::Color32::from_rgb(255, 238, 158));
    painter.rect_stroke(
        handle,
        2.0,
        egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 32, 18)),
        egui::StrokeKind::Inside,
    );
    if cell >= 18.0 {
        painter.text(
            handle.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional((cell * 0.32).clamp(8.0, 12.0)),
            egui::Color32::from_rgb(40, 32, 18),
        );
    }
}

fn apply_editor_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(12, 15, 22);
    visuals.window_fill = egui::Color32::from_rgb(16, 20, 28);
    visuals.extreme_bg_color = egui::Color32::from_rgb(7, 9, 14);
    visuals.faint_bg_color = egui::Color32::from_rgb(22, 28, 38);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(16, 20, 28);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(26, 33, 45);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(41, 54, 74);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(65, 96, 132);
    visuals.selection.bg_fill = egui::Color32::from_rgb(79, 124, 172);
    visuals.hyperlink_color = egui::Color32::from_rgb(118, 174, 230);
    ctx.set_visuals(visuals);
}

fn default_selected_tile(registry: &ContentRegistry) -> Option<String> {
    registry
        .tilesets
        .values()
        .flat_map(|tileset| tileset.named_tiles.iter())
        .find(|tile| tile.id.contains("grass"))
        .or_else(|| {
            registry
                .tilesets
                .values()
                .flat_map(|tileset| tileset.named_tiles.iter())
                .next()
        })
        .map(|tile| tile.id.clone())
}

fn atlas_cell_for_tile(registry: &ContentRegistry, tile_id: &str) -> Option<(u32, u32)> {
    registry
        .tilesets
        .values()
        .flat_map(|tileset| tileset.named_tiles.iter())
        .find(|tile| tile.id == tile_id)
        .map(|tile| (tile.x, tile.y))
}

fn color_for_tile(tile: &TileInstance) -> egui::Color32 {
    let seed = tile
        .atlas_x
        .wrapping_mul(47)
        .wrapping_add(tile.atlas_y.wrapping_mul(89));
    let r = 70 + (seed.wrapping_mul(19) % 110) as u8;
    let g = 80 + (seed.wrapping_mul(29) % 120) as u8;
    let b = 70 + (seed.wrapping_mul(37) % 110) as u8;
    egui::Color32::from_rgb(r, g, b)
}

#[allow(dead_code)]
fn layer_tile_id_at(layer: &TileLayerDef, x: usize, y: usize) -> Option<String> {
    let row = layer.rows.get(y)?;
    let symbol = row.chars().nth(x)?;
    layer
        .legend
        .iter()
        .find(|entry| entry.symbol.chars().next() == Some(symbol))
        .map(|entry| entry.tile_id.clone())
}

/// Draw a simple isometric voxel projection of a loaded .vox model onto an egui canvas rect.
///
/// The projection uses an orthographic isometric view (X right, Y into screen, Z up).
/// Each voxel is drawn as a filled dot/square colored from the model palette.
/// Voxels are sorted back-to-front so closer voxels overdraw farther ones.
fn draw_vox_isometric_preview(
    painter: &egui::Painter,
    rect: egui::Rect,
    model: &VoxModel,
) {
    if model.voxels.is_empty() || model.palette.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No voxels",
            egui::FontId::proportional(13.0),
            egui::Color32::from_rgb(120, 130, 150),
        );
        return;
    }

    let w = model.width.max(1) as f32;
    let h = model.height.max(1) as f32;
    let d = model.depth.max(1) as f32;

    // Isometric projection: X → right+down, Y → left+down, Z → up
    // iso_x = (vx - vy) * cos30,  iso_y = (vx + vy) * sin30 - vz
    let cos30 = 0.866_f32;
    let sin30 = 0.5_f32;

    let project = |vx: f32, vy: f32, vz: f32| -> egui::Pos2 {
        let ix = (vx - vy) * cos30;
        let iy = (vx + vy) * sin30 - vz;
        egui::pos2(ix, iy)
    };

    // Compute bounding box of all corners to fit in rect
    let corners = [
        project(0.0, 0.0, 0.0),
        project(w, 0.0, 0.0),
        project(0.0, h, 0.0),
        project(w, h, 0.0),
        project(0.0, 0.0, d),
        project(w, 0.0, d),
        project(0.0, h, d),
        project(w, h, d),
    ];
    let min_x = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let min_y = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_y = corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
    let span_x = (max_x - min_x).max(1.0);
    let span_y = (max_y - min_y).max(1.0);
    let scale = (rect.width() / span_x).min(rect.height() / span_y) * 0.88;

    let offset_x = rect.center().x - (min_x + span_x * 0.5) * scale;
    let offset_y = rect.center().y - (min_y + span_y * 0.5) * scale;

    let to_screen = |vx: f32, vy: f32, vz: f32| -> egui::Pos2 {
        let iso = project(vx, vy, vz);
        egui::pos2(iso.x * scale + offset_x, iso.y * scale + offset_y)
    };

    let dot_radius = (scale * 0.55).clamp(1.0, 6.0);

    // Sort voxels back-to-front: larger x+y and smaller z draw first (painter's algorithm)
    let mut sorted: Vec<&engine_assets::vox::VoxVoxel> = model.voxels.iter().collect();
    sorted.sort_by(|a, b| {
        let depth_a = (a.x as i32 + a.y as i32) * 1000 - a.z as i32;
        let depth_b = (b.x as i32 + b.y as i32) * 1000 - b.z as i32;
        depth_a.cmp(&depth_b)
    });

    for voxel in sorted {
        let color_index = voxel.color_index as usize;
        let color = model.palette.get(color_index).copied().unwrap_or(
            engine_assets::vox::VoxColor { r: 180, g: 60, b: 200, a: 255 },
        );
        if color.a == 0 {
            continue;
        }
        let center = to_screen(voxel.x as f32 + 0.5, voxel.y as f32 + 0.5, voxel.z as f32 + 0.5);
        if !rect.expand(dot_radius).contains(center) {
            continue;
        }
        let face_color = egui::Color32::from_rgb(color.r, color.g, color.b);
        let shade = egui::Color32::from_rgb(
            (color.r as f32 * 0.72) as u8,
            (color.g as f32 * 0.72) as u8,
            (color.b as f32 * 0.72) as u8,
        );
        painter.circle_filled(center, dot_radius, face_color);
        painter.circle_stroke(center, dot_radius, egui::Stroke::new(0.5, shade));
    }
}

/// Validate a generated .vox file against the expected dimensions from a profile.
/// Returns `None` if the file looks correct, or a short diagnostic string.
fn validate_vox_profile(path: &std::path::Path, expected: [u8; 3]) -> Option<String> {
    let model = load_vox_file(path).ok()?;
    let (ew, eh, ed) = (expected[0] as u32, expected[1] as u32, expected[2] as u32);
    if model.width != ew || model.height != eh || model.depth != ed {
        Some(format!(
            "Dimensions {}×{}×{} do not match expected {}×{}×{}.",
            model.width, model.height, model.depth, ew, eh, ed
        ))
    } else if model.voxels.is_empty() {
        Some("Model has no voxels.".to_string())
    } else {
        None
    }
}
