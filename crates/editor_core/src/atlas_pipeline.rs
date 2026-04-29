use game_data::registry::ContentRegistry;

#[derive(Debug, Clone, Default)]
pub struct EditorAtlasPipelineReport {
    pub catalogs: usize,
    pub atlases: usize,
    pub editable_atlases: usize,
    pub season_sets: usize,
    pub water_animations: usize,
    pub clipboard_tools: usize,
    pub validation_checks: usize,
    pub game_preview_profiles: usize,
}

impl EditorAtlasPipelineReport {
    pub fn from_registry(registry: &ContentRegistry) -> Self {
        let mut report = Self {
            catalogs: registry.editor_atlas_pipelines.len(),
            ..Self::default()
        };

        for catalog in registry.editor_atlas_pipelines.values() {
            report.atlases += catalog.atlases.len();
            report.editable_atlases += catalog
                .atlases
                .iter()
                .filter(|atlas| atlas.editable)
                .count();
            report.season_sets += catalog.season_sets.len();
            report.water_animations += catalog.water_animations.len();
            report.clipboard_tools += catalog.clipboard_tools.len();
            report.validation_checks += catalog.validation_checks.len();
            report.game_preview_profiles += catalog.game_preview_profiles.len();
        }

        report
    }

    pub fn summary(&self) -> String {
        format!(
            "catalogs={} atlases={} editable_atlases={} season_sets={} water_animations={} clipboard_tools={} validation_checks={} game_preview_profiles={}",
            self.catalogs,
            self.atlases,
            self.editable_atlases,
            self.season_sets,
            self.water_animations,
            self.clipboard_tools,
            self.validation_checks,
            self.game_preview_profiles,
        )
    }
}

pub fn log_editor_pipeline_report(registry: &ContentRegistry) {
    let report = EditorAtlasPipelineReport::from_registry(registry);
    if report.catalogs == 0 {
        log::warn!(
            "phase19 editor atlas pipeline is not loaded; atlas compare/import stays scaffold-only"
        );
        return;
    }

    log::info!("phase19 editor atlas pipeline ready: {}", report.summary());
}
