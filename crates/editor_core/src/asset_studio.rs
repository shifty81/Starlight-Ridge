#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilePreviewMode {
    SelectedTile,
    AtlasOutput,
    SeamDiagnostic,
    LiveViewport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelTool {
    Pencil,
    Eraser,
    Fill,
    Eyedropper,
    MarqueeSelect,
    MoveSelection,
}

#[derive(Debug, Clone)]
pub struct AssetStudioDescriptor {
    pub atlas_path: &'static str,
    pub roles_path: &'static str,
    pub tile_size: u32,
    pub active_tool: PixelTool,
    pub preview_modes: Vec<TilePreviewMode>,
    pub runtime_safe_export_enabled: bool,
}

impl AssetStudioDescriptor {
    pub fn phase27_default() -> Self {
        Self {
            atlas_path: "assets/textures/terrain_atlas_phase17_generated.png",
            roles_path: "content/tiles/base_tileset_roles.ron",
            tile_size: 32,
            active_tool: PixelTool::Pencil,
            preview_modes: vec![
                TilePreviewMode::SelectedTile,
                TilePreviewMode::AtlasOutput,
                TilePreviewMode::SeamDiagnostic,
                TilePreviewMode::LiveViewport,
            ],
            runtime_safe_export_enabled: true,
        }
    }
}
