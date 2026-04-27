use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VoxColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VoxVoxel {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub color_index: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxModel {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub voxels: Vec<VoxVoxel>,
    pub palette: Vec<VoxColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoxAssetInfo {
    pub id: String,
    pub path: PathBuf,
    pub relative_path: String,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub voxel_count: usize,
    pub palette_colors: usize,
}

impl VoxModel {
    pub fn is_empty(&self) -> bool {
        self.voxels.is_empty()
    }
}

pub fn load_vox_file(path: impl AsRef<Path>) -> anyhow::Result<VoxModel> {
    let path = path.as_ref();
    let bytes = fs::read(path).with_context(|| format!("failed to read .vox file {}", path.display()))?;
    parse_vox_bytes(&bytes).with_context(|| format!("failed to parse .vox file {}", path.display()))
}

pub fn parse_vox_bytes(bytes: &[u8]) -> anyhow::Result<VoxModel> {
    if bytes.len() < 8 || &bytes[0..4] != b"VOX " {
        bail!("missing VOX magic header");
    }

    let _version = read_u32(bytes, 4)?;
    let mut state = VoxParseState::default();
    let mut cursor = 8;
    while cursor < bytes.len() {
        cursor = parse_chunk(bytes, cursor, &mut state)?;
    }

    let (width, height, depth) = state.size.unwrap_or((0, 0, 0));
    if width == 0 || height == 0 || depth == 0 {
        bail!(".vox file did not contain a valid SIZE chunk");
    }

    if state.voxels.is_empty() {
        bail!(".vox file did not contain voxel data in an XYZI chunk");
    }

    Ok(VoxModel {
        width,
        height,
        depth,
        voxels: state.voxels,
        palette: state.palette.unwrap_or_else(default_palette),
    })
}

pub fn scan_vox_files(project_root: impl AsRef<Path>) -> anyhow::Result<Vec<VoxAssetInfo>> {
    let project_root = project_root.as_ref();
    let search_roots = [
        project_root.join("assets").join("voxels"),
        project_root.join("assets").join("models"),
        project_root.join("content").join("voxels"),
    ];

    let mut files = Vec::new();
    for root in search_roots {
        collect_vox_files(&root, &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut assets = Vec::new();
    for path in files {
        match load_vox_file(&path) {
            Ok(model) => {
                let relative_path = path
                    .strip_prefix(project_root)
                    .unwrap_or(path.as_path())
                    .to_string_lossy()
                    .replace('\\', "/");
                let mut id = path
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("vox_model")
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
                    .collect::<String>();
                id = id.trim_matches('_').to_string();

                assets.push(VoxAssetInfo {
                    id: if id.is_empty() { "vox_model".to_string() } else { id },
                    path,
                    relative_path,
                    width: model.width,
                    height: model.height,
                    depth: model.depth,
                    voxel_count: model.voxels.len(),
                    palette_colors: model.palette.len(),
                });
            }
            Err(error) => {
                log::warn!("skipping invalid .vox asset {}: {error:#}", path.display());
            }
        }
    }

    Ok(assets)
}

#[derive(Default)]
struct VoxParseState {
    size: Option<(u32, u32, u32)>,
    voxels: Vec<VoxVoxel>,
    palette: Option<Vec<VoxColor>>,
}

fn collect_vox_files(root: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(root).with_context(|| format!("failed to scan {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_vox_files(&path, files)?;
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("vox"))
        {
            files.push(path);
        }
    }

    Ok(())
}

fn parse_chunk(bytes: &[u8], offset: usize, state: &mut VoxParseState) -> anyhow::Result<usize> {
    if offset + 12 > bytes.len() {
        bail!("truncated chunk header at byte {offset}");
    }

    let id = &bytes[offset..offset + 4];
    let content_size = read_u32(bytes, offset + 4)? as usize;
    let children_size = read_u32(bytes, offset + 8)? as usize;
    let content_start = offset + 12;
    let content_end = content_start
        .checked_add(content_size)
        .context("chunk content size overflow")?;
    let children_end = content_end
        .checked_add(children_size)
        .context("chunk child size overflow")?;

    if children_end > bytes.len() {
        bail!("chunk extends past end of file");
    }

    match id {
        b"SIZE" => parse_size_chunk(&bytes[content_start..content_end], state)?,
        b"XYZI" => parse_xyzi_chunk(&bytes[content_start..content_end], state)?,
        b"RGBA" => parse_rgba_chunk(&bytes[content_start..content_end], state)?,
        _ => {}
    }

    let mut child_cursor = content_end;
    while child_cursor < children_end {
        child_cursor = parse_chunk(bytes, child_cursor, state)?;
    }

    Ok(children_end)
}

fn parse_size_chunk(content: &[u8], state: &mut VoxParseState) -> anyhow::Result<()> {
    if content.len() < 12 {
        bail!("SIZE chunk is too small");
    }
    let width = read_i32(content, 0)?;
    let height = read_i32(content, 4)?;
    let depth = read_i32(content, 8)?;
    if width <= 0 || height <= 0 || depth <= 0 {
        bail!("SIZE chunk contains non-positive dimensions");
    }
    state.size = Some((width as u32, height as u32, depth as u32));
    Ok(())
}

fn parse_xyzi_chunk(content: &[u8], state: &mut VoxParseState) -> anyhow::Result<()> {
    if content.len() < 4 {
        bail!("XYZI chunk is too small");
    }
    let count = read_u32(content, 0)? as usize;
    let expected = 4usize
        .checked_add(count.checked_mul(4).context("XYZI voxel count overflow")?)
        .context("XYZI size overflow")?;
    if content.len() < expected {
        bail!("XYZI chunk expected {expected} bytes but found {}", content.len());
    }

    state.voxels.clear();
    state.voxels.reserve(count);
    let mut cursor = 4;
    for _ in 0..count {
        state.voxels.push(VoxVoxel {
            x: content[cursor],
            y: content[cursor + 1],
            z: content[cursor + 2],
            color_index: content[cursor + 3],
        });
        cursor += 4;
    }
    Ok(())
}

fn parse_rgba_chunk(content: &[u8], state: &mut VoxParseState) -> anyhow::Result<()> {
    if content.len() < 256 * 4 {
        bail!("RGBA chunk is too small");
    }

    let mut palette = Vec::with_capacity(256);
    for index in 0..256 {
        let cursor = index * 4;
        palette.push(VoxColor {
            r: content[cursor],
            g: content[cursor + 1],
            b: content[cursor + 2],
            a: content[cursor + 3],
        });
    }
    state.palette = Some(palette);
    Ok(())
}

fn read_u32(bytes: &[u8], offset: usize) -> anyhow::Result<u32> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| anyhow::anyhow!("unexpected EOF while reading u32 at byte {offset}"))?;
    Ok(u32::from_le_bytes(slice.try_into().expect("slice length checked")))
}

fn read_i32(bytes: &[u8], offset: usize) -> anyhow::Result<i32> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| anyhow::anyhow!("unexpected EOF while reading i32 at byte {offset}"))?;
    Ok(i32::from_le_bytes(slice.try_into().expect("slice length checked")))
}

fn default_palette() -> Vec<VoxColor> {
    let mut palette = Vec::with_capacity(256);
    palette.push(VoxColor { r: 0, g: 0, b: 0, a: 0 });
    for index in 1..=255u8 {
        palette.push(VoxColor {
            r: index,
            g: index,
            b: index,
            a: 255,
        });
    }
    palette
}
