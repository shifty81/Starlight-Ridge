use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use eframe::egui;
use engine_assets::vox::{scan_vox_files, VoxAssetInfo};
use engine_render_gl::{TileInstance, TileMapRenderData};
use game_data::defs::{TilesetDef, TileLayerDef};
use game_data::registry::ContentRegistry;

use super::{
    build_tile_map_render_data, locate_project_root, load_tile_role_state,
    save_tile_role_state, write_editor_live_preview_manifest, TileRoleState,
    EDITOR_COLLISION_CYCLE, EDITOR_ROLE_CYCLE,
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
    Voxels,
    Props,
    Seasons,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorldSubTab {
    MapPaint,
    Layers,
    Interactions,
    Spawns,
    TerrainRules,
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
            let backup_path = self
                .image_path
                .with_file_name(format!(
                    "{}.phase51e.{}.bak.png",
                    self.image_path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("atlas"),
                    timestamp
                ));
            std::fs::copy(&self.image_path, &backup_path)
                .with_context(|| format!("failed to create atlas backup {}", backup_path.display()))?;
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
        self.clipboard = Some(PixelClipboard { width, height, pixels });
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
        let width = if turns % 2 == 0 { source.width } else { source.height };
        let height = if turns % 2 == 0 { source.height } else { source.width };
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
                pixels[target_index..target_index + 4].copy_from_slice(&source.pixels[source_index..source_index + 4]);
            }
        }

        Some(PixelClipboard { width, height, pixels })
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

    fn handle_canvas_interaction(&mut self, response: &egui::Response, image_rect: egui::Rect) -> Option<String> {
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
                    return Some(format!("Picked RGBA {},{},{},{}.", self.primary_color[0], self.primary_color[1], self.primary_color[2], self.primary_color[3]));
                }
                PixelTool::Fill => {
                    self.push_undo("Fill");
                    self.flood_fill(pixel.0, pixel.1, self.primary_color);
                    return Some(format!("Filled region at {},{}.", pixel.0, pixel.1));
                }
                PixelTool::ReplaceColorFill => {
                    self.push_undo("Replace color fill");
                    self.replace_color_fill(pixel.0, pixel.1, self.primary_color);
                    return Some(format!("Replaced matching color from {},{}.", pixel.0, pixel.1));
                }
                PixelTool::Line => {
                    if let Some(start) = self.line_start.take() {
                        self.push_undo("Line");
                        self.draw_line_pixels(start, pixel, self.primary_color);
                        return Some(format!("Drew line {},{} -> {},{}.", start.0, start.1, pixel.0, pixel.1));
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
            let stroke = egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180));
            for x in 0..=self.width() {
                let px = image_rect.left() + x as f32 * grid_w;
                painter.line_segment([egui::pos2(px, image_rect.top()), egui::pos2(px, image_rect.bottom())], stroke);
            }
            for y in 0..=self.height() {
                let py = image_rect.top() + y as f32 * grid_h;
                painter.line_segment([egui::pos2(image_rect.left(), py), egui::pos2(image_rect.right(), py)], stroke);
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
                [egui::pos2(x, image_rect.top()), egui::pos2(x, image_rect.bottom())],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 70, 70)),
            );
        }
        if self.mirror_y {
            let y = image_rect.center().y;
            painter.line_segment(
                [egui::pos2(image_rect.left(), y), egui::pos2(image_rect.right(), y)],
                egui::Stroke::new(1.5, egui::Color32::from_rgb(220, 70, 70)),
            );
        }

        if let Some((x, y, width, height)) = self.normalized_selection() {
            let selection_rect = egui::Rect::from_min_max(
                self.pixel_rect(image_rect, x, y).min,
                self.pixel_rect(image_rect, x + width - 1, y + height - 1).max,
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

struct StarlightRidgeEguiEditor {
    project_root: PathBuf,
    registry: ContentRegistry,
    active_map_id: String,
    tile_map: Option<TileMapRenderData>,
    selected_tool: usize,
    selected_asset_index: usize,
    selected_tile_id: String,
    selected_cell: (u32, u32),
    role_state: TileRoleState,
    selected_map_cell: Option<(u32, u32)>,
    left_tab: LeftTab,
    right_tab: RightTab,
    bottom_tab: BottomTab,
    workspace_tab: WorkspaceTab,
    asset_subtab: AssetSubTab,
    world_subtab: WorldSubTab,
    logic_subtab: LogicSubTab,
    tile_filter: String,
    status: String,
    log_lines: Vec<String>,
    show_grid: bool,
    show_transitions: bool,
    preview_zoom: f32,
    vox_assets: Vec<VoxAssetInfo>,
    selected_vox_index: usize,
    pixel_editor: PixelEditorState,
}

impl StarlightRidgeEguiEditor {
    fn new(project_root: PathBuf, registry: ContentRegistry, active_map_id: String) -> anyhow::Result<Self> {
        let tile_map = build_tile_map_render_data(&project_root, &registry, &active_map_id)
            .with_context(|| format!("failed to build egui editor preview for map '{active_map_id}'"))?;
        let selected_tile_id = default_selected_tile(&registry).unwrap_or_else(|| "grass_bright".to_string());
        let selected_cell = atlas_cell_for_tile(&registry, &selected_tile_id).unwrap_or((0, 0));
        let role_state = load_tile_role_state(&project_root, &selected_tile_id);
        let pixel_editor = PixelEditorState::load_for_active_tileset(&project_root, &registry, &active_map_id);
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
            selected_tool: 0,
            selected_asset_index: 0,
            selected_tile_id,
            selected_cell,
            role_state,
            selected_map_cell: None,
            left_tab: LeftTab::Project,
            right_tab: RightTab::Tile,
            bottom_tab: BottomTab::Console,
            workspace_tab: WorkspaceTab::Assets,
            asset_subtab: AssetSubTab::TerrainAtlas,
            world_subtab: WorldSubTab::MapPaint,
            logic_subtab: LogicSubTab::Graphs,
            tile_filter: String::new(),
            status: "egui editor ready. Native GL overlay is no longer the editor UI path.".to_string(),
            log_lines: Vec::new(),
            show_grid: true,
            show_transitions: true,
            preview_zoom: 1.0,
            vox_assets,
            selected_vox_index: 0,
            pixel_editor,
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

    fn active_tileset(&self) -> Option<&TilesetDef> {
        self.registry
            .maps
            .get(&self.active_map_id)
            .and_then(|map| self.registry.tilesets.get(&map.metadata.tileset))
            .or_else(|| self.registry.tilesets.values().next())
    }

    fn reload_content(&mut self) {
        match scan_vox_files(&self.project_root) {
            Ok(assets) => {
                self.vox_assets = assets;
                if self.selected_vox_index >= self.vox_assets.len() {
                    self.selected_vox_index = self.vox_assets.len().saturating_sub(1);
                }
            }
            Err(error) => {
                self.log(format!("VOX scan error: {error:#}"));
            }
        }

        match game_data::load_registry(&self.project_root) {
            Ok(registry) => {
                self.registry = registry;
                match build_tile_map_render_data(&self.project_root, &self.registry, &self.active_map_id) {
                    Ok(tile_map) => {
                        self.tile_map = tile_map;
                        self.status = "Reloaded content and rebuilt egui preview.".to_string();
                        self.log(self.status.clone());
                    }
                    Err(error) => {
                        self.status = "Reload failed while rebuilding tile preview.".to_string();
                        self.log(format!("Reload preview error: {error:#}"));
                    }
                }
            }
            Err(error) => {
                self.status = "Reload failed while reading content registry.".to_string();
                self.log(format!("Reload registry error: {error:#}"));
            }
        }
    }

    fn switch_map(&mut self, map_id: String) {
        if self.active_map_id == map_id {
            return;
        }

        self.active_map_id = map_id;
        self.selected_map_cell = None;
        match build_tile_map_render_data(&self.project_root, &self.registry, &self.active_map_id) {
            Ok(tile_map) => {
                self.tile_map = tile_map;
                let _ = write_editor_live_preview_manifest(&self.project_root, &self.active_map_id);
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
        self.selected_cell = atlas_cell_for_tile(&self.registry, &self.selected_tile_id).unwrap_or(self.selected_cell);
        self.role_state = load_tile_role_state(&self.project_root, &self.selected_tile_id);
        self.status = format!("{source} selected tile {} at atlas {},{}.", self.selected_tile_id, self.selected_cell.0, self.selected_cell.1);
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
        let path = self.project_root.join("artifacts").join("egui_asset_studio_selection.ron");
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
                    if let Some(label) = self.pixel_editor.undo() {
                        self.status = format!("Undid {label}.");
                    }
                    return;
                }
                if input.key_pressed(egui::Key::Y) {
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
                            self.status = format!("Copied selected atlas tile {},{}.", self.selected_cell.0, self.selected_cell.1);
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

    fn draw_top_bar(&mut self, ctx: &egui::Context) {
        egui::Panel::top("editor_top_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong("Starlight Ridge Editor");
                ui.separator();
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Project, "Project");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::World, "World");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Assets, "Assets");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Animation, "Animation");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Character, "Character");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Logic, "Logic");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Data, "Data");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Playtest, "Playtest");
                ui.selectable_value(&mut self.workspace_tab, WorkspaceTab::Settings, "Settings");
                ui.separator();
                if ui.button("Reload F5").clicked() {
                    self.reload_content();
                }
                if ui.button("Save checkpoint").clicked() {
                    self.write_selection_manifest();
                }
            });

            ui.separator();
            ui.horizontal_wrapped(|ui| {
                match self.workspace_tab {
                    WorkspaceTab::World => {
                        ui.label("World:");
                        ui.selectable_value(&mut self.world_subtab, WorldSubTab::MapPaint, "Map Paint");
                        ui.selectable_value(&mut self.world_subtab, WorldSubTab::Layers, "Layers");
                        ui.selectable_value(&mut self.world_subtab, WorldSubTab::Interactions, "Interactions");
                        ui.selectable_value(&mut self.world_subtab, WorldSubTab::Spawns, "Spawns");
                        ui.selectable_value(&mut self.world_subtab, WorldSubTab::TerrainRules, "Terrain Rules");
                    }
                    WorkspaceTab::Assets => {
                        ui.label("Assets:");
                        ui.selectable_value(&mut self.asset_subtab, AssetSubTab::TerrainAtlas, "Terrain Atlas");
                        ui.selectable_value(&mut self.asset_subtab, AssetSubTab::AtlasCompare, "Atlas Compare / Import");
                        ui.selectable_value(&mut self.asset_subtab, AssetSubTab::PixelEditor, "Pixel Editor");
                        ui.selectable_value(&mut self.asset_subtab, AssetSubTab::Voxels, "VOX Models");
                        ui.selectable_value(&mut self.asset_subtab, AssetSubTab::Props, "Props / Objects");
                        ui.selectable_value(&mut self.asset_subtab, AssetSubTab::Seasons, "Seasons");
                    }
                    WorkspaceTab::Logic => {
                        ui.label("Logic:");
                        ui.selectable_value(&mut self.logic_subtab, LogicSubTab::Graphs, "Graphs");
                        ui.selectable_value(&mut self.logic_subtab, LogicSubTab::EventBindings, "Event Bindings");
                        ui.selectable_value(&mut self.logic_subtab, LogicSubTab::Tools, "Tools");
                        ui.selectable_value(&mut self.logic_subtab, LogicSubTab::Blocks, "Blocks / Tiles");
                        ui.selectable_value(&mut self.logic_subtab, LogicSubTab::Validation, "Validation");
                    }
                    WorkspaceTab::Animation => {
                        ui.label("Animation subtabs: Clips · Timeline · Events · Sockets · Hitboxes · Seasonal Variants");
                    }
                    WorkspaceTab::Character => {
                        ui.label("Character subtabs: Bases · Outfits · Tools · Direction Sets · Preview");
                    }
                    WorkspaceTab::Data => {
                        ui.label("Data subtabs: Items · Crops · NPCs · Dialogue · Quests · Shops");
                    }
                    WorkspaceTab::Project | WorkspaceTab::Playtest | WorkspaceTab::Settings => {
                        ui.label("Project subtabs: Overview · Validation · Build · Export · Diagnostics");
                    }
                }
            });

            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.label("Tools:");
                for (index, label) in TOOL_NAMES.iter().enumerate() {
                    if ui
                        .selectable_label(self.selected_tool == index, *label)
                        .on_hover_text(format!("{} tool", label))
                        .clicked()
                    {
                        self.selected_tool = index;
                        self.status = format!("Active tool: {label}");
                    }
                }
                ui.separator();
                ui.label(&self.status);
            });
        });
    }
    fn draw_left_panel(&mut self, ctx: &egui::Context) {
        egui::Panel::left("editor_left_panel")
            .resizable(true)
            .default_width(280.0)
            .width_range(220.0..=460.0)
            .show(ctx, |ui| {
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
        ui.label(format!("Sprite sheets: {}", self.registry.sprite_sheets.len()));
        ui.label(format!("Terrain rulesets: {}", self.registry.terrain_rulesets.len()));
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
            ui.label(format!("Size: {} x {}", map.metadata.width, map.metadata.height));
            ui.label(format!("Tileset: {}", map.metadata.tileset));
            ui.label(format!("Props: {}", map.props.len()));
            ui.label(format!("Spawns: {}", map.spawns.len()));
            ui.label(format!("Triggers: {}", map.triggers.len()));
        }

        ui.separator();
        ui.checkbox(&mut self.show_grid, "Show map grid");
        ui.checkbox(&mut self.show_transitions, "Show transition overlays");
        ui.add(egui::Slider::new(&mut self.preview_zoom, 0.5..=3.0).text("Preview zoom"));
    }

    fn draw_right_panel(&mut self, ctx: &egui::Context) {
        egui::Panel::right("editor_right_panel")
            .resizable(true)
            .default_width(320.0)
            .width_range(260.0..=520.0)
            .show(ctx, |ui| {
                ui.heading("Inspector");
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.right_tab, RightTab::Tile, "Tile");
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
        ui.label("Selected tile");
        ui.monospace(&self.selected_tile_id);
        ui.label(format!("Atlas cell: {},{}", self.selected_cell.0, self.selected_cell.1));

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
            self.status = format!("Changed role for {} to {}.", self.selected_tile_id, self.role_state.role);
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
                self.selected_tile_id,
                self.role_state.collision
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
            self.status = format!("Focused native egui Asset Lab for {}.", self.selected_tile_id);
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

    fn draw_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::Panel::bottom("editor_bottom_panel")
            .resizable(false)
            .exact_height(190.0)
            .show(ctx, |ui| {
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

    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::Panel::bottom("editor_static_status_bar")
            .resizable(false)
            .exact_height(28.0)
            .show(ctx, |ui| {
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
        ui.label("Known missing validation targets:");
        ui.label("• external atlas import must validate size, tile grid, role tags, and collisions before merging");
        ui.label("• animation timeline events need socket/hitbox preview and save validation");
        ui.label("• maps need save-time checks for missing tile IDs, blocked spawn points, and invalid transitions");
        ui.label("• seasonal tile sets need parity checks across spring/summer/autumn/winter");
    }

    fn draw_hot_reload_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Manual hot reload is active through F5.");
        ui.label("The old editor watched PNG/RON/TOML files from the GL loop. The egui path currently reloads explicitly so the UI stays deterministic.");
        if ui.button("Reload now").clicked() {
            self.reload_content();
        }
    }

    fn draw_runtime_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Active map: {}", self.active_map_id));
        ui.label(format!(
            "Tile render instances: {}",
            self.tile_map.as_ref().map(|map| map.tiles.len()).unwrap_or(0)
        ));
        ui.label(format!("Selected tool: {}", TOOL_NAMES[self.selected_tool]));
        ui.label(format!("Selected tile: {}", self.selected_tile_id));
    }

    fn draw_center_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(12, 15, 22)))
            .show(ctx, |ui| match self.workspace_tab {
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
        self.draw_workspace_header(ui, "Project Overview", "Project health, content counts, and build/readiness checks.");
        ui.columns(3, |columns| {
            columns[0].heading("Content");
            columns[0].label(format!("Maps: {}", self.registry.maps.len()));
            columns[0].label(format!("Tilesets: {}", self.registry.tilesets.len()));
            columns[0].label(format!("Sprite sheets: {}", self.registry.sprite_sheets.len()));
            columns[1].heading("Gameplay data");
            columns[1].label(format!("Items: {}", self.registry.items.len()));
            columns[1].label(format!("Crops: {}", self.registry.crops.len()));
            columns[1].label(format!("NPCs: {}", self.registry.npcs.len()));
            columns[2].heading("Editor contracts");
            columns[2].label(format!("Atlas pipelines: {}", self.registry.editor_atlas_pipelines.len()));
            columns[2].label(format!("Export pipelines: {}", self.registry.editor_export_pipelines.len()));
            columns[2].label(format!("Animation pipelines: {}", self.registry.editor_animation_pipelines.len()));
        });
    }

    fn draw_world_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(ui, "World Editor", "Map preview, layer inspection, interaction zones, spawns, and terrain-rule scaffolds.");
        match self.world_subtab {
            WorldSubTab::MapPaint => self.draw_world_preview_workspace(ui),
            WorldSubTab::Layers => self.draw_workspace_notes(ui, "Layer tools", &["Layer visibility/edit locks", "Layer order", "Collision and transition overlays", "Save-time layer validation"]),
            WorldSubTab::Interactions => self.draw_workspace_notes(ui, "Interaction tools", &["Clickable trigger regions", "Door/warp metadata", "Tool-hit interaction zones", "Object interaction probes"]),
            WorldSubTab::Spawns => self.draw_workspace_notes(ui, "Spawn tools", &["Player spawn marker editing", "NPC spawn markers", "Blocked-spawn validation", "Playtest from selected cell"]),
            WorldSubTab::TerrainRules => self.draw_workspace_notes(ui, "Terrain rules", &["Autotile ruleset view", "Transition preview pairs", "Protected layer report", "Bake generated draft to editable map"]),
        }
    }

    fn draw_world_preview_workspace(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("World Preview");
            ui.label(format!("Map: {}", self.active_map_id));
            ui.label(format!("Tool: {}", TOOL_NAMES[self.selected_tool]));
        });
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

        if response.clicked() {
            if let Some(pointer) = response.interact_pointer_pos() {
                if let Some((map_x, map_y)) = self.pos_to_map_cell(rect, pointer) {
                    self.selected_map_cell = Some((map_x, map_y));
                    if let Some(tile_id) = self.tile_id_at_map_cell(map_x, map_y) {
                        self.select_tile(tile_id, "World preview");
                    } else {
                        self.status = format!("Selected empty map cell {map_x},{map_y}.");
                    }
                }
            }
        }
    }

    fn draw_assets_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(ui, "Asset Studio", "Terrain atlas, compare/import, pixel editing, props, and seasonal asset readiness.");
        match self.asset_subtab {
            AssetSubTab::TerrainAtlas => self.draw_workspace_notes(ui, "Terrain Atlas", &["Current patch keeps the selectable metadata list stable", "Phase 42 should replace color-cell preview with the real atlas texture", "Tile role and collision metadata can be saved from the inspector"]),
            AssetSubTab::AtlasCompare => self.draw_workspace_notes(ui, "Atlas Compare / Import", &["Side-by-side source/project tilesheet preview", "Drag tile transfer", "Overwrite/append modes", "Mirror-aware paste", "Metadata rewrite and validation"]),
            AssetSubTab::PixelEditor => self.draw_pixel_editor_workspace(ui),
            AssetSubTab::Voxels => self.draw_voxels_workspace(ui),
            AssetSubTab::Props => self.draw_workspace_notes(ui, "Props / Objects", &["Static prop atlas", "Object placement metadata", "Collision footprint preview", "Interaction marker preview"]),
            AssetSubTab::Seasons => self.draw_workspace_notes(ui, "Season Variants", &["Spring/summer/autumn/winter parity", "Season-specific atlas selection", "Water animation preview", "Missing variant validation"]),
        }
    }


    fn draw_voxels_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(
            ui,
            "VOX Models",
            "Phase 51f: MagicaVoxel .vox files are scanned from assets/voxels, assets/models, and content/voxels for editor/runtime asset use.",
        );

        ui.horizontal_wrapped(|ui| {
            if ui.button("Reload VOX assets").clicked() {
                match scan_vox_files(&self.project_root) {
                    Ok(assets) => {
                        self.vox_assets = assets;
                        self.selected_vox_index = self.selected_vox_index.min(self.vox_assets.len().saturating_sub(1));
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
                if let Some(asset) = self.vox_assets.get(index) {
                    self.status = format!("Selected VOX model {}.", asset.id);
                }
            }

            columns[1].heading("Selected model");
            if let Some(asset) = self.vox_assets.get(self.selected_vox_index) {
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
                columns[1].label("Current integration: discovery, validation-safe parsing, editor listing, and metadata summary.");
                columns[1].label("Next step: add projection/bake tools so .vox models can become tile sprites, prop sprites, collision footprints, or preview thumbnails.");
            }
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
            if ui.button("Undo Ctrl+Z").clicked() {
                if let Some(label) = self.pixel_editor.undo() {
                    self.status = format!("Undid {label}.");
                }
            }
            if ui.button("Redo Ctrl+Y").clicked() {
                if self.pixel_editor.redo().is_some() {
                    self.status = "Redid pixel edit.".to_string();
                }
            }
            if ui.button("Copy tile").clicked() {
                let tile_size = self.active_tile_size();
                if self.pixel_editor.copy_tile(self.selected_cell, tile_size) {
                    self.pixel_editor.tool = PixelTool::Paste;
                    self.status = format!("Copied selected atlas tile {},{}.", self.selected_cell.0, self.selected_cell.1);
                }
            }
            if ui.button("Copy selection Ctrl+C").clicked() {
                if self.pixel_editor.copy_selection() {
                    self.pixel_editor.tool = PixelTool::Paste;
                    self.status = "Copied pixel selection.".to_string();
                }
            }
            if ui.button("Save PNG").clicked() {
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
            let swatch = egui::RichText::new("     ").background_color(egui::Color32::from_rgba_unmultiplied(
                self.pixel_editor.primary_color[0],
                self.pixel_editor.primary_color[1],
                self.pixel_editor.primary_color[2],
                self.pixel_editor.primary_color[3],
            ));
            ui.label(swatch);
        });

        ui.horizontal_wrapped(|ui| {
            ui.checkbox(&mut self.pixel_editor.mirror_x, "Mirror X");
            ui.checkbox(&mut self.pixel_editor.mirror_y, "Mirror Y");
            ui.checkbox(&mut self.pixel_editor.flip_paste_x, "Flip paste X");
            ui.checkbox(&mut self.pixel_editor.flip_paste_y, "Flip paste Y");
            if ui.button("Rotate paste 90°").clicked() {
                self.pixel_editor.rotate_paste_quarters = (self.pixel_editor.rotate_paste_quarters + 1) % 4;
            }
            ui.label(format!("Paste rotation: {}°", self.pixel_editor.rotate_paste_quarters * 90));
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
        let Some(texture_id) = self.pixel_editor.texture.as_ref().map(|texture| texture.id()) else {
            ui.label("Pixel editor texture is not loaded.");
            return;
        };

        let image_width = self.pixel_editor.width().max(1) as f32;
        let image_height = self.pixel_editor.height().max(1) as f32;
        let available = ui.available_size_before_wrap();
        let max_width = available.x.max(360.0);
        let max_height = available.y.max(360.0);
        let fit = (max_width / image_width).min(max_height / image_height).max(0.25);
        let scale = (fit * self.pixel_editor.zoom).clamp(0.25, 32.0);
        let canvas_size = egui::vec2(image_width * scale, image_height * scale);

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::click_and_drag());
                self.pixel_editor.last_canvas_rect = Some(rect);
                let painter = ui.painter_at(rect);
                self.pixel_editor.paint_checkerboard(&painter, rect);
                painter.image(
                    texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                if let Some(message) = self.pixel_editor.handle_canvas_interaction(&response, rect) {
                    self.status = message;
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
        self.draw_workspace_header(ui, "Animation Editor", "Timeline, frame events, sockets, hitboxes, and directional groups.");
        self.draw_workspace_notes(ui, "Phase 41 scaffold", &["Clip list and timeline routing", "Frame event metadata", "Tool sockets", "Hitbox and interaction box preview", "Seasonal animation variants"]);
    }

    fn draw_character_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(ui, "Character Editor", "Base sprites, outfits, tool-use previews, and 8-direction animation contracts.");
        self.draw_workspace_notes(ui, "Phase 41 scaffold", &["Character base selector", "Outfit/layer preview", "Tool animation preview", "Direction set validation", "Sprite sheet export contract"]);
    }

    fn draw_logic_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(ui, "Logic Blueprint Editor", "Event graph, bindings, tool logic, block/tile actions, and graph validation.");
        match self.logic_subtab {
            LogicSubTab::Graphs => self.draw_workspace_notes(ui, "Graphs", &["Node graph canvas", "Event/condition/action nodes", "Save/load graph contract"]),
            LogicSubTab::EventBindings => self.draw_workspace_notes(ui, "Event Bindings", &["OnInteract", "OnToolHit", "OnEnterTile", "OnDayStart", "OnSeasonChanged"]),
            LogicSubTab::Tools => self.draw_workspace_notes(ui, "Tool Logic", &["Hoe/water/axe/pick/sword event mapping", "Required item/tool conditions", "Runtime interpreter handoff"]),
            LogicSubTab::Blocks => self.draw_workspace_notes(ui, "Blocks / Tiles", &["Replace tile", "Spawn prop", "Remove prop", "Drop item", "Play sound"]),
            LogicSubTab::Validation => self.draw_workspace_notes(ui, "Graph Validation", &["Missing references", "Unreachable branches", "Runtime-safe payload checks"]),
        }
    }

    fn draw_data_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(ui, "Data Editors", "Items, crops, NPCs, dialogue, quests, shops, and schedules.");
        ui.columns(3, |columns| {
            columns[0].label(format!("Items: {}", self.registry.items.len()));
            columns[0].label(format!("Crops: {}", self.registry.crops.len()));
            columns[0].label(format!("Shops: {}", self.registry.shops.len()));
            columns[1].label(format!("NPCs: {}", self.registry.npcs.len()));
            columns[1].label(format!("Dialogue: {}", self.registry.dialogues.len()));
            columns[1].label(format!("Schedules: {}", self.registry.schedules.len()));
            columns[2].label(format!("Quests: {}", self.registry.quests.len()));
            columns[2].label(format!("Terrain types: {}", self.registry.terrain_types.len()));
            columns[2].label(format!("Biome packs: {}", self.registry.biome_packs.len()));
        });
    }

    fn draw_playtest_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(ui, "Playtest", "Runtime launch, reload, selected-map manifest, and diagnostics staging.");
        self.draw_runtime_tab(ui);
        ui.separator();
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
    }

    fn draw_settings_workspace(&mut self, ui: &mut egui::Ui) {
        self.draw_workspace_header(ui, "Settings", "Theme, layout, web companion, and editor behavior controls.");
        self.draw_workspace_notes(ui, "Current settings", &["Dark mode is now forced through the egui theme pass", "Bottom console/status layout is fixed", "Web companion server controls are queued for Phase 45"]);
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

    fn paint_world_preview(&self, rect: egui::Rect, painter: &egui::Painter) {
        let Some(tile_map) = &self.tile_map else {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No tile map render data loaded",
                egui::FontId::proportional(18.0),
                egui::Color32::from_rgb(220, 230, 240),
            );
            return;
        };

        let map_w = tile_map.map_width.max(1) as f32;
        let map_h = tile_map.map_height.max(1) as f32;
        let base_cell = (rect.width() / map_w).min(rect.height() / map_h) * self.preview_zoom;
        let cell = base_cell.clamp(4.0, 48.0);
        let world_w = cell * map_w;
        let world_h = cell * map_h;
        let origin = egui::pos2(
            rect.center().x - world_w * 0.5,
            rect.center().y - world_h * 0.5,
        );

        for tile in &tile_map.tiles {
            if !self.show_transitions && is_likely_transition(tile) {
                continue;
            }
            let x = origin.x + tile.x as f32 * cell;
            let y = origin.y + tile.y as f32 * cell;
            let tile_rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(cell, cell));
            if !tile_rect.intersects(rect) {
                continue;
            }
            painter.rect_filled(tile_rect, 0.0, color_for_tile(tile));
        }

        if self.show_grid && cell >= 7.0 {
            let stroke = egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 96));
            for x in 0..=tile_map.map_width {
                let px = origin.x + x as f32 * cell;
                painter.line_segment(
                    [egui::pos2(px, origin.y), egui::pos2(px, origin.y + world_h)],
                    stroke,
                );
            }
            for y in 0..=tile_map.map_height {
                let py = origin.y + y as f32 * cell;
                painter.line_segment(
                    [egui::pos2(origin.x, py), egui::pos2(origin.x + world_w, py)],
                    stroke,
                );
            }
        }

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
    }

    fn pos_to_map_cell(&self, rect: egui::Rect, pos: egui::Pos2) -> Option<(u32, u32)> {
        let tile_map = self.tile_map.as_ref()?;
        let map_w = tile_map.map_width.max(1) as f32;
        let map_h = tile_map.map_height.max(1) as f32;
        let base_cell = (rect.width() / map_w).min(rect.height() / map_h) * self.preview_zoom;
        let cell = base_cell.clamp(4.0, 48.0);
        let world_w = cell * map_w;
        let world_h = cell * map_h;
        let origin = egui::pos2(rect.center().x - world_w * 0.5, rect.center().y - world_h * 0.5);

        if pos.x < origin.x || pos.y < origin.y || pos.x >= origin.x + world_w || pos.y >= origin.y + world_h {
            return None;
        }

        let x = ((pos.x - origin.x) / cell).floor() as u32;
        let y = ((pos.y - origin.y) / cell).floor() as u32;
        if x < tile_map.map_width && y < tile_map.map_height {
            Some((x, y))
        } else {
            None
        }
    }

    fn tile_id_at_map_cell(&self, x: u32, y: u32) -> Option<String> {
        let tile_map = self.tile_map.as_ref()?;
        let tile = tile_map
            .tiles
            .iter()
            .rev()
            .find(|tile| tile.x == x && tile.y == y)?;

        self.active_tileset()?
            .named_tiles
            .iter()
            .find(|entry| entry.x == tile.atlas_x && entry.y == tile.atlas_y)
            .map(|entry| entry.id.clone())
    }
}

impl eframe::App for StarlightRidgeEguiEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_editor_theme(ctx);
        self.handle_shortcuts(ctx);
        self.draw_top_bar(ctx);
        self.draw_left_panel(ctx);
        self.draw_right_panel(ctx);
        self.draw_status_bar(ctx);
        self.draw_bottom_panel(ctx);
        self.draw_center_panel(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.update(ui.ctx(), frame);
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
        .or_else(|| registry.tilesets.values().flat_map(|tileset| tileset.named_tiles.iter()).next())
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

fn is_likely_transition(tile: &TileInstance) -> bool {
    tile.atlas_y > 3
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
