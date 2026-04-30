use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AssetRoot {
    path: PathBuf,
}

impl AssetRoot {
    pub fn discover(root: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = root.as_ref().join("assets");
        anyhow::ensure!(
            path.exists(),
            "assets folder not found at {}",
            path.display()
        );
        log::info!("asset root: {}", path.display());
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub mod vox;
