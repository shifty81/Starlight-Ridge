use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use game_data::defs::MapLayersDef;
use game_world::{PropPlacement, SpawnPoint, TriggerZone};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct EditorContentPaths {
    project_root: PathBuf,
}

impl EditorContentPaths {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn map_dir(&self, map_id: &str) -> PathBuf {
        self.project_root.join("content").join("maps").join(map_id)
    }

    pub fn map_file(&self, map_id: &str, file_name: &str) -> PathBuf {
        self.map_dir(map_id).join(file_name)
    }

    pub fn map_layers_path(&self, map_id: &str) -> PathBuf {
        self.map_file(map_id, "layers.ron")
    }

    pub fn props_path(&self, map_id: &str) -> PathBuf {
        self.map_file(map_id, "props.ron")
    }

    pub fn spawns_path(&self, map_id: &str) -> PathBuf {
        self.map_file(map_id, "spawns.ron")
    }

    pub fn triggers_path(&self, map_id: &str) -> PathBuf {
        self.map_file(map_id, "triggers.ron")
    }

    pub fn voxel_objects_path(&self, map_id: &str) -> PathBuf {
        self.map_file(map_id, "voxel_objects.ron")
    }

    pub fn scene_dir(&self, map_id: &str) -> PathBuf {
        self.project_root
            .join("content")
            .join("scenes")
            .join(map_id)
    }

    pub fn scene_voxel_objects_path(&self, map_id: &str) -> PathBuf {
        self.scene_dir(map_id).join("voxel_objects.ron")
    }

    pub fn voxel_panel_kit_dir(&self) -> PathBuf {
        self.project_root
            .join("content")
            .join("editor_voxel_panels")
            .join("panel_kits")
    }
}

#[derive(Debug, Clone)]
pub struct LoadedAsset<T> {
    pub path: PathBuf,
    pub value: T,
}

#[derive(Debug, Clone)]
pub struct WorldPlacementFiles {
    pub map_id: String,
    pub props_path: PathBuf,
    pub spawns_path: PathBuf,
    pub triggers_path: PathBuf,
    pub props: Vec<PropPlacement>,
    pub spawns: Vec<SpawnPoint>,
    pub triggers: Vec<TriggerZone>,
}

