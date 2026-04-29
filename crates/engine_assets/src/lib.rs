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

pub mod vox {
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone)]
    pub struct VoxAssetInfo {
        pub id: String,
        pub relative_path: String,
        pub absolute_path: PathBuf,
        pub width: u32,
        pub height: u32,
        pub depth: u32,
        pub voxel_count: u32,
        pub palette_colors: u32,
    }

    #[derive(Debug, Default)]
    struct VoxParsedInfo {
        width: u32,
        height: u32,
        depth: u32,
        voxel_count: u32,
        palette_colors: u32,
    }

    pub fn scan_vox_files(project_root: impl AsRef<Path>) -> anyhow::Result<Vec<VoxAssetInfo>> {
        let project_root = project_root.as_ref();
        let search_roots = [
            project_root.join("assets").join("voxels"),
            project_root.join("assets").join("models"),
            project_root.join("content").join("voxels"),
        ];

        let mut assets = Vec::new();
        for root in search_roots {
            if root.exists() {
                scan_vox_dir(project_root, &root, &mut assets)?;
            }
        }

        assets.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
        dedupe_ids(&mut assets);
        Ok(assets)
    }

    fn scan_vox_dir(
        project_root: &Path,
        dir: &Path,
        assets: &mut Vec<VoxAssetInfo>,
    ) -> anyhow::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_vox_dir(project_root, &path, assets)?;
                continue;
            }

            let is_vox = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("vox"))
                .unwrap_or(false);
            if !is_vox {
                continue;
            }

            let parsed = parse_vox_summary(&path).unwrap_or_default();
            let relative_path = path
                .strip_prefix(project_root)
                .unwrap_or(path.as_path())
                .to_string_lossy()
                .replace('\\', "/");

            assets.push(VoxAssetInfo {
                id: asset_id_from_path(&path),
                relative_path,
                absolute_path: path,
                width: parsed.width,
                height: parsed.height,
                depth: parsed.depth,
                voxel_count: parsed.voxel_count,
                palette_colors: parsed.palette_colors,
            });
        }
        Ok(())
    }

    fn dedupe_ids(assets: &mut [VoxAssetInfo]) {
        for index in 0..assets.len() {
            let original_id = assets[index].id.clone();
            let duplicate_count = assets[..index]
                .iter()
                .filter(|asset| asset.id == original_id)
                .count();
            if duplicate_count > 0 {
                assets[index].id = format!("{}_{}", original_id, duplicate_count + 1);
            }
        }
    }

    fn asset_id_from_path(path: &Path) -> String {
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("vox_asset");
        let mut id = String::new();
        let mut last_was_separator = false;
        for ch in stem.chars().flat_map(|ch| ch.to_lowercase()) {
            if ch.is_ascii_alphanumeric() {
                id.push(ch);
                last_was_separator = false;
            } else if !last_was_separator {
                id.push('_');
                last_was_separator = true;
            }
        }
        let id = id.trim_matches('_').to_string();
        if id.is_empty() {
            "vox_asset".to_string()
        } else {
            id
        }
    }

    fn parse_vox_summary(path: &Path) -> anyhow::Result<VoxParsedInfo> {
        let bytes = fs::read(path)?;
        anyhow::ensure!(
            bytes.len() >= 8,
            "{} is too small to be a .vox file",
            path.display()
        );
        anyhow::ensure!(
            &bytes[0..4] == b"VOX ",
            "{} is missing VOX header",
            path.display()
        );

        let mut parsed = VoxParsedInfo::default();
        parse_chunks(&bytes, 8, bytes.len(), &mut parsed);
        if parsed.palette_colors == 0 {
            parsed.palette_colors = 256;
        }
        Ok(parsed)
    }

    fn parse_chunks(bytes: &[u8], mut cursor: usize, end: usize, parsed: &mut VoxParsedInfo) {
        while cursor + 12 <= end && cursor + 12 <= bytes.len() {
            let id = &bytes[cursor..cursor + 4];
            let content_size = read_u32(bytes, cursor + 4).unwrap_or(0) as usize;
            let children_size = read_u32(bytes, cursor + 8).unwrap_or(0) as usize;
            let content_start = cursor + 12;
            let content_end = content_start.saturating_add(content_size).min(bytes.len());
            let children_end = content_end.saturating_add(children_size).min(bytes.len());

            match id {
                b"SIZE" if content_start + 12 <= content_end => {
                    parsed.width = read_u32(bytes, content_start).unwrap_or(0);
                    parsed.height = read_u32(bytes, content_start + 4).unwrap_or(0);
                    parsed.depth = read_u32(bytes, content_start + 8).unwrap_or(0);
                }
                b"XYZI" if content_start + 4 <= content_end => {
                    parsed.voxel_count = read_u32(bytes, content_start).unwrap_or(0);
                }
                b"RGBA" if content_end.saturating_sub(content_start) >= 1024 => {
                    parsed.palette_colors = bytes[content_start..content_start + 1024]
                        .chunks_exact(4)
                        .filter(|rgba| rgba[3] != 0)
                        .count() as u32;
                }
                _ => {}
            }

            if children_size > 0 && content_end < children_end {
                parse_chunks(bytes, content_end, children_end, parsed);
            }

            if children_end <= cursor {
                break;
            }
            cursor = children_end;
        }
    }

    fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
        let slice = bytes.get(offset..offset + 4)?;
        Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }
}
