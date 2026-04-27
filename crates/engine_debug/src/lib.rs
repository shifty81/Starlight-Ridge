#[derive(Debug, Default)]
pub struct DebugOverlayState {
    pub enabled: bool,
}

impl DebugOverlayState {
    pub fn new() -> Self {
        let state = Self { enabled: true };
        log::info!("debug overlay bootstrap enabled={}", state.enabled);
        state
    }
}
