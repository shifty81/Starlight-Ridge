pub mod profiles;
pub mod vox_writer;

use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};

pub fn generate_phase53b_templates(project_root: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let root = project_root.as_ref();
    let character_root = root.join("content/voxels/generated_templates/characters");
    let tool_root = root.join("content/voxels/generated_templates/tools");
    fs::create_dir_all(&character_root)?;
    fs::create_dir_all(&tool_root)?;

    let mut written = Vec::new();
    for profile in profiles::default_profiles() {
        if profile.generator_kind == profiles::GeneratorKind::CharacterBase {
            anyhow::ensure!(
                profile.no_hair && profile.no_facial_hair,
                "character base profiles must be bald and clean-shaven"
            );
        }
        let model = vox_writer::placeholder_model(profile.dimensions, profile.generator_kind);
        let path = root.join(&profile.output_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, vox_writer::write_vox(&model)?)
            .with_context(|| format!("failed to write {}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}
