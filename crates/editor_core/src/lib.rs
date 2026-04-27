pub mod mode;
pub mod selection;
pub mod atlas_pipeline;
pub mod export_pipeline;
pub mod animation_pipeline;

pub fn init() -> anyhow::Result<()> {
    log::info!("editor_core initialized");
    Ok(())
}


pub fn init_with_registry(registry: &game_data::registry::ContentRegistry) -> anyhow::Result<()> {
    init()?;
    atlas_pipeline::log_editor_pipeline_report(registry);
    export_pipeline::log_editor_export_pipeline_report(registry);
    animation_pipeline::log_editor_animation_pipeline_report(registry);
    Ok(())
}
