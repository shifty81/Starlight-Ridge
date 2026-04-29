use game_data::registry::ContentRegistry;

#[derive(Debug, Clone, Default)]
pub struct EditorExportPipelineReport {
    pub pipelines: usize,
    pub export_profiles: usize,
    pub validation_panels: usize,
    pub validation_filters: usize,
    pub validation_jump_targets: usize,
    pub autotile_rule_sets: usize,
    pub autotile_rules: usize,
    pub transition_rule_editors: usize,
    pub transition_preview_pairs: usize,
    pub collision_interaction_profiles: usize,
    pub atlas_cleanup_manifests: usize,
}

impl EditorExportPipelineReport {
    pub fn from_registry(registry: &ContentRegistry) -> Self {
        let mut report = Self {
            pipelines: registry.editor_export_pipelines.len(),
            ..Self::default()
        };

        for pipeline in registry.editor_export_pipelines.values() {
            report.export_profiles += pipeline.export_profiles.len();
            report.validation_panels += pipeline.validation_panels.len();
            report.autotile_rule_sets += pipeline.autotile_rule_sets.len();
            report.transition_rule_editors += pipeline.transition_rule_editors.len();
            report.collision_interaction_profiles += pipeline.collision_interaction_profiles.len();
            report.atlas_cleanup_manifests += pipeline.atlas_cleanup_manifests.len();

            for panel in &pipeline.validation_panels {
                report.validation_filters += panel.issue_filters.len();
                report.validation_jump_targets += panel.jump_targets.len();
            }

            for ruleset in &pipeline.autotile_rule_sets {
                report.autotile_rules += ruleset.rules.len();
            }

            for editor in &pipeline.transition_rule_editors {
                report.transition_preview_pairs += editor.preview_pairs.len();
            }
        }

        report
    }

    pub fn summary(&self) -> String {
        format!(
            "pipelines={} export_profiles={} validation_panels={} filters={} jump_targets={} autotile_rule_sets={} autotile_rules={} transition_editors={} transition_preview_pairs={} collision_profiles={} cleanup_manifests={}",
            self.pipelines,
            self.export_profiles,
            self.validation_panels,
            self.validation_filters,
            self.validation_jump_targets,
            self.autotile_rule_sets,
            self.autotile_rules,
            self.transition_rule_editors,
            self.transition_preview_pairs,
            self.collision_interaction_profiles,
            self.atlas_cleanup_manifests,
        )
    }
}

pub fn log_editor_export_pipeline_report(registry: &ContentRegistry) {
    let report = EditorExportPipelineReport::from_registry(registry);
    if report.pipelines == 0 {
        log::warn!(
            "phase20 editor export/validation pipeline is not loaded; export validation stays scaffold-only"
        );
        return;
    }

    log::info!(
        "phase20 editor export/validation pipeline ready: {}",
        report.summary()
    );
}
