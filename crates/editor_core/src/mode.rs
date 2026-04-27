#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Select,
    TerrainPaint,
    PropPlace,
    NpcPlace,
    TriggerPlace,
}
