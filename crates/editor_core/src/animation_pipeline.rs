use game_data::registry::ContentRegistry;

#[derive(Debug, Clone, Default)]
pub struct EditorAnimationPipelineReport {
    pub pipelines: usize,
    pub timeline_schemas: usize,
    pub animation_clips: usize,
    pub frames: usize,
    pub frame_events: usize,
    pub directional_groups: usize,
    pub socket_profiles: usize,
    pub socket_frames: usize,
    pub hitbox_profiles: usize,
    pub hitbox_frames: usize,
    pub water_preview_profiles: usize,
    pub seasonal_animation_sets: usize,
    pub validation_reports: usize,
}

impl EditorAnimationPipelineReport {
    pub fn from_registry(registry: &ContentRegistry) -> Self {
        let mut report = Self {
            pipelines: registry.editor_animation_pipelines.len(),
            ..Self::default()
        };

        for pipeline in registry.editor_animation_pipelines.values() {
            report.timeline_schemas += pipeline.timeline_schemas.len();
            report.animation_clips += pipeline.animation_clips.len();
            report.directional_groups += pipeline.directional_groups.len();
            report.socket_profiles += pipeline.socket_profiles.len();
            report.hitbox_profiles += pipeline.hitbox_profiles.len();
            report.water_preview_profiles += pipeline.water_preview_profiles.len();
            report.seasonal_animation_sets += pipeline.seasonal_animation_sets.len();
            report.validation_reports += pipeline.validation_reports.len();

            for clip in &pipeline.animation_clips {
                report.frames += clip.frames.len();
                for frame in &clip.frames {
                    report.frame_events += frame.events.len();
                }
            }
            for profile in &pipeline.socket_profiles {
                report.socket_frames += profile.sockets.len();
            }
            for profile in &pipeline.hitbox_profiles {
                report.hitbox_frames += profile.boxes.len();
            }
        }

        report
    }

    pub fn summary(&self) -> String {
        format!(
            "pipelines={} timeline_schemas={} clips={} frames={} events={} directional_groups={} socket_profiles={} socket_frames={} hitbox_profiles={} hitbox_frames={} water_previews={} seasonal_sets={} validation_reports={}",
            self.pipelines,
            self.timeline_schemas,
            self.animation_clips,
            self.frames,
            self.frame_events,
            self.directional_groups,
            self.socket_profiles,
            self.socket_frames,
            self.hitbox_profiles,
            self.hitbox_frames,
            self.water_preview_profiles,
            self.seasonal_animation_sets,
            self.validation_reports,
        )
    }
}

pub fn log_editor_animation_pipeline_report(registry: &ContentRegistry) {
    let report = EditorAnimationPipelineReport::from_registry(registry);
    if report.pipelines == 0 {
        log::warn!("phase21 editor animation pipeline is not loaded; animation timeline/event editing stays scaffold-only");
        return;
    }

    log::info!("phase21 editor animation pipeline ready: {}", report.summary());
}
