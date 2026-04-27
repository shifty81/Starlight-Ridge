pub mod modes;
pub mod state;

pub fn init() -> anyhow::Result<()> {
    log::info!("game_core initialized");
    Ok(())
}
