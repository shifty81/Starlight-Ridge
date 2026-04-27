#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Title,
    Loading,
    InGame,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionMode {
    Play,
    Edit,
}

#[derive(Debug, Clone)]
pub struct BootstrapState {
    pub app_mode: AppMode,
    pub interaction_mode: InteractionMode,
    pub active_map_id: String,
}