#[derive(Debug, Clone)]
pub struct SaveOutcome {
    pub path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub temp_path: Option<PathBuf>,
    pub used_temp_write: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorSaveState {
    Clean,
    Dirty,
    Saving,
    Failed,
}

impl EditorSaveState {
    pub fn label(self) -> &'static str {
        match self {
            EditorSaveState::Clean => "clean",
            EditorSaveState::Dirty => "dirty",
            EditorSaveState::Saving => "saving",
            EditorSaveState::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct EditorFileState {
    pub label: String,
    pub path: PathBuf,
    pub state: EditorSaveState,
    pub last_backup_path: Option<PathBuf>,
    pub last_error: Option<String>,
}

impl EditorFileState {
    pub fn clean(label: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            label: label.into(),
            path: path.into(),
            state: EditorSaveState::Clean,
            last_backup_path: None,
            last_error: None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.state = EditorSaveState::Dirty;
        self.last_error = None;
    }

    pub fn mark_saving(&mut self) {
        self.state = EditorSaveState::Saving;
        self.last_error = None;
    }

    pub fn mark_saved(&mut self, outcome: &SaveOutcome) {
        self.state = EditorSaveState::Clean;
        self.last_backup_path = outcome.backup_path.clone();
        self.last_error = None;
    }

    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.state = EditorSaveState::Failed;
        self.last_error = Some(error.into());
    }

    pub fn is_dirty(&self) -> bool {
        self.state == EditorSaveState::Dirty
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditorDirtyState {
    pub files: Vec<EditorFileState>,
}

impl EditorDirtyState {
    pub fn dirty_count(&self) -> usize {
        self.files.iter().filter(|file| file.is_dirty()).count()
    }

    pub fn has_dirty_files(&self) -> bool {
        self.dirty_count() > 0
    }

    pub fn upsert(&mut self, file: EditorFileState) {
        if let Some(existing) = self.files.iter_mut().find(|entry| entry.path == file.path) {
            *existing = file;
        } else {
            self.files.push(file);
        }
    }
}

pub fn init() -> anyhow::Result<()> {
    log::info!("editor_data_bridge initialized");
    Ok(())
}

pub fn load_map_layers(
    project_root: impl AsRef<Path>,
    map_id: &str,
) -> anyhow::Result<LoadedAsset<MapLayersDef>> {
    let paths = EditorContentPaths::new(project_root.as_ref());
    let path = paths.map_layers_path(map_id);
    let value = game_data::loader::load_map_layers(&path)?;
    Ok(LoadedAsset { path, value })
}

pub fn load_world_placements(
    project_root: impl AsRef<Path>,
    map_id: &str,
) -> anyhow::Result<WorldPlacementFiles> {
    let paths = EditorContentPaths::new(project_root.as_ref());
    let props_path = paths.props_path(map_id);
    let spawns_path = paths.spawns_path(map_id);
    let triggers_path = paths.triggers_path(map_id);

    Ok(WorldPlacementFiles {
        map_id: map_id.to_string(),
        props: game_data::loader::load_prop_list(&props_path)?,
        spawns: game_data::loader::load_spawn_list(&spawns_path)?,
        triggers: game_data::loader::load_trigger_list(&triggers_path)?,
        props_path,
        spawns_path,
        triggers_path,
    })
}

/// Save a map's voxel object list with a temp-write-then-rename and a timestamped backup.
pub fn save_voxel_objects_with_backup<T: Serialize>(
    project_root: impl AsRef<Path>,
    map_id: &str,
    value: &T,
) -> anyhow::Result<SaveOutcome> {
    let paths = EditorContentPaths::new(project_root.as_ref());
    save_ron_with_backup(paths.voxel_objects_path(map_id), value, "voxel_objects")
}

/// Save a scene's voxel object list with a temp-write-then-rename and a timestamped backup.
pub fn save_scene_voxel_objects_with_backup<T: Serialize>(
    project_root: impl AsRef<Path>,
    map_id: &str,
    value: &T,
) -> anyhow::Result<SaveOutcome> {
    let paths = EditorContentPaths::new(project_root.as_ref());
    save_ron_with_backup(
        paths.scene_voxel_objects_path(map_id),
        value,
        "scene_voxel_objects",
    )
}

pub fn save_ron_with_backup<T: Serialize>(
    path: impl AsRef<Path>,
    value: &T,
    backup_tag: &str,
) -> anyhow::Result<SaveOutcome> {
    let path = path.as_ref();
    let safe_tag = sanitize_backup_tag(backup_tag);
    let backup_path = if path.exists() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let backup_path = path.with_file_name(format!(
            "{}.{}.{}.bak.ron",
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("content"),
            safe_tag,
            timestamp
        ));
        std::fs::copy(path, &backup_path).with_context(|| {
            format!(
                "failed to create editor content backup {}",
                backup_path.display()
            )
        })?;
        Some(backup_path)
    } else {
        None
    };

    let temp_path = temp_save_path(path, &safe_tag);
    game_data::loader::save_ron_file(&temp_path, value)?;
    commit_temp_save(&temp_path, path, backup_path.as_deref())?;
    Ok(SaveOutcome {
        path: path.to_path_buf(),
        backup_path,
        temp_path: Some(temp_path),
        used_temp_write: true,
    })
}

fn sanitize_backup_tag(tag: &str) -> String {
    let safe = tag
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if safe.is_empty() {
        "editor".to_string()
    } else {
        safe
    }
}

fn temp_save_path(path: &Path, safe_tag: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    path.with_file_name(format!(
        ".{}.{}.{}.tmp.ron",
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("content"),
        safe_tag,
        timestamp
    ))
}

fn commit_temp_save(
    temp_path: &Path,
    path: &Path,
    backup_path: Option<&Path>,
) -> anyhow::Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .with_context(|| format!("failed to replace content file {}", path.display()))?;
    }

    if let Err(error) = fs::rename(temp_path, path) {
        if let Some(backup_path) = backup_path {
            let _ = fs::copy(backup_path, path);
        }
        let _ = fs::remove_file(temp_path);
        return Err(error)
            .with_context(|| format!("failed to commit temp content file {}", path.display()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_paths_are_centralized() {
        let paths = EditorContentPaths::new(PathBuf::from("project"));

        assert_eq!(
            paths.map_layers_path("starter_farm"),
            PathBuf::from("project/content/maps/starter_farm/layers.ron")
        );
        assert_eq!(
            paths.voxel_objects_path("starter_farm"),
            PathBuf::from("project/content/maps/starter_farm/voxel_objects.ron")
        );
        assert_eq!(
            paths.scene_voxel_objects_path("starter_farm"),
            PathBuf::from("project/content/scenes/starter_farm/voxel_objects.ron")
        );
        assert_eq!(
            paths.voxel_panel_kit_dir(),
            PathBuf::from("project/content/editor_voxel_panels/panel_kits")
        );
    }

    #[test]
    fn save_ron_with_backup_uses_temp_write_and_backup() {
        let dir = std::env::temp_dir().join(format!(
            "starlight_editor_data_bridge_test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&dir).expect("test directory");
        let path = dir.join("sample.ron");

        let first =
            save_ron_with_backup(&path, &vec!["one".to_string()], "unit test").expect("first save");
        assert!(first.used_temp_write);
        assert!(first.backup_path.is_none());
        assert!(path.exists());
        assert!(!first.temp_path.expect("temp path").exists());

        let second = save_ron_with_backup(&path, &vec!["two".to_string()], "unit test")
            .expect("second save");
        assert!(second.used_temp_write);
        assert!(
            second
                .backup_path
                .as_ref()
                .is_some_and(|path| path.exists())
        );
        assert!(!second.temp_path.expect("temp path").exists());

        let loaded: Vec<String> = game_data::loader::load_ron_file(&path).expect("saved ron");
        assert_eq!(loaded, vec!["two".to_string()]);

        fs::remove_dir_all(dir).expect("cleanup test directory");
    }
}
