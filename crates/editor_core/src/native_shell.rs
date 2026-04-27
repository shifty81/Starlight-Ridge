#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorWorkspace {
    AssetStudio,
    AnimationStudio,
    CharacterStudio,
    MapEditor,
    PrefabEditor,
    SceneEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockRegion {
    Left,
    Right,
    Bottom,
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeEditorTool {
    Select,
    Pan,
    Brush,
    Eraser,
    Fill,
    Eyedropper,
    TilePicker,
    CollisionPaint,
    AssetStudio,
    Playtest,
}

#[derive(Debug, Clone)]
pub struct DockPanelDescriptor {
    pub id: &'static str,
    pub title: &'static str,
    pub region: DockRegion,
    pub default_open: bool,
}

#[derive(Debug, Clone)]
pub struct ToolDescriptor {
    pub tool: NativeEditorTool,
    pub icon_id: &'static str,
    pub tooltip: &'static str,
    pub shortcut: &'static str,
}

#[derive(Debug, Clone)]
pub struct NativeEditorShellDescriptor {
    pub active_workspace: EditorWorkspace,
    pub center_viewport_id: &'static str,
    pub panels: Vec<DockPanelDescriptor>,
    pub toolbar: Vec<ToolDescriptor>,
}

impl NativeEditorShellDescriptor {
    pub fn phase29_default() -> Self {
        Self {
            active_workspace: EditorWorkspace::AssetStudio,
            center_viewport_id: "game_viewport",
            panels: vec![
                DockPanelDescriptor { id: "project_browser", title: "Project Browser", region: DockRegion::Left, default_open: true },
                DockPanelDescriptor { id: "asset_browser", title: "Asset Browser", region: DockRegion::Left, default_open: true },
                DockPanelDescriptor { id: "workflow_steps", title: "Workflow", region: DockRegion::Left, default_open: true },
                DockPanelDescriptor { id: "inspector", title: "Inspector", region: DockRegion::Right, default_open: true },
                DockPanelDescriptor { id: "tile_properties", title: "Selected Tile", region: DockRegion::Right, default_open: true },
                DockPanelDescriptor { id: "preview_tools", title: "Tile Preview Tools", region: DockRegion::Right, default_open: true },
                DockPanelDescriptor { id: "console", title: "Console", region: DockRegion::Bottom, default_open: true },
                DockPanelDescriptor { id: "validation", title: "Validation", region: DockRegion::Bottom, default_open: true },
                DockPanelDescriptor { id: "hot_reload", title: "Hot Reload", region: DockRegion::Bottom, default_open: true },
                DockPanelDescriptor { id: "runtime_log", title: "Runtime Log", region: DockRegion::Bottom, default_open: true },
            ],
            toolbar: vec![
                ToolDescriptor { tool: NativeEditorTool::Select, icon_id: "select", tooltip: "Select (V)", shortcut: "V" },
                ToolDescriptor { tool: NativeEditorTool::Pan, icon_id: "pan", tooltip: "Pan viewport (Space)", shortcut: "Space" },
                ToolDescriptor { tool: NativeEditorTool::Brush, icon_id: "brush", tooltip: "Paint tile (B)", shortcut: "B" },
                ToolDescriptor { tool: NativeEditorTool::Eraser, icon_id: "eraser", tooltip: "Erase tile/object (E)", shortcut: "E" },
                ToolDescriptor { tool: NativeEditorTool::Fill, icon_id: "fill", tooltip: "Bucket fill (G)", shortcut: "G" },
                ToolDescriptor { tool: NativeEditorTool::Eyedropper, icon_id: "eyedropper", tooltip: "Pick tile/color (I)", shortcut: "I" },
                ToolDescriptor { tool: NativeEditorTool::TilePicker, icon_id: "tile_picker", tooltip: "Tile picker (T)", shortcut: "T" },
                ToolDescriptor { tool: NativeEditorTool::CollisionPaint, icon_id: "collision", tooltip: "Paint collision (C)", shortcut: "C" },
                ToolDescriptor { tool: NativeEditorTool::AssetStudio, icon_id: "asset_studio", tooltip: "Asset Studio (A)", shortcut: "A" },
                ToolDescriptor { tool: NativeEditorTool::Playtest, icon_id: "playtest", tooltip: "Playtest viewport (P)", shortcut: "P" },
            ],
        }
    }
}
