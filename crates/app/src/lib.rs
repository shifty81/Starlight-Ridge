use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use engine_assets::vox::load_vox_file;
use engine_assets::AssetRoot;
use engine_audio::AudioBootstrap;
use engine_debug::DebugOverlayState;
use engine_input::{InputSnapshot, handle_keyboard_event};
use engine_render_gl::{
    EditorShellRenderState, RenderBootstrap, SpriteInstance, SpriteRenderData, TileInstance,
    TileMapRenderData, VoxelSceneRenderData, VoxelVertex, WorldLighting,
};
use engine_time::FrameTimer;
use engine_window::{WindowConfig, create_gl_window};
use game_core::modes::bootstrap_state;
use game_core::state::InteractionMode;
use game_data::defs::{MapLayersDef, SpriteSheetDef, TileLayerDef, TilesetDef};
use game_data::registry::ContentRegistry;
use game_world::autotile::{
    AutotileResolver, ResolvedTileKind, TerrainFlags, TerrainResolveCatalog, TerrainTransitionRule,
    TerrainVariantChoice, TerrainVariantSet,
};
use game_world::{SemanticTerrainCell, SemanticTerrainGrid, WorldBootstrap};
use serde::Deserialize;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

mod egui_editor;

#[derive(Debug, Clone, Deserialize)]
struct VoxelObjectPlacementList {
    schema_version: u32,
    map_id: String,
    objects: Vec<VoxelObjectPlacement>,
}

#[derive(Debug, Clone, Deserialize)]
struct VoxelObjectPlacement {
    id: String,
    display_name: String,
    source_kind: String,
    source_id: String,
    source_path: String,
    x: f32,
    y: f32,
    z: f32,
    yaw_degrees: f32,
    footprint_width: f32,
    footprint_height: f32,
    height: f32,
    anchor_x: f32,
    anchor_y: f32,
    collision_kind: String,
    locked: bool,
    notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaunchMode {
    Game,
    Editor,
}

impl LaunchMode {
    fn window_title(self) -> &'static str {
        match self {
            Self::Game => "Starlight Ridge",
            Self::Editor => "Starlight Ridge Editor",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Game => "game",
            Self::Editor => "editor",
        }
    }
}

/// Initializes process-wide runtime logging for the game/editor binaries.
pub fn init_runtime_logging(label: &str) {
    let mut builder =
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"));
    builder.format_timestamp_secs();
    builder.format_target(false);

    if let Err(error) = builder.try_init() {
        // Multiple bins/tests can initialize logging in the same process. Do not
        // fail startup if a logger already exists.
        eprintln!("Starlight Ridge logging already initialized for {label}: {error}");
    }

    install_runtime_panic_hook(label);

    match runtime_logs_dir() {
        Ok(log_dir) => log::info!(
            "runtime logging ready: label={} logs={}",
            label,
            log_dir.display()
        ),
        Err(error) => {
            eprintln!("Starlight Ridge could not prepare runtime logs for {label}: {error:#}")
        }
    }
}

/// Writes the latest startup/runtime failure into the project /logs folder and shows
/// a visible Windows dialog so double-click launches do not fail silently.
pub fn write_runtime_failure(label: &str, error: &anyhow::Error) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let body = format!(
        "Starlight Ridge runtime failure\nlabel: {label}\nunix_timestamp: {timestamp}\n\n{error:#}\n"
    );
    write_runtime_failure_text(label, &body);
}

fn write_runtime_failure_text(label: &str, body: &str) {
    log::error!("{label} runtime failure: {body}");

    let mut latest_path_for_dialog = None;
    match runtime_logs_dir() {
        Ok(log_dir) => {
            let latest_path = log_dir.join(format!("{label}_runtime_failure_latest.log"));
            if let Err(write_error) = fs::write(&latest_path, body) {
                eprintln!(
                    "Starlight Ridge could not write {}: {write_error:#}",
                    latest_path.display()
                );
            } else {
                latest_path_for_dialog = Some(latest_path);
            }

            let shared_latest_path = log_dir.join("latest_runtime_failure.log");
            if let Err(write_error) = fs::write(&shared_latest_path, body) {
                eprintln!(
                    "Starlight Ridge could not write {}: {write_error:#}",
                    shared_latest_path.display()
                );
            }
        }
        Err(log_dir_error) => {
            eprintln!("Starlight Ridge runtime failure for {label}: {body}");
            eprintln!("Could not prepare runtime log directory: {log_dir_error:#}");
        }
    }

    show_runtime_failure_dialog(label, body, latest_path_for_dialog.as_deref());
}

fn install_runtime_panic_hook(label: &str) {
    let label = label.to_string();
    std::panic::set_hook(Box::new(move |panic_info| {
        let location = panic_info
            .location()
            .map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            })
            .unwrap_or_else(|| "unknown location".to_string());

        let payload = panic_info
            .payload()
            .downcast_ref::<&str>()
            .map(|message| (*message).to_string())
            .or_else(|| panic_info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic payload".to_string());

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);
        let body = format!(
            "Starlight Ridge panic\nlabel: {}\nunix_timestamp: {}\nlocation: {}\n\n{}\n",
            label, timestamp, location, payload
        );
        write_runtime_failure_text(&label, &body);
    }));
}

#[cfg(target_os = "windows")]
fn show_runtime_failure_dialog(label: &str, body: &str, log_path: Option<&Path>) {
    let title = format!("Starlight Ridge {} failed", label);
    let log_hint = log_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "logs/latest_runtime_failure.log".to_string());
    let message = format!(
        "Starlight Ridge {} failed.\n\nA diagnostic log was written to:\n{}\n\n{}",
        label,
        log_hint,
        truncate_for_dialog(body, 1600),
    );

    let escape = |text: &str| text.replace('`', "``").replace('\'', "''");
    let script = format!(
        "Add-Type -AssemblyName PresentationFramework; [System.Windows.MessageBox]::Show('{}', '{}', 'OK', 'Error') | Out-Null",
        escape(&message),
        escape(&title),
    );

    if Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .spawn()
        .is_err()
    {
        eprintln!("{title}: {message}");
    }
}

#[cfg(not(target_os = "windows"))]
fn show_runtime_failure_dialog(label: &str, body: &str, log_path: Option<&Path>) {
    eprintln!(
        "Starlight Ridge {} failed. Log: {}\n{}",
        label,
        log_path
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "logs/latest_runtime_failure.log".to_string()),
        truncate_for_dialog(body, 1600),
    );
}

fn truncate_for_dialog(body: &str, max_chars: usize) -> String {
    if body.chars().count() <= max_chars {
        return body.to_string();
    }

    let mut text = body.chars().take(max_chars).collect::<String>();
    text.push_str("\n... truncated; see full log file ...");
    text
}

fn runtime_logs_dir() -> anyhow::Result<PathBuf> {
    let project_root = locate_project_root().or_else(|_| {
        std::env::current_dir().context("failed to locate project root or current directory")
    })?;
    let log_dir = project_root.join("logs");
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("failed to create log directory {}", log_dir.display()))?;
    Ok(log_dir)
}

pub fn run() -> anyhow::Result<()> {
    run_with_mode(LaunchMode::Game)
}

pub fn run_editor() -> anyhow::Result<()> {
    egui_editor::run_editor_egui()
}

/// Keeps the pre-egui OpenGL overlay editor available for renderer debugging.
/// The normal `editor.exe` path now uses egui.
pub fn run_legacy_gl_editor() -> anyhow::Result<()> {
    editor_core::init().context("failed to initialize editor core")?;
    run_with_mode(LaunchMode::Editor)
}

fn run_with_mode(launch_mode: LaunchMode) -> anyhow::Result<()> {
    let project_root = locate_project_root()?;
    let registry = game_data::load_registry(&project_root)?;
    let mut boot_state = bootstrap_state(&registry);

    // Phase 21 restores the authored starter farm as the normal launch map while
    // keeping the autotile test scenes available through content for diagnostics.
    if registry.maps.contains_key("starter_farm") {
        boot_state.active_map_id = "starter_farm".to_string();
    } else if registry.maps.contains_key("autotile_test_coast") {
        boot_state.active_map_id = "autotile_test_coast".to_string();
    }

    if launch_mode == LaunchMode::Editor {
        boot_state.interaction_mode = InteractionMode::Edit;
    }

    let _world = WorldBootstrap::new(boot_state.active_map_id.clone());
    let _assets = AssetRoot::discover(&project_root)?;
    let _audio = AudioBootstrap::new();
    let _debug = DebugOverlayState::new();
    let tile_map = build_tile_map_render_data(&project_root, &registry, &boot_state.active_map_id)
        .with_context(|| {
            format!(
                "failed to build render data for map '{}'",
                boot_state.active_map_id
            )
        })?;

    let player_sprite_data =
        build_sprite_render_data(&project_root, &registry, "phase5_entities", 32, 48);
    let prop_sprite_data = build_sprite_render_data(
        &project_root,
        &registry,
        "oceans_heart_bridge_phase17",
        32,
        32,
    );
    let game_runtime =
        RuntimeWorldState::new(&registry, &boot_state.active_map_id, tile_map.as_ref());
    let static_prop_sprites =
        build_static_prop_instances(&registry, &boot_state.active_map_id, tile_map.as_ref());
    let voxel_scene =
        build_voxel_scene_render_data(&project_root, &boot_state.active_map_id, tile_map.as_ref())
            .with_context(|| {
                format!(
                    "failed to build voxel scene render data for map '{}'",
                    boot_state.active_map_id
                )
            })?;

    log::info!(
        "phase bootstrap ready: launch={} mode={:?} interaction={:?} map={} content=[{}] tilemap={}",
        launch_mode.label(),
        boot_state.app_mode,
        boot_state.interaction_mode,
        boot_state.active_map_id,
        registry.summary(),
        tile_map.as_ref().map(|map| map.tiles.len()).unwrap_or(0)
    );

    if launch_mode == LaunchMode::Editor {
        write_editor_live_preview_manifest(&project_root, &boot_state.active_map_id)
            .context("failed to write editor live-preview manifest")?;
    }

    let event_loop = EventLoop::new().context("failed to create winit event loop")?;
    let mut app = BootstrapApp::new(
        project_root,
        registry,
        launch_mode,
        boot_state.active_map_id.clone(),
        tile_map,
        player_sprite_data,
        prop_sprite_data,
        static_prop_sprites,
        voxel_scene,
        game_runtime,
    );
    event_loop.run_app(&mut app).context("app loop failed")?;

    if let Some(error) = app.take_fatal_error() {
        anyhow::bail!("{}", error);
    }

    Ok(())
}

fn locate_project_root() -> anyhow::Result<PathBuf> {
    let candidate = std::env::current_dir()?.canonicalize()?;
    if candidate.join("content").exists() {
        return Ok(candidate);
    }

    if let Some(parent) = candidate
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
    {
        if parent.join("content").exists() {
            return Ok(parent);
        }
    }

    anyhow::bail!(
        "could not locate project root containing /content from {}",
        candidate.display()
    )
}

fn write_editor_live_preview_manifest(
    project_root: &Path,
    active_map_id: &str,
) -> anyhow::Result<()> {
    let manifest_path = project_root
        .join("artifacts")
        .join("editor_live_preview.ron");
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let body = format!(
        "(
    active_map: \"{}\",
    asset_lab: \"tools/asset_lab.html\",
    hot_reload: true,
    watched_roots: [
        \"assets/textures\",
        \"content/maps\",
        \"content/tiles\",
        \"content/metadata\",
    ],
)
",
        active_map_id
    );
    fs::write(&manifest_path, body)
        .with_context(|| format!("failed to write {}", manifest_path.display()))?;
    log::info!(
        "editor live-preview manifest updated: {}",
        manifest_path.display()
    );
    Ok(())
}

#[derive(Debug)]
struct AssetWatchState {
    watched_roots: Vec<PathBuf>,
    last_signature: Vec<(PathBuf, Option<SystemTime>, u64)>,
    last_check: Instant,
    check_interval: Duration,
}

impl AssetWatchState {
    fn new(project_root: &Path) -> Self {
        let watched_roots = [
            "assets/textures",
            "content/maps",
            "content/tiles",
            "content/metadata",
        ]
        .into_iter()
        .map(|relative| project_root.join(relative))
        .collect::<Vec<_>>();

        let last_signature = collect_asset_signature(&watched_roots);
        Self {
            watched_roots,
            last_signature,
            last_check: Instant::now(),
            check_interval: Duration::from_millis(750),
        }
    }

    fn should_check(&mut self) -> bool {
        if self.last_check.elapsed() < self.check_interval {
            return false;
        }
        self.last_check = Instant::now();
        true
    }

    fn has_changed(&mut self) -> bool {
        let current = collect_asset_signature(&self.watched_roots);
        if current != self.last_signature {
            self.last_signature = current;
            return true;
        }
        false
    }

    fn reset(&mut self) {
        self.last_signature = collect_asset_signature(&self.watched_roots);
        self.last_check = Instant::now();
    }
}

fn collect_asset_signature(roots: &[PathBuf]) -> Vec<(PathBuf, Option<SystemTime>, u64)> {
    let mut files = Vec::new();
    for root in roots {
        collect_asset_signature_recursive(root, root, &mut files);
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

fn collect_asset_signature_recursive(
    _root: &Path,
    path: &Path,
    files: &mut Vec<(PathBuf, Option<SystemTime>, u64)>,
) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.is_dir() {
            collect_asset_signature_recursive(_root, &path, files);
        } else if is_live_preview_file(&path) {
            files.push((path, metadata.modified().ok(), metadata.len()));
        }
    }
}

fn is_live_preview_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()).map(|extension| extension.to_ascii_lowercase()),
        Some(extension) if matches!(extension.as_str(), "png" | "ron" | "json" | "toml")
    )
}

#[derive(Debug, Clone)]
struct TileRoleState {
    role: String,
    collision: String,
    walkable: bool,
    blocks_movement: bool,
    water: bool,
    interactable: bool,
    crop_soil: bool,
    door: bool,
}

impl TileRoleState {
    fn inferred(tile_id: &str) -> Self {
        let role = if tile_id.contains("water")
            || tile_id.starts_with("shore")
            || tile_id.starts_with("depth")
        {
            "water"
        } else if tile_id.contains("tilled") {
            "crop_soil"
        } else if tile_id.contains("sand") {
            "sand"
        } else if tile_id.contains("path") || tile_id == "dirt" {
            "path"
        } else if tile_id.contains("wood") {
            "wood"
        } else if tile_id.contains("stone") || tile_id.contains("rock") {
            "stone"
        } else if tile_id.contains("fence") {
            "fence"
        } else if tile_id.contains("door") || tile_id.contains("gate") {
            "door"
        } else if tile_id.contains("roof")
            || tile_id.contains("wall")
            || tile_id.contains("farmhouse")
        {
            "building"
        } else if tile_id.contains("tree")
            || tile_id.contains("boulder")
            || tile_id.contains("barrel")
            || tile_id.contains("crate")
        {
            "blocking_prop"
        } else if tile_id.contains("grass") {
            "grass"
        } else {
            "decor"
        };
        Self::from_role(role)
    }

    fn from_role(role: &str) -> Self {
        match role {
            "water" => Self::from_collision(role, "water"),
            "crop_soil" => {
                let mut state = Self::from_collision(role, "walkable");
                state.crop_soil = true;
                state.interactable = true;
                state
            }
            "building" | "blocking_prop" | "fence" => Self::from_collision(role, "blocked"),
            "door" => {
                let mut state = Self::from_collision(role, "door");
                state.door = true;
                state.interactable = true;
                state
            }
            other => Self::from_collision(other, "walkable"),
        }
    }

    fn from_collision(role: &str, collision: &str) -> Self {
        let mut state = Self {
            role: role.to_string(),
            collision: collision.to_string(),
            walkable: true,
            blocks_movement: false,
            water: false,
            interactable: false,
            crop_soil: false,
            door: false,
        };

        match collision {
            "blocked" => {
                state.walkable = false;
                state.blocks_movement = true;
            }
            "water" => {
                state.walkable = false;
                state.blocks_movement = true;
                state.water = true;
            }
            "interactable" => {
                state.interactable = true;
            }
            "crop_soil" => {
                state.crop_soil = true;
                state.interactable = true;
            }
            "door" => {
                state.door = true;
                state.interactable = true;
            }
            _ => {}
        }

        state
    }

    fn entry(&self, tile_id: &str) -> String {
        format!(
            "        (tile_id: \"{}\", role: \"{}\", collision: \"{}\", walkable: {}, blocks_movement: {}, water: {}, interactable: {}, crop_soil: {}, door: {}),",
            tile_id,
            self.role,
            self.collision,
            self.walkable,
            self.blocks_movement,
            self.water,
            self.interactable,
            self.crop_soil,
            self.door
        )
    }
}

const EDITOR_ROLE_CYCLE: &[&str] = &[
    "grass",
    "path",
    "sand",
    "water",
    "crop_soil",
    "stone",
    "wood",
    "building",
    "decor",
    "blocking_prop",
    "fence",
    "door",
];

const EDITOR_COLLISION_CYCLE: &[&str] = &[
    "walkable",
    "blocked",
    "water",
    "interactable",
    "crop_soil",
    "door",
];

fn next_cycle_value(current: &str, values: &[&str]) -> String {
    let index = values
        .iter()
        .position(|value| *value == current)
        .unwrap_or(0);
    values[(index + 1) % values.len()].to_string()
}

fn load_tile_role_state(project_root: &Path, tile_id: &str) -> TileRoleState {
    let path = project_root
        .join("content")
        .join("tiles")
        .join("base_tileset_roles.ron");
    let Ok(body) = fs::read_to_string(&path) else {
        return TileRoleState::inferred(tile_id);
    };

    let Some((start, end)) = find_role_entry_range(&body, tile_id) else {
        return TileRoleState::inferred(tile_id);
    };

    let block = &body[start..end];
    let role = extract_quoted_field(block, "role")
        .unwrap_or_else(|| TileRoleState::inferred(tile_id).role);
    let collision = extract_quoted_field(block, "collision")
        .unwrap_or_else(|| TileRoleState::from_role(&role).collision);

    let mut state = TileRoleState::from_collision(&role, &collision);
    state.walkable = extract_bool_field(block, "walkable").unwrap_or(state.walkable);
    state.blocks_movement =
        extract_bool_field(block, "blocks_movement").unwrap_or(state.blocks_movement);
    state.water = extract_bool_field(block, "water").unwrap_or(state.water);
    state.interactable = extract_bool_field(block, "interactable").unwrap_or(state.interactable);
    state.crop_soil = extract_bool_field(block, "crop_soil").unwrap_or(state.crop_soil);
    state.door = extract_bool_field(block, "door").unwrap_or(state.door);
    state
}

fn save_tile_role_state(
    project_root: &Path,
    tile_id: &str,
    state: &TileRoleState,
) -> anyhow::Result<()> {
    let path = project_root
        .join("content")
        .join("tiles")
        .join("base_tileset_roles.ron");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut body = if path.exists() {
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?
    } else {
        "(\n    tileset: \"base_tiles\",\n    source: \"content/tiles/base_tileset.ron\",\n    entries: [\n    ],\n)\n".to_string()
    };

    if path.exists() {
        let backup = path.with_extension("ron.phase31.bak");
        let _ = fs::copy(&path, &backup);
    }

    let entry = state.entry(tile_id);
    if let Some((start, end)) = find_role_entry_range(&body, tile_id) {
        body.replace_range(start..end, &entry);
    } else if let Some(insert_at) = body.rfind("    ],") {
        body.insert_str(insert_at, &format!("{}\n", entry));
    } else {
        body.push_str(&format!("\n{}\n", entry));
    }

    fs::write(&path, body).with_context(|| format!("failed to write {}", path.display()))?;
    log::info!(
        "native Asset Studio saved tile role metadata: tile={} role={} collision={} file={}",
        tile_id,
        state.role,
        state.collision,
        path.display()
    );
    Ok(())
}

fn find_role_entry_range(body: &str, tile_id: &str) -> Option<(usize, usize)> {
    let needle = format!("tile_id: \"{}\"", tile_id);
    let tile_pos = body.find(&needle)?;
    let start = body[..tile_pos]
        .rfind("        (")
        .or_else(|| body[..tile_pos].rfind("("))?;
    let relative_end = body[tile_pos..].find("),")?;
    Some((start, tile_pos + relative_end + 2))
}

fn extract_quoted_field(block: &str, field: &str) -> Option<String> {
    let needle = format!("{}: \"", field);
    let start = block.find(&needle)? + needle.len();
    let end = block[start..].find('"')?;
    Some(block[start..start + end].to_string())
}

fn extract_bool_field(block: &str, field: &str) -> Option<bool> {
    let needle = format!("{}: ", field);
    let start = block.find(&needle)? + needle.len();
    let tail = &block[start..];
    if tail.starts_with("true") {
        Some(true)
    } else if tail.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

#[derive(Debug, Default)]
struct TileBuildStats {
    terrain_layers: usize,
    base_cells: usize,
    transition_overlays: usize,
    direct_cells: usize,
    missing_refs: usize,
}

fn build_tile_map_render_data(
    project_root: &Path,
    registry: &ContentRegistry,
    map_id: &str,
) -> anyhow::Result<Option<TileMapRenderData>> {
    let Some(map_bundle) = registry.maps.get(map_id) else {
        log::warn!(
            "no map bundle found for '{}'; renderer will use fallback grid",
            map_id
        );
        return Ok(None);
    };
    let Some(layers) = registry.map_layers.get(map_id) else {
        log::warn!(
            "no layers.ron found for '{}'; renderer will use fallback grid",
            map_id
        );
        return Ok(None);
    };
    let Some(tileset) = registry.tilesets.get(&map_bundle.metadata.tileset) else {
        log::warn!(
            "tileset '{}' not found for '{}'; renderer will use fallback grid",
            map_bundle.metadata.tileset,
            map_id
        );
        return Ok(None);
    };

    let atlas_lookup: HashMap<&str, (u32, u32)> = tileset
        .named_tiles
        .iter()
        .map(|entry| (entry.id.as_str(), (entry.x, entry.y)))
        .collect();
    let known_tile_ids = tileset
        .named_tiles
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<HashSet<_>>();
    let terrain_resolve_catalog = build_terrain_resolve_catalog(registry, tileset, &known_tile_ids);

    let map_width = map_bundle
        .metadata
        .width
        .max(max_layer_width(layers))
        .max(1);
    let map_height = map_bundle
        .metadata
        .height
        .max(max_layer_height(layers))
        .max(1);

    let mut tiles = Vec::new();
    let mut stats = TileBuildStats::default();

    for layer in layers.layers.iter().filter(|layer| layer.visible) {
        let tile_grid = layer_to_tile_grid(layer, map_width, map_height);

        if is_contract_semantic_layer(layer, registry) {
            stats.terrain_layers += 1;
            if let Some(catalog) = terrain_resolve_catalog.as_ref() {
                push_contract_semantic_layer_tiles(
                    &mut tiles,
                    layer,
                    map_width,
                    map_height,
                    registry,
                    catalog,
                    &atlas_lookup,
                    &mut stats,
                    map_id,
                )?;
            } else {
                log::warn!(
                    "map '{}' layer '{}' is semantic terrain but no Phase 18 resolve catalog is loaded",
                    map_id,
                    layer.id
                );
            }
        } else if is_semantic_terrain_layer(layer, &tile_grid) {
            stats.terrain_layers += 1;
            push_terrain_layer_tiles(
                &mut tiles,
                &tile_grid,
                &atlas_lookup,
                &mut stats,
                map_id,
                &layer.id,
            );
        } else {
            push_direct_layer_tiles(
                &mut tiles,
                &tile_grid,
                &atlas_lookup,
                &mut stats,
                map_id,
                &layer.id,
            );
        }
    }

    if tiles.is_empty() {
        log::warn!(
            "map '{}' produced zero visible tiles; renderer will use fallback grid",
            map_id
        );
        return Ok(None);
    }

    let texture_path = project_root.join(&tileset.texture_path);
    log::info!(
        "prepared packed base tile renderer data: map={} draws={} terrain_layers={} base_cells={} transition_overlays={} direct_cells={} missing_refs={} texture={}",
        map_id,
        tiles.len(),
        stats.terrain_layers,
        stats.base_cells,
        stats.transition_overlays,
        stats.direct_cells,
        stats.missing_refs,
        texture_path.display()
    );

    Ok(Some(TileMapRenderData {
        texture_path,
        map_width,
        map_height,
        tile_width: layers.tile_width.max(1),
        tile_height: layers.tile_height.max(1),
        atlas_columns: tileset.columns.max(1),
        atlas_rows: tileset.rows.max(1),
        tiles,
    }))
}

fn build_sprite_render_data(
    project_root: &Path,
    registry: &ContentRegistry,
    sheet_id: &str,
    sprite_width: u32,
    sprite_height: u32,
) -> Option<SpriteRenderData> {
    let sheet = registry.sprite_sheets.get(sheet_id)?;
    Some(sprite_render_data_from_sheet(
        project_root,
        sheet,
        sprite_width,
        sprite_height,
    ))
}

fn sprite_render_data_from_sheet(
    project_root: &Path,
    sheet: &SpriteSheetDef,
    sprite_width: u32,
    sprite_height: u32,
) -> SpriteRenderData {
    SpriteRenderData {
        texture_path: project_root.join(&sheet.texture_path),
        sprite_width: sprite_width.max(1),
        sprite_height: sprite_height.max(1),
        atlas_columns: sheet.columns.max(1),
        atlas_rows: sheet.rows.max(1),
    }
}

fn build_static_prop_instances(
    registry: &ContentRegistry,
    map_id: &str,
    tile_map: Option<&TileMapRenderData>,
) -> Vec<SpriteInstance> {
    let Some(map) = registry.maps.get(map_id) else {
        return Vec::new();
    };
    let Some(sheet) = registry.sprite_sheets.get("oceans_heart_bridge_phase17") else {
        return Vec::new();
    };
    let Some(tile_map) = tile_map else {
        return Vec::new();
    };

    let entry_lookup = sheet
        .entries
        .iter()
        .map(|entry| (entry.id.as_str(), (entry.x, entry.y)))
        .collect::<HashMap<_, _>>();

    let tile_width = tile_map.tile_width.max(1) as f32;
    let tile_height = tile_map.tile_height.max(1) as f32;

    map.props
        .iter()
        .filter_map(|prop| {
            let (atlas_x, atlas_y) = prop_atlas_cell(&prop.kind, &entry_lookup)?;
            Some(SpriteInstance {
                x: prop.x.max(0) as f32 * tile_width,
                y: prop.y.max(0) as f32 * tile_height,
                w: tile_width,
                h: tile_height,
                atlas_x,
                atlas_y,
            })
        })
        .collect()
}

fn prop_atlas_cell(kind: &str, entry_lookup: &HashMap<&str, (u32, u32)>) -> Option<(u32, u32)> {
    let preferred = match kind {
        "seagull" => "seagull_idle",
        "weak_tree" => "weak_tree_full",
        other => other,
    };
    entry_lookup.get(preferred).copied()
}

fn build_voxel_scene_render_data(
    project_root: &Path,
    map_id: &str,
    tile_map: Option<&TileMapRenderData>,
) -> anyhow::Result<Option<VoxelSceneRenderData>> {
    let Some(tile_map) = tile_map else {
        return Ok(None);
    };

    let voxel_objects_path = project_root
        .join("content")
        .join("maps")
        .join(map_id)
        .join("voxel_objects.ron");
    if !voxel_objects_path.exists() {
        return Ok(None);
    }

    let placements: VoxelObjectPlacementList =
        game_data::loader::load_ron_file(&voxel_objects_path)
            .with_context(|| format!("failed to load {}", voxel_objects_path.display()))?;
    if placements.objects.is_empty() {
        return Ok(None);
    }

    let tile_width = tile_map.tile_width.max(1) as f32;
    let tile_height = tile_map.tile_height.max(1) as f32;
    let world_origin_x = -((tile_map.map_width.max(1) * tile_map.tile_width.max(1)) as f32) * 0.5;
    let world_origin_y = -((tile_map.map_height.max(1) * tile_map.tile_height.max(1)) as f32) * 0.5;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut bounds_min = glam::Vec3::splat(f32::INFINITY);
    let mut bounds_max = glam::Vec3::splat(f32::NEG_INFINITY);

    for object in &placements.objects {
        let x0 = world_origin_x + object.x * tile_width;
        let y0 = world_origin_y + object.y * tile_height;
        let z0 = object.z.max(0.0) * tile_height;
        let x1 = x0 + object.footprint_width.max(0.25) * tile_width;
        let y1 = y0 + object.footprint_height.max(0.25) * tile_height;
        let z1 = z0 + object.height.max(0.25) * tile_height;

        let min = glam::Vec3::new(x0.min(x1), y0.min(y1), z0.min(z1));
        let max = glam::Vec3::new(x0.max(x1), y0.max(y1), z0.max(z1));
        bounds_min = bounds_min.min(min);
        bounds_max = bounds_max.max(max);

        // Try loading the actual .vox source file for real voxel geometry.
        let vox_path = if object.source_path.is_empty() {
            None
        } else {
            Some(project_root.join(&object.source_path))
        };
        let used_real_mesh = vox_path
            .as_deref()
            .filter(|p| p.exists())
            .and_then(|p| load_vox_file(p).ok())
            .map(|model| {
                push_vox_model_mesh(
                    &mut vertices,
                    &mut indices,
                    &model,
                    min,
                    max,
                );
            })
            .is_some();

        if !used_real_mesh {
            let base_color = color_for_voxel_object(object);
            push_voxel_prism_mesh(&mut vertices, &mut indices, min, max, base_color);
        }
    }

    if vertices.is_empty() || indices.is_empty() {
        return Ok(None);
    }

    log::info!(
        "voxel scene ready: map={} schema={} placements={} bounds=({:.1},{:.1},{:.1})..({:.1},{:.1},{:.1})",
        placements.map_id,
        placements.schema_version,
        placements.objects.len(),
        bounds_min.x,
        bounds_min.y,
        bounds_min.z,
        bounds_max.x,
        bounds_max.y,
        bounds_max.z,
    );

    Ok(Some(VoxelSceneRenderData {
        vertices,
        indices,
        bounds_min: bounds_min.to_array(),
        bounds_max: bounds_max.to_array(),
    }))
}

fn color_for_voxel_object(object: &VoxelObjectPlacement) -> [f32; 4] {
    let mut seed = 0_u32;
    for byte in object
        .source_id
        .bytes()
        .chain(object.display_name.bytes())
        .chain(object.id.bytes())
        .chain(object.source_path.bytes())
    {
        seed = seed.wrapping_mul(16777619).wrapping_add(byte as u32 + 1);
    }

    let tint = if object.locked { 0.72 } else { 1.0 };
    let source_boost = if object.source_kind.contains("phase52") {
        [0.08, 0.12, 0.0]
    } else {
        [0.0, 0.06, 0.12]
    };
    let collision_boost = if object.collision_kind.contains("solid") {
        [0.10, 0.02, 0.02]
    } else {
        [0.0, 0.03, 0.0]
    };
    let note_boost = (object.notes.len() as f32 / 240.0).clamp(0.0, 0.08);
    let anchor_bias =
        ((object.anchor_x + object.anchor_y + object.yaw_degrees.abs()) / 360.0).clamp(0.0, 0.12);
    let mut color = [
        (0.30 + ((seed >> 0 & 0xFF) as f32 / 255.0) * 0.45) * tint
            + source_boost[0]
            + collision_boost[0],
        (0.32 + ((seed >> 8 & 0xFF) as f32 / 255.0) * 0.40) * tint
            + source_boost[1]
            + collision_boost[1]
            + note_boost,
        (0.34 + ((seed >> 16 & 0xFF) as f32 / 255.0) * 0.38) * tint
            + source_boost[2]
            + collision_boost[2]
            + anchor_bias,
        0.92,
    ];
    for channel in &mut color[..3] {
        *channel = channel.clamp(0.18, 0.95);
    }
    color
}

fn push_voxel_prism_mesh(
    vertices: &mut Vec<VoxelVertex>,
    indices: &mut Vec<u32>,
    min: glam::Vec3,
    max: glam::Vec3,
    base_color: [f32; 4],
) {
    let corners = [
        [min.x, min.y, min.z],
        [max.x, min.y, min.z],
        [max.x, max.y, min.z],
        [min.x, max.y, min.z],
        [min.x, min.y, max.z],
        [max.x, min.y, max.z],
        [max.x, max.y, max.z],
        [min.x, max.y, max.z],
    ];
    let faces = [
        ([0, 1, 2, 3], shade_color(base_color, 0.64)),
        ([4, 5, 6, 7], shade_color(base_color, 1.08)),
        ([0, 1, 5, 4], shade_color(base_color, 0.82)),
        ([1, 2, 6, 5], shade_color(base_color, 0.92)),
        ([2, 3, 7, 6], shade_color(base_color, 0.76)),
        ([3, 0, 4, 7], shade_color(base_color, 0.70)),
    ];

    for (corner_indices, face_color) in faces {
        let base = vertices.len() as u32;
        for corner in corner_indices {
            vertices.push(VoxelVertex {
                position: corners[corner],
                color: face_color,
            });
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

fn shade_color(color: [f32; 4], multiplier: f32) -> [f32; 4] {
    [
        (color[0] * multiplier).clamp(0.0, 1.0),
        (color[1] * multiplier).clamp(0.0, 1.0),
        (color[2] * multiplier).clamp(0.0, 1.0),
        color[3],
    ]
}

/// Convert a loaded .vox model into cube mesh geometry placed inside the given world bounds.
///
/// Each voxel in the model maps to one small axis-aligned cube. The model grid is scaled
/// to fill the world bounding box `[min, max]` so the object occupies its expected footprint
/// and height in the scene.
fn push_vox_model_mesh(
    vertices: &mut Vec<VoxelVertex>,
    indices: &mut Vec<u32>,
    model: &engine_assets::vox::VoxModel,
    world_min: glam::Vec3,
    world_max: glam::Vec3,
) {
    if model.voxels.is_empty() {
        return;
    }

    let vw = model.width.max(1) as f32;
    let vh = model.height.max(1) as f32;
    let vd = model.depth.max(1) as f32;
    let world_size = world_max - world_min;
    let scale = glam::Vec3::new(
        world_size.x / vw,
        world_size.y / vh,
        world_size.z / vd,
    );

    for voxel in &model.voxels {
        let color_index = voxel.color_index as usize;
        let palette_color = model
            .palette
            .get(color_index)
            .copied()
            .unwrap_or(engine_assets::vox::VoxColor { r: 180, g: 60, b: 200, a: 255 });
        if palette_color.a == 0 {
            continue;
        }
        let r = palette_color.r as f32 / 255.0;
        let g = palette_color.g as f32 / 255.0;
        let b = palette_color.b as f32 / 255.0;
        let base_color = [r, g, b, 1.0];

        let vmin = world_min
            + glam::Vec3::new(voxel.x as f32, voxel.y as f32, voxel.z as f32) * scale;
        let vmax = vmin + scale;
        push_voxel_prism_mesh(vertices, indices, vmin, vmax, base_color);
    }
}

fn build_terrain_resolve_catalog(
    registry: &ContentRegistry,
    tileset: &TilesetDef,
    known_tile_ids: &HashSet<String>,
) -> Option<TerrainResolveCatalog> {
    if registry.terrain_types.is_empty() || registry.biome_packs.is_empty() {
        return None;
    }

    let biome = registry
        .biome_packs
        .get("coastal_grassland")
        .or_else(|| registry.biome_packs.values().next())?;
    if biome.atlas_id != tileset.id {
        log::warn!(
            "terrain biome pack '{}' targets atlas '{}' but active tileset is '{}'",
            biome.id,
            biome.atlas_id,
            tileset.id
        );
    }

    let mut catalog = TerrainResolveCatalog {
        seed: 0x7374_6172_6c69_6768,
        ..TerrainResolveCatalog::default()
    };

    for terrain in registry.terrain_types.values() {
        catalog.terrain_flags.insert(
            terrain.id.clone(),
            TerrainFlags {
                walkable: terrain.walkable,
                blocks_movement: terrain.blocks_movement,
                water: terrain.water,
            },
        );
    }

    for set in &biome.terrain_variant_sets {
        let fallback_tile_id = choose_known_tile(
            known_tile_ids,
            &set.fallback_tile_id,
            registry
                .terrain_types
                .get(&set.terrain_id)
                .map(|terrain| terrain.fallback_tile_id.as_str()),
        );
        let variants = set
            .variants
            .iter()
            .filter(|variant| known_tile_ids.contains(&variant.tile_id))
            .map(|variant| TerrainVariantChoice {
                tile_id: variant.tile_id.clone(),
                weight: variant.weight.max(1),
            })
            .collect::<Vec<_>>();

        catalog.variant_sets.insert(
            set.terrain_id.clone(),
            TerrainVariantSet {
                terrain_id: set.terrain_id.clone(),
                fallback_tile_id,
                variants,
            },
        );
    }

    let active_transition_set_ids = registry
        .terrain_rulesets
        .get(&biome.ruleset)
        .map(|ruleset| ruleset.active_transition_sets.clone())
        .unwrap_or_else(|| registry.transition_sets.keys().cloned().collect());

    for set_id in active_transition_set_ids {
        let Some(set) = registry.transition_sets.get(&set_id) else {
            continue;
        };
        for rule in &set.transitions {
            let fallback_tile_id = choose_known_tile(known_tile_ids, &rule.fallback_tile_id, None);
            let tiles_by_mask = rule
                .tiles
                .iter()
                .filter(|tile| known_tile_ids.contains(&tile.tile_id))
                .map(|tile| (tile.mask, tile.tile_id.clone()))
                .collect::<HashMap<_, _>>();

            catalog.transition_rules.push(TerrainTransitionRule {
                id: rule.id.clone(),
                from: rule.from.clone(),
                to: rule.to.clone(),
                render_layer: rule.render_layer,
                fallback_tile_id,
                tiles_by_mask,
            });
        }
    }

    Some(catalog)
}

fn choose_known_tile(
    known_tile_ids: &HashSet<String>,
    preferred: &str,
    fallback: Option<&str>,
) -> String {
    if known_tile_ids.contains(preferred) {
        return preferred.to_string();
    }
    if let Some(fallback) = fallback {
        if known_tile_ids.contains(fallback) {
            return fallback.to_string();
        }
    }
    "grass_bright".to_string()
}

fn is_contract_semantic_layer(layer: &TileLayerDef, registry: &ContentRegistry) -> bool {
    if registry.terrain_types.is_empty() {
        return false;
    }

    let id = layer.id.to_ascii_lowercase();
    if id.contains("semantic") || id.contains("terrain_contract") || id == "terrain" {
        return true;
    }

    !layer.legend.is_empty()
        && layer
            .legend
            .iter()
            .all(|entry| registry.terrain_types.contains_key(&entry.tile_id))
}

fn push_contract_semantic_layer_tiles(
    tiles: &mut Vec<TileInstance>,
    layer: &TileLayerDef,
    map_width: u32,
    map_height: u32,
    registry: &ContentRegistry,
    catalog: &TerrainResolveCatalog,
    atlas_lookup: &HashMap<&str, (u32, u32)>,
    stats: &mut TileBuildStats,
    map_id: &str,
) -> anyhow::Result<()> {
    let grid = semantic_grid_from_layer(layer, map_width, map_height, registry, map_id)?;
    let resolved = AutotileResolver::resolve(&grid, catalog);

    for tile in resolved.tiles {
        if let Some((atlas_x, atlas_y)) = atlas_lookup.get(tile.tile_id.as_str()).copied() {
            tiles.push(TileInstance {
                x: tile.x,
                y: tile.y,
                atlas_x,
                atlas_y,
            });
            match tile.kind {
                ResolvedTileKind::Base => stats.base_cells += 1,
                ResolvedTileKind::Transition => stats.transition_overlays += 1,
            }
        } else {
            stats.missing_refs += 1;
            log::warn!(
                "map '{}' semantic layer '{}' resolved missing tile id '{}' for terrain '{}' mask {}",
                map_id,
                layer.id,
                tile.tile_id,
                tile.source_terrain_id,
                tile.mask
            );
        }
    }

    Ok(())
}

fn semantic_grid_from_layer(
    layer: &TileLayerDef,
    map_width: u32,
    map_height: u32,
    registry: &ContentRegistry,
    map_id: &str,
) -> anyhow::Result<SemanticTerrainGrid> {
    let legend: HashMap<char, &str> = layer
        .legend
        .iter()
        .filter_map(|entry| {
            entry
                .symbol
                .chars()
                .next()
                .map(|symbol| (symbol, entry.tile_id.as_str()))
        })
        .collect();

    let mut cells = Vec::with_capacity((map_width as usize).saturating_mul(map_height as usize));
    for y in 0..map_height as usize {
        let row = layer.rows.get(y).ok_or_else(|| {
            anyhow::anyhow!("map '{}' layer '{}' missing row {}", map_id, layer.id, y)
        })?;
        for (x, symbol) in row.chars().enumerate().take(map_width as usize) {
            let terrain_id = legend.get(&symbol).ok_or_else(|| {
                anyhow::anyhow!(
                    "map '{}' layer '{}' uses unmapped semantic symbol '{}' at {},{}",
                    map_id,
                    layer.id,
                    symbol,
                    x,
                    y
                )
            })?;
            anyhow::ensure!(
                registry.terrain_types.contains_key(*terrain_id),
                "map '{}' layer '{}' symbol '{}' resolves to unknown terrain id '{}'",
                map_id,
                layer.id,
                symbol,
                terrain_id
            );
            cells.push(SemanticTerrainCell::new(*terrain_id));
        }
    }

    SemanticTerrainGrid::new(map_id.to_string(), map_width, map_height, cells)
}

fn layer_to_tile_grid(
    layer: &TileLayerDef,
    map_width: u32,
    map_height: u32,
) -> Vec<Vec<Option<String>>> {
    let legend: HashMap<char, &str> = layer
        .legend
        .iter()
        .filter_map(|entry| {
            entry
                .symbol
                .chars()
                .next()
                .map(|symbol| (symbol, entry.tile_id.as_str()))
        })
        .collect();

    let mut tile_grid = vec![vec![None::<String>; map_width as usize]; map_height as usize];

    for (y, row) in layer.rows.iter().enumerate().take(map_height as usize) {
        for (x, symbol) in row.chars().enumerate().take(map_width as usize) {
            if let Some(tile_id) = legend.get(&symbol) {
                tile_grid[y][x] = Some((*tile_id).to_string());
            }
        }
    }

    tile_grid
}

fn is_semantic_terrain_layer(layer: &TileLayerDef, grid: &[Vec<Option<String>>]) -> bool {
    let id = layer.id.to_ascii_lowercase();
    if id.contains("decor") || id.contains("prop") || id.contains("object") {
        return false;
    }
    if id.contains("ground")
        || id.contains("terrain")
        || id.contains("soil")
        || id.contains("water")
    {
        return true;
    }

    let mut filled = 0usize;
    let mut terrain = 0usize;
    for tile_id in grid.iter().flatten().filter_map(|entry| entry.as_deref()) {
        filled += 1;
        if TerrainKind::from_tile_id(tile_id).is_some() {
            terrain += 1;
        }
    }

    filled > 0 && terrain * 100 / filled >= 80
}

fn push_terrain_layer_tiles(
    tiles: &mut Vec<TileInstance>,
    grid: &[Vec<Option<String>>],
    atlas_lookup: &HashMap<&str, (u32, u32)>,
    stats: &mut TileBuildStats,
    map_id: &str,
    layer_id: &str,
) {
    let height = grid.len();
    let width = grid.first().map(|row| row.len()).unwrap_or(0);

    // Phase 23 visual-safe terrain pass.
    // Resolve authored terrain IDs only through base_tileset.ron and avoid runtime
    // substitution until a complete directional transition atlas contract exists.
    for y in 0..height {
        for x in 0..width {
            let Some(tile_id) = grid[y][x].as_deref() else {
                continue;
            };

            // Phase 23 visual-safe mode: draw the authored tile id directly.
            // The previous legacy transition substitution was technically atlas-safe,
            // but it still produced visually wrong beaches/cliffs because the Phase 17
            // atlas did not contain a complete directional terrain contract. Keep the
            // full semantic resolver path available for future contract layers, but do
            // not mutate authored starter_farm ground symbols at render time.
            let resolved_tile_id = tile_id.to_string();

            if let Some((atlas_x, atlas_y)) = atlas_lookup.get(resolved_tile_id.as_str()).copied() {
                tiles.push(TileInstance {
                    x: x as u32,
                    y: y as u32,
                    atlas_x,
                    atlas_y,
                });
                if TerrainKind::from_tile_id(tile_id).is_some() {
                    stats.base_cells += 1;
                } else {
                    stats.direct_cells += 1;
                }
            } else {
                stats.missing_refs += 1;
                log::warn!(
                    "map '{}' layer '{}' references missing tile id '{}' resolved from '{}'",
                    map_id,
                    layer_id,
                    resolved_tile_id,
                    tile_id
                );
            }
        }
    }
}

fn push_direct_layer_tiles(
    tiles: &mut Vec<TileInstance>,
    grid: &[Vec<Option<String>>],
    atlas_lookup: &HashMap<&str, (u32, u32)>,
    stats: &mut TileBuildStats,
    map_id: &str,
    layer_id: &str,
) {
    for (y, row) in grid.iter().enumerate() {
        for (x, tile_id) in row
            .iter()
            .enumerate()
            .filter_map(|(x, entry)| entry.as_deref().map(|tile_id| (x, tile_id)))
        {
            if let Some((atlas_x, atlas_y)) = atlas_lookup.get(tile_id).copied() {
                tiles.push(TileInstance {
                    x: x as u32,
                    y: y as u32,
                    atlas_x,
                    atlas_y,
                });
                stats.direct_cells += 1;
            } else {
                stats.missing_refs += 1;
                log::warn!(
                    "map '{}' layer '{}' references missing tile id '{}'",
                    map_id,
                    layer_id,
                    tile_id
                );
            }
        }
    }
}

fn max_layer_width(layers: &MapLayersDef) -> u32 {
    layers
        .layers
        .iter()
        .flat_map(|layer| layer.rows.iter())
        .map(|row| row.chars().count() as u32)
        .max()
        .unwrap_or(0)
}

fn max_layer_height(layers: &MapLayersDef) -> u32 {
    layers
        .layers
        .iter()
        .map(|layer| layer.rows.len() as u32)
        .max()
        .unwrap_or(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerrainKind {
    GrassBright,
    GrassDark,
    Dirt,
    PathSand,
    TilledDry,
    TilledWatered,
    Stone,
    Wood,
    Sand,
    WaterShallow,
    WaterDeep,
    Cliff,
}

impl TerrainKind {
    fn from_tile_id(tile_id: &str) -> Option<Self> {
        match tile_id {
            "grass_bright" | "grass_flowers" => Some(Self::GrassBright),
            "grass_dark" => Some(Self::GrassDark),
            "dirt" => Some(Self::Dirt),
            "path_sand" => Some(Self::PathSand),
            "tilled_dry" => Some(Self::TilledDry),
            "tilled_watered" => Some(Self::TilledWatered),
            "stone_floor" => Some(Self::Stone),
            "wood_floor" => Some(Self::Wood),
            "sand" => Some(Self::Sand),
            "water_shallow" => Some(Self::WaterShallow),
            "water_deep" => Some(Self::WaterDeep),
            "cliff" => Some(Self::Cliff),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerFacing {
    Down,
    Left,
    Right,
    Up,
}

#[derive(Debug, Clone)]
struct RuntimeWorldState {
    player_x: f32,
    player_y: f32,
    world_width: f32,
    world_height: f32,
    player_width: f32,
    player_height: f32,
    facing: PlayerFacing,
    moving: bool,
    animation_time: f32,
    game_minutes: f32,
    ambient_light: f32,
}

impl RuntimeWorldState {
    fn new(registry: &ContentRegistry, map_id: &str, tile_map: Option<&TileMapRenderData>) -> Self {
        let tile_width = tile_map
            .map(|map| map.tile_width.max(1) as f32)
            .unwrap_or(32.0);
        let tile_height = tile_map
            .map(|map| map.tile_height.max(1) as f32)
            .unwrap_or(32.0);
        let world_width = tile_map
            .map(|map| (map.map_width.max(1) * map.tile_width.max(1)) as f32)
            .unwrap_or(1280.0);
        let world_height = tile_map
            .map(|map| (map.map_height.max(1) * map.tile_height.max(1)) as f32)
            .unwrap_or(896.0);

        let map = registry.maps.get(map_id);
        let spawn = map.and_then(|map| {
            map.spawns
                .iter()
                .find(|spawn| spawn.kind == "player" || spawn.id == "player_start")
        });

        let spawn_x = spawn.map(|spawn| spawn.x.max(0) as f32).unwrap_or(2.0);
        let spawn_y = spawn.map(|spawn| spawn.y.max(0) as f32).unwrap_or(2.0);
        let player_width = 32.0;
        let player_height = 48.0;
        let ambient_light = map
            .map(|map| map.metadata.ambient_light)
            .unwrap_or(0.88)
            .clamp(0.2, 1.2);

        Self {
            player_x: spawn_x * tile_width,
            player_y: (spawn_y * tile_height - (player_height - tile_height)).max(0.0),
            world_width,
            world_height,
            player_width,
            player_height,
            facing: PlayerFacing::Down,
            moving: false,
            animation_time: 0.0,
            game_minutes: 9.0 * 60.0,
            ambient_light,
        }
    }

    fn update(&mut self, input: InputSnapshot, delta_seconds: f32) {
        let delta_seconds = delta_seconds.clamp(0.0, 0.1);
        let mut dx = 0.0_f32;
        let mut dy = 0.0_f32;

        if input.move_left {
            dx -= 1.0;
        }
        if input.move_right {
            dx += 1.0;
        }
        if input.move_up {
            dy -= 1.0;
        }
        if input.move_down {
            dy += 1.0;
        }

        let len = (dx * dx + dy * dy).sqrt();
        self.moving = len > 0.001;
        if self.moving {
            dx /= len;
            dy /= len;
            const PLAYER_SPEED_PX_PER_SECOND: f32 = 96.0;
            self.player_x += dx * PLAYER_SPEED_PX_PER_SECOND * delta_seconds;
            self.player_y += dy * PLAYER_SPEED_PX_PER_SECOND * delta_seconds;
            self.animation_time += delta_seconds;

            if dy.abs() >= dx.abs() {
                self.facing = if dy >= 0.0 {
                    PlayerFacing::Down
                } else {
                    PlayerFacing::Up
                };
            } else {
                self.facing = if dx >= 0.0 {
                    PlayerFacing::Right
                } else {
                    PlayerFacing::Left
                };
            }
        } else {
            self.animation_time = 0.0;
        }

        self.player_x = self
            .player_x
            .clamp(0.0, (self.world_width - self.player_width).max(0.0));
        self.player_y = self
            .player_y
            .clamp(0.0, (self.world_height - self.player_height).max(0.0));

        // Target contract: 1 in-game day = 3 real-world hours.
        const GAME_MINUTES_PER_REAL_SECOND: f32 = 1440.0 / (3.0 * 60.0 * 60.0);
        self.game_minutes =
            (self.game_minutes + delta_seconds * GAME_MINUTES_PER_REAL_SECOND) % 1440.0;
    }

    fn player_sprites(&self) -> [SpriteInstance; 1] {
        let row = match self.facing {
            PlayerFacing::Down => 0,
            PlayerFacing::Left => 1,
            PlayerFacing::Right => 2,
            PlayerFacing::Up => 3,
        };
        let frame = if self.moving {
            1 + ((self.animation_time * 8.0) as u32 % 3)
        } else {
            0
        };

        [SpriteInstance {
            x: self.player_x.round(),
            y: self.player_y.round(),
            w: self.player_width,
            h: self.player_height,
            atlas_x: frame,
            atlas_y: row,
        }]
    }

    fn lighting_state(&self) -> WorldLighting {
        let hour = self.game_minutes / 60.0;
        let sun = if (6.0..=18.0).contains(&hour) {
            let t = (hour - 6.0) / 12.0;
            (std::f32::consts::PI * t).sin().max(0.0)
        } else {
            0.0
        };
        let ambient = (0.36 + sun * 0.64) * self.ambient_light;
        let night_alpha = ((1.0 - ambient).clamp(0.0, 1.0) * 0.58).clamp(0.04, 0.46);
        let warm_dawn_dusk = if (5.0..7.5).contains(&hour) || (17.0..20.0).contains(&hour) {
            0.025
        } else {
            0.0
        };

        WorldLighting::new(
            0.03 + warm_dawn_dusk,
            0.055 + warm_dawn_dusk * 0.5,
            0.145,
            night_alpha,
        )
    }
}

struct BootstrapApp {
    project_root: PathBuf,
    registry: ContentRegistry,
    window_id: Option<WindowId>,
    renderer: Option<RenderBootstrap>,
    launch_mode: LaunchMode,
    active_map_id: String,
    tile_map: Option<TileMapRenderData>,
    player_sprite_data: Option<SpriteRenderData>,
    prop_sprite_data: Option<SpriteRenderData>,
    static_prop_sprites: Vec<SpriteInstance>,
    voxel_scene: Option<VoxelSceneRenderData>,
    game_runtime: RuntimeWorldState,
    timer: FrameTimer,
    input: InputSnapshot,
    asset_watch: AssetWatchState,
    editor_ui: EditorShellRenderState,
    cursor_ndc: Option<(f32, f32)>,
    fatal_error: Option<String>,
}

impl BootstrapApp {
    fn new(
        project_root: PathBuf,
        registry: ContentRegistry,
        launch_mode: LaunchMode,
        active_map_id: String,
        tile_map: Option<TileMapRenderData>,
        player_sprite_data: Option<SpriteRenderData>,
        prop_sprite_data: Option<SpriteRenderData>,
        static_prop_sprites: Vec<SpriteInstance>,
        voxel_scene: Option<VoxelSceneRenderData>,
        game_runtime: RuntimeWorldState,
    ) -> Self {
        let asset_watch = AssetWatchState::new(&project_root);
        let mut editor_ui = EditorShellRenderState::default();
        let initial_role = load_tile_role_state(&project_root, &editor_ui.selected_tile_name);
        editor_ui.selected_role = initial_role.role;
        editor_ui.selected_collision = initial_role.collision;

        Self {
            project_root,
            registry,
            window_id: None,
            renderer: None,
            launch_mode,
            active_map_id,
            tile_map,
            player_sprite_data,
            prop_sprite_data,
            static_prop_sprites,
            voxel_scene,
            game_runtime,
            timer: FrameTimer::new(),
            input: InputSnapshot::default(),
            asset_watch,
            editor_ui,
            cursor_ndc: None,
            fatal_error: None,
        }
    }

    fn set_fatal_error(&mut self, context: &str, error: impl std::fmt::Display) {
        let message = format!("{}: {}", context, error);
        log::error!("{}", message);
        self.fatal_error = Some(message);
    }

    fn take_fatal_error(&mut self) -> Option<String> {
        self.fatal_error.take()
    }

    fn reload_live_preview_assets(&mut self) -> anyhow::Result<()> {
        let registry = game_data::load_registry(&self.project_root)
            .context("failed to reload content registry")?;
        let tile_map =
            build_tile_map_render_data(&self.project_root, &registry, &self.active_map_id)
                .with_context(|| {
                    format!(
                        "failed to rebuild render data for map '{}'",
                        self.active_map_id
                    )
                })?;

        if let Some(renderer) = &mut self.renderer {
            renderer.replace_tile_map(tile_map.clone())?;
        }

        self.registry = registry;
        self.static_prop_sprites =
            build_static_prop_instances(&self.registry, &self.active_map_id, tile_map.as_ref());
        self.voxel_scene = build_voxel_scene_render_data(
            &self.project_root,
            &self.active_map_id,
            tile_map.as_ref(),
        )
        .with_context(|| {
            format!(
                "failed to rebuild voxel scene render data for map '{}'",
                self.active_map_id
            )
        })?;
        self.tile_map = tile_map;
        self.asset_watch.reset();

        log::info!(
            "live-preview reload complete: launch={} map={} content=[{}] tilemap={}",
            self.launch_mode.label(),
            self.active_map_id,
            self.registry.summary(),
            self.tile_map
                .as_ref()
                .map(|map| map.tiles.len())
                .unwrap_or(0)
        );

        Ok(())
    }

    fn maybe_hot_reload_assets(&mut self) {
        if !self.asset_watch.should_check() || !self.asset_watch.has_changed() {
            return;
        }

        log::info!("live-preview asset change detected; rebuilding renderer data");
        if let Err(error) = self.reload_live_preview_assets() {
            log::warn!("live-preview reload skipped after error: {error:#}");
            self.asset_watch.reset();
        }
    }

    fn handle_editor_shortcut(&mut self, event: &winit::event::KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }

        match event.physical_key {
            PhysicalKey::Code(KeyCode::F1) => {
                self.editor_ui.left_dock_open = !self.editor_ui.left_dock_open;
                self.editor_ui.status_message = format!(
                    "Assets dock {}",
                    if self.editor_ui.left_dock_open {
                        "opened"
                    } else {
                        "collapsed"
                    }
                );
                self.sync_editor_ui();
            }
            PhysicalKey::Code(KeyCode::F2) => {
                self.editor_ui.right_dock_open = !self.editor_ui.right_dock_open;
                self.editor_ui.status_message = format!(
                    "Inspector dock {}",
                    if self.editor_ui.right_dock_open {
                        "opened"
                    } else {
                        "collapsed"
                    }
                );
                self.sync_editor_ui();
            }
            PhysicalKey::Code(KeyCode::F3) => {
                self.editor_ui.bottom_dock_open = !self.editor_ui.bottom_dock_open;
                self.editor_ui.status_message = format!(
                    "Log dock {}",
                    if self.editor_ui.bottom_dock_open {
                        "opened"
                    } else {
                        "collapsed"
                    }
                );
                self.sync_editor_ui();
            }
            PhysicalKey::Code(KeyCode::F5) => {
                match self.reload_live_preview_assets() {
                    Ok(()) => {
                        self.editor_ui.status_message = "Manual F5 reload complete.".to_string()
                    }
                    Err(error) => {
                        self.editor_ui.status_message =
                            "Manual F5 reload failed. See logs/latest.log.".to_string();
                        log::warn!("manual F5 live-preview reload failed: {error:#}");
                    }
                }
                self.sync_editor_ui();
            }
            PhysicalKey::Code(KeyCode::KeyV) => self.set_active_editor_tool(0),
            PhysicalKey::Code(KeyCode::Space) => self.set_active_editor_tool(1),
            PhysicalKey::Code(KeyCode::KeyB) => self.set_active_editor_tool(2),
            PhysicalKey::Code(KeyCode::KeyE) => self.set_active_editor_tool(3),
            PhysicalKey::Code(KeyCode::KeyG) => self.set_active_editor_tool(4),
            PhysicalKey::Code(KeyCode::KeyI) => self.set_active_editor_tool(5),
            PhysicalKey::Code(KeyCode::KeyT) => self.set_active_editor_tool(6),
            PhysicalKey::Code(KeyCode::KeyC) => self.set_active_editor_tool(7),
            PhysicalKey::Code(KeyCode::KeyA) => self.set_active_editor_tool(8),
            PhysicalKey::Code(KeyCode::KeyP) => self.set_active_editor_tool(9),
            _ => {}
        }
    }

    fn set_active_editor_tool(&mut self, index: usize) {
        self.editor_ui.active_tool = index.min(9);
        self.editor_ui.status_message = format!(
            "Active tool: {}",
            editor_tool_name(self.editor_ui.active_tool)
        );
        self.editor_ui.hover_hint = editor_tool_hint(self.editor_ui.active_tool).to_string();
        self.sync_editor_ui();
    }

    fn select_asset(&mut self, index: usize, label: &str, status: &str) {
        self.editor_ui.selected_asset_index = index;
        self.editor_ui.status_message = status.to_string();
        self.editor_ui.hover_hint =
            format!("Selected asset: {label}. Use Tile Picker or Inspector actions.");
        self.sync_editor_ui();
    }

    fn update_selected_tile_metadata(&mut self) {
        let state = load_tile_role_state(&self.project_root, &self.editor_ui.selected_tile_name);
        self.editor_ui.selected_role = state.role;
        self.editor_ui.selected_collision = state.collision;
    }

    fn select_tile_by_atlas_cell(&mut self, atlas_cell: (u32, u32), status_source: &str) {
        self.editor_ui.selected_cell = atlas_cell;
        self.editor_ui.selected_tile_name = self
            .tile_id_for_atlas_cell(atlas_cell)
            .unwrap_or_else(|| format!("atlas_{},{}", atlas_cell.0, atlas_cell.1));
        self.update_selected_tile_metadata();
        self.editor_ui.status_message = format!(
            "{} selected {} at atlas cell {},{}",
            status_source, self.editor_ui.selected_tile_name, atlas_cell.0, atlas_cell.1
        );
        self.editor_ui.hover_hint =
            "Inspector actions can cycle role/collision and save metadata.".to_string();
        self.sync_editor_ui();
    }

    fn select_next_named_tile(&mut self, step: isize) {
        let Some(tileset) = self.active_tileset() else {
            self.editor_ui.status_message =
                "No active tileset available for tile cycling.".to_string();
            self.sync_editor_ui();
            return;
        };

        if tileset.named_tiles.is_empty() {
            self.editor_ui.status_message = "Active tileset has no named tiles.".to_string();
            self.sync_editor_ui();
            return;
        }

        let current_index = tileset
            .named_tiles
            .iter()
            .position(|tile| tile.id == self.editor_ui.selected_tile_name)
            .unwrap_or(0) as isize;
        let len = tileset.named_tiles.len() as isize;
        let next_index = (current_index + step).rem_euclid(len) as usize;
        let tile = &tileset.named_tiles[next_index];
        let id = tile.id.clone();
        let cell = (tile.x, tile.y);

        self.editor_ui.selected_tile_name = id;
        self.editor_ui.selected_cell = cell;
        self.update_selected_tile_metadata();
        self.editor_ui.status_message = format!(
            "Selected next atlas tile: {} at {},{}",
            self.editor_ui.selected_tile_name, cell.0, cell.1
        );
        self.sync_editor_ui();
    }

    fn select_viewport_tile(&mut self, ndc: (f32, f32)) -> bool {
        let Some((map_x, map_y)) = self.ndc_to_map_tile(ndc) else {
            return false;
        };
        let Some(tile_map) = self.tile_map.as_ref() else {
            return false;
        };
        let Some(tile) = tile_map
            .tiles
            .iter()
            .rev()
            .find(|tile| tile.x == map_x && tile.y == map_y)
            .copied()
        else {
            self.editor_ui.status_message =
                format!("No rendered tile at map cell {},{}", map_x, map_y);
            self.sync_editor_ui();
            return true;
        };

        self.select_tile_by_atlas_cell((tile.atlas_x, tile.atlas_y), "Viewport");
        self.editor_ui.status_message = format!(
            "Viewport tile {},{} -> {} atlas {},{}",
            map_x, map_y, self.editor_ui.selected_tile_name, tile.atlas_x, tile.atlas_y
        );
        self.sync_editor_ui();
        true
    }

    fn ndc_to_map_tile(&self, ndc: (f32, f32)) -> Option<(u32, u32)> {
        let tile_map = self.tile_map.as_ref()?;
        let renderer = self.renderer.as_ref()?;
        let size = renderer.window().inner_size();
        let viewport_width = size.width.max(1) as f32;
        let viewport_height = size.height.max(1) as f32;

        let world_width = (tile_map.map_width.max(1) * tile_map.tile_width.max(1)) as f32;
        let world_height = (tile_map.map_height.max(1) * tile_map.tile_height.max(1)) as f32;
        let scale = ((viewport_width * 0.92) / world_width.max(1.0))
            .min((viewport_height * 0.92) / world_height.max(1.0))
            .max(0.0001);

        let screen_x = ndc.0 * viewport_width * 0.5;
        let screen_y = -ndc.1 * viewport_height * 0.5;
        let world_x = screen_x / scale + world_width * 0.5;
        let world_y = screen_y / scale + world_height * 0.5;

        if world_x < 0.0 || world_y < 0.0 || world_x >= world_width || world_y >= world_height {
            return None;
        }

        Some((
            (world_x / tile_map.tile_width.max(1) as f32).floor() as u32,
            (world_y / tile_map.tile_height.max(1) as f32).floor() as u32,
        ))
    }

    fn active_tileset(&self) -> Option<&TilesetDef> {
        let map_bundle = self.registry.maps.get(&self.active_map_id)?;
        self.registry.tilesets.get(&map_bundle.metadata.tileset)
    }

    fn tile_id_for_atlas_cell(&self, atlas_cell: (u32, u32)) -> Option<String> {
        let tileset = self.active_tileset()?;
        tileset
            .named_tiles
            .iter()
            .find(|tile| tile.x == atlas_cell.0 && tile.y == atlas_cell.1)
            .map(|tile| tile.id.clone())
    }

    fn cycle_selected_role(&mut self) {
        let mut state =
            load_tile_role_state(&self.project_root, &self.editor_ui.selected_tile_name);
        let next_role = next_cycle_value(&state.role, EDITOR_ROLE_CYCLE);
        state = TileRoleState::from_role(&next_role);
        self.editor_ui.selected_role = state.role.clone();
        self.editor_ui.selected_collision = state.collision.clone();

        match save_tile_role_state(
            &self.project_root,
            &self.editor_ui.selected_tile_name,
            &state,
        ) {
            Ok(()) => {
                self.editor_ui.status_message = format!(
                    "Saved role '{}' for {}.",
                    state.role, self.editor_ui.selected_tile_name
                );
                self.reload_after_metadata_save();
            }
            Err(error) => {
                self.editor_ui.status_message =
                    "Role save failed. See logs/latest.log.".to_string();
                log::warn!("native role metadata save failed: {error:#}");
            }
        }
        self.sync_editor_ui();
    }

    fn cycle_selected_collision(&mut self) {
        let current = load_tile_role_state(&self.project_root, &self.editor_ui.selected_tile_name);
        let next_collision = next_cycle_value(&current.collision, EDITOR_COLLISION_CYCLE);
        let mut state = TileRoleState::from_collision(&current.role, &next_collision);
        if current.role == "crop_soil" {
            state.crop_soil = true;
            state.interactable = true;
        }
        if current.role == "door" {
            state.door = true;
            state.interactable = true;
        }
        self.editor_ui.selected_role = state.role.clone();
        self.editor_ui.selected_collision = state.collision.clone();

        match save_tile_role_state(
            &self.project_root,
            &self.editor_ui.selected_tile_name,
            &state,
        ) {
            Ok(()) => {
                self.editor_ui.status_message = format!(
                    "Saved collision '{}' for {}.",
                    state.collision, self.editor_ui.selected_tile_name
                );
                self.reload_after_metadata_save();
            }
            Err(error) => {
                self.editor_ui.status_message =
                    "Collision save failed. See logs/latest.log.".to_string();
                log::warn!("native collision metadata save failed: {error:#}");
            }
        }
        self.sync_editor_ui();
    }

    fn write_asset_studio_selection_manifest(&mut self) {
        let path = self
            .project_root
            .join("artifacts")
            .join("native_asset_studio_selection.ron");
        let body = format!(
            "(\n    selected_asset_index: {},\n    selected_tile: \"{}\",\n    selected_cell: ({}, {}),\n    role: \"{}\",\n    collision: \"{}\",\n    note: \"Phase 31 native Asset Studio selection/export checkpoint\",\n)\n",
            self.editor_ui.selected_asset_index,
            self.editor_ui.selected_tile_name,
            self.editor_ui.selected_cell.0,
            self.editor_ui.selected_cell.1,
            self.editor_ui.selected_role,
            self.editor_ui.selected_collision,
        );

        match path
            .parent()
            .map(fs::create_dir_all)
            .transpose()
            .and_then(|_| fs::write(&path, body))
        {
            Ok(()) => {
                self.editor_ui.status_message =
                    format!("Wrote native Asset Studio checkpoint: {}", path.display());
            }
            Err(error) => {
                self.editor_ui.status_message =
                    "Selection checkpoint write failed. See logs/latest.log.".to_string();
                log::warn!("failed to write native asset studio checkpoint: {error:#}");
            }
        }
        self.sync_editor_ui();
    }

    fn reload_after_metadata_save(&mut self) {
        if let Err(error) = self.reload_live_preview_assets() {
            log::warn!("live-preview reload after metadata save failed: {error:#}");
        }
    }

    fn sync_editor_ui(&mut self) {
        if self.launch_mode != LaunchMode::Editor {
            return;
        }
        if let Some(renderer) = &mut self.renderer {
            renderer.set_editor_shell_state(self.editor_ui.clone());
        }
    }

    fn update_editor_hover(&mut self, ndc: (f32, f32)) {
        self.cursor_ndc = Some(ndc);
        if self.launch_mode != LaunchMode::Editor {
            return;
        }

        if let Some(index) = hit_toolbar_tool(ndc) {
            self.editor_ui.hover_hint = editor_tool_hint(index).to_string();
        } else if let Some(index) = hit_left_tab(ndc) {
            self.editor_ui.hover_hint =
                format!("Switch left dock tab to {}.", left_tab_name(index));
        } else if let Some(index) = hit_right_tab(ndc) {
            self.editor_ui.hover_hint =
                format!("Switch inspector tab to {}.", right_tab_name(index));
        } else if let Some(index) = hit_bottom_tab(ndc) {
            self.editor_ui.hover_hint =
                format!("Switch bottom dock tab to {}.", bottom_tab_name(index));
        } else if hit_right_action_role(ndc) {
            self.editor_ui.hover_hint =
                "Role action: click to cycle selected tile role and save sidecar metadata."
                    .to_string();
        } else if hit_right_action_collision(ndc) {
            self.editor_ui.hover_hint =
                "Collision action: click to cycle walk/block/water flags and save metadata."
                    .to_string();
        } else if hit_right_action_seam(ndc) {
            self.editor_ui.hover_hint =
                "Seam action: focus seam diagnostics for the selected tile.".to_string();
        } else if hit_right_action_export(ndc) {
            self.editor_ui.hover_hint =
                "Export action: write an Asset Studio checkpoint manifest.".to_string();
        } else if hit_left_asset_row(ndc, 0.700) {
            self.editor_ui.hover_hint =
                "Terrain Atlas asset: select for tile metadata and atlas workflow.".to_string();
        } else if hit_left_asset_row(ndc, 0.570) {
            self.editor_ui.hover_hint =
                "Player Walk asset: character sprite workflow placeholder.".to_string();
        } else if hit_left_asset_row(ndc, 0.440) {
            self.editor_ui.hover_hint =
                "Ocean Props asset: static prop workflow placeholder.".to_string();
        } else if hit_rect(ndc, -1.0, -1.0, -0.765, 0.920) {
            self.editor_ui.hover_hint =
                "Assets dock: choose project content. F1 collapses it.".to_string();
        } else if hit_rect(ndc, 0.765, -1.0, 1.0, 0.920) {
            self.editor_ui.hover_hint =
                "Inspector dock: selected tile, seams, role, collision.".to_string();
        } else if hit_rect(ndc, -0.765, -1.0, 0.765, -0.835) {
            self.editor_ui.hover_hint =
                "Console dock: validation, hot reload, runtime logs.".to_string();
        } else {
            self.editor_ui.hover_hint =
                "Viewport: click a rendered tile to inspect its atlas cell and metadata."
                    .to_string();
        }
        self.sync_editor_ui();
    }

    fn handle_editor_mouse_click(&mut self) {
        if self.launch_mode != LaunchMode::Editor {
            return;
        }
        let Some(ndc) = self.cursor_ndc else {
            return;
        };

        if let Some(index) = hit_toolbar_tool(ndc) {
            self.set_active_editor_tool(index);
            return;
        }

        if hit_rect(ndc, -0.995, 0.735, -0.905, 0.775) && !self.editor_ui.left_dock_open {
            self.editor_ui.left_dock_open = true;
            self.editor_ui.status_message = "Assets dock opened.".to_string();
            self.sync_editor_ui();
            return;
        }
        if hit_rect(ndc, 0.905, 0.735, 0.995, 0.775) && !self.editor_ui.right_dock_open {
            self.editor_ui.right_dock_open = true;
            self.editor_ui.status_message = "Inspector dock opened.".to_string();
            self.sync_editor_ui();
            return;
        }
        if hit_rect(ndc, -0.050, -0.995, 0.050, -0.950) && !self.editor_ui.bottom_dock_open {
            self.editor_ui.bottom_dock_open = true;
            self.editor_ui.status_message = "Bottom log dock opened.".to_string();
            self.sync_editor_ui();
            return;
        }

        if let Some(index) = hit_left_tab(ndc) {
            self.editor_ui.active_left_tab = index;
            self.editor_ui.status_message = format!("Left dock tab: {}", left_tab_name(index));
            self.sync_editor_ui();
            return;
        }
        if let Some(index) = hit_right_tab(ndc) {
            self.editor_ui.active_right_tab = index;
            self.editor_ui.status_message = format!("Inspector tab: {}", right_tab_name(index));
            self.sync_editor_ui();
            return;
        }
        if let Some(index) = hit_bottom_tab(ndc) {
            self.editor_ui.active_bottom_tab = index;
            self.editor_ui.status_message = format!("Bottom dock tab: {}", bottom_tab_name(index));
            self.sync_editor_ui();
            return;
        }

        if hit_right_action_role(ndc) {
            self.cycle_selected_role();
            return;
        }
        if hit_right_action_collision(ndc) {
            self.cycle_selected_collision();
            return;
        }
        if hit_right_action_seam(ndc) {
            self.editor_ui.active_right_tab = 1;
            self.editor_ui.status_message = format!(
                "Seam diagnostics focused for {}. Native pixel seam painting comes after metadata workflow.",
                self.editor_ui.selected_tile_name
            );
            self.sync_editor_ui();
            return;
        }
        if hit_right_action_export(ndc) {
            self.write_asset_studio_selection_manifest();
            return;
        }
        if hit_preview_tile(ndc) {
            self.editor_ui.active_right_tab = 0;
            self.select_next_named_tile(1);
            return;
        }
        if hit_preview_seam(ndc) {
            self.editor_ui.active_right_tab = 1;
            self.editor_ui.status_message = "Seam preview selected.".to_string();
            self.sync_editor_ui();
            return;
        }
        if hit_preview_atlas(ndc) {
            self.editor_ui.active_right_tab = 2;
            self.select_next_named_tile(1);
            return;
        }

        if hit_left_asset_row(ndc, 0.700) {
            self.select_asset(0, "Terrain Atlas", "Selected Terrain Atlas. Click viewport tiles or inspector preview to choose atlas cells.");
            return;
        } else if hit_left_asset_row(ndc, 0.570) {
            self.select_asset(1, "Player Walk", "Selected Player Walk sprite sheet. Character animation editing is a later native panel.");
            return;
        } else if hit_left_asset_row(ndc, 0.440) {
            self.select_asset(
                2,
                "Ocean Props",
                "Selected Ocean Props sheet. Prop metadata editing is a later native panel.",
            );
            return;
        }

        if self.select_viewport_tile(ndc) {
            return;
        }
    }
}

fn editor_tool_name(index: usize) -> &'static str {
    match index {
        0 => "Select",
        1 => "Pan",
        2 => "Brush",
        3 => "Eraser",
        4 => "Fill",
        5 => "Eyedropper",
        6 => "Tile Picker",
        7 => "Collision Paint",
        8 => "Asset Studio",
        9 => "Playtest",
        _ => "Tool",
    }
}

fn editor_tool_hint(index: usize) -> &'static str {
    match index {
        0 => "Select (V): pick assets, tiles, objects, and panels.",
        1 => "Pan (Space): viewport movement tool placeholder.",
        2 => "Brush (B): terrain paint tool placeholder.",
        3 => "Eraser (E): erase terrain/object placeholder.",
        4 => "Fill (G): bucket fill placeholder.",
        5 => "Eyedropper (I): pick tile/color placeholder.",
        6 => "Tile Picker (T): select atlas cells and roles.",
        7 => "Collision (C): paint walk/block/water flags.",
        8 => "Asset Studio (A): atlas, seam, role, export panels.",
        9 => "Playtest (P): focus viewport testing.",
        _ => "Editor tool",
    }
}

fn hit_toolbar_tool((x, y): (f32, f32)) -> Option<usize> {
    let mut tx = -0.925_f32;
    for index in 0..10 {
        if hit_rect((x, y), tx, 0.932, tx + 0.042, 0.978) {
            return Some(index);
        }
        tx += 0.052;
    }
    None
}

fn hit_left_tab((x, y): (f32, f32)) -> Option<usize> {
    // Matches draw_clean_tab_row(-0.982, 0.808, ["Project", "Textures", "Maps"]).
    if hit_rect((x, y), -0.982, 0.808, -0.890, 0.853) {
        Some(0)
    } else if hit_rect((x, y), -0.882, 0.808, -0.770, 0.853) {
        Some(1)
    } else if hit_rect((x, y), -0.762, 0.808, -0.692, 0.853) {
        Some(2)
    } else {
        None
    }
}

fn hit_right_tab((x, y): (f32, f32)) -> Option<usize> {
    // Matches draw_clean_tab_row(0.782, 0.808, ["Tile", "Seams", "Export"]).
    if hit_rect((x, y), 0.782, 0.808, 0.852, 0.853) {
        Some(0)
    } else if hit_rect((x, y), 0.860, 0.808, 0.940, 0.853) {
        Some(1)
    } else if hit_rect((x, y), 0.948, 0.808, 1.000, 0.853) {
        Some(2)
    } else {
        None
    }
}

fn hit_bottom_tab((x, y): (f32, f32)) -> Option<usize> {
    // Matches draw_clean_tab_row(-0.720, -0.872, ["Console", "Validation", "Hot Reload", "Runtime"]).
    if hit_rect((x, y), -0.720, -0.872, -0.622, -0.827) {
        Some(0)
    } else if hit_rect((x, y), -0.614, -0.872, -0.488, -0.827) {
        Some(1)
    } else if hit_rect((x, y), -0.480, -0.872, -0.340, -0.827) {
        Some(2)
    } else if hit_rect((x, y), -0.332, -0.872, -0.226, -0.827) {
        Some(3)
    } else {
        None
    }
}

fn left_tab_name(index: usize) -> &'static str {
    match index {
        0 => "Project",
        1 => "Textures",
        2 => "Maps",
        _ => "Tab",
    }
}

fn right_tab_name(index: usize) -> &'static str {
    match index {
        0 => "Tile",
        1 => "Seams",
        2 => "Export",
        _ => "Inspector",
    }
}

fn bottom_tab_name(index: usize) -> &'static str {
    match index {
        0 => "Console",
        1 => "Validation",
        2 => "Hot Reload",
        3 => "Runtime",
        _ => "Log",
    }
}

fn hit_left_asset_row((x, y): (f32, f32), row_y: f32) -> bool {
    hit_rect((x, y), -0.972, row_y, -0.762, row_y + 0.092)
}

fn hit_preview_tile(ndc: (f32, f32)) -> bool {
    hit_rect(ndc, 0.798, 0.445, 0.876, 0.590)
}

fn hit_preview_seam(ndc: (f32, f32)) -> bool {
    hit_rect(ndc, 0.890, 0.445, 0.968, 0.590)
}

fn hit_preview_atlas(ndc: (f32, f32)) -> bool {
    hit_rect(ndc, 0.798, 0.245, 0.876, 0.390)
}

fn hit_right_action_role(ndc: (f32, f32)) -> bool {
    hit_rect(ndc, 0.798, 0.005, 0.862, 0.050)
}

fn hit_right_action_collision(ndc: (f32, f32)) -> bool {
    hit_rect(ndc, 0.875, 0.005, 0.939, 0.050)
}

fn hit_right_action_seam(ndc: (f32, f32)) -> bool {
    hit_rect(ndc, 0.798, -0.058, 0.862, -0.013)
}

fn hit_right_action_export(ndc: (f32, f32)) -> bool {
    hit_rect(ndc, 0.875, -0.058, 0.939, -0.013)
}

fn hit_rect((x, y): (f32, f32), x0: f32, y0: f32, x1: f32, y1: f32) -> bool {
    x >= x0 && x <= x1 && y >= y0 && y <= y1
}

impl ApplicationHandler for BootstrapApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }

        let mut config = WindowConfig::default();
        config.title = self.launch_mode.window_title().to_string();

        match create_gl_window(event_loop, &config) {
            Ok(window_bootstrap) => {
                self.window_id = Some(window_bootstrap.window.id());
                match RenderBootstrap::new(
                    window_bootstrap,
                    self.tile_map.clone(),
                    self.player_sprite_data.clone(),
                    self.prop_sprite_data.clone(),
                ) {
                    Ok(mut renderer) => {
                        if self.launch_mode == LaunchMode::Editor {
                            renderer.set_editor_shell_visible(true);
                            renderer.set_editor_shell_state(self.editor_ui.clone());
                        }
                        renderer.window().request_redraw();
                        self.renderer = Some(renderer);
                        log::info!(
                            "{} bootstrap resumed from root {} with {}",
                            self.launch_mode.label(),
                            self.project_root.display(),
                            self.registry.summary()
                        );
                    }
                    Err(error) => {
                        self.set_fatal_error("renderer bootstrap failed", format!("{error:#}"));
                        event_loop.exit();
                    }
                }
            }
            Err(error) => {
                self.set_fatal_error("window creation failed", format!("{error:#}"));
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(active_window_id) = self.window_id else {
            return;
        };
        if active_window_id != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                handle_keyboard_event(&mut self.input, &event);
                if self.launch_mode == LaunchMode::Editor {
                    self.handle_editor_shortcut(&event);
                }
                if self.input.escape_pressed {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                    renderer.window().request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(renderer) = &self.renderer {
                    let size = renderer.window().inner_size();
                    let width = size.width.max(1) as f32;
                    let height = size.height.max(1) as f32;
                    let ndc = (
                        (position.x as f32 / width) * 2.0 - 1.0,
                        1.0 - (position.y as f32 / height) * 2.0,
                    );
                    self.update_editor_hover(ndc);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed && button == MouseButton::Left {
                    self.handle_editor_mouse_click();
                }
            }
            WindowEvent::RedrawRequested => {
                let stats = self.timer.tick();
                self.maybe_hot_reload_assets();

                self.game_runtime
                    .update(self.input, stats.delta.as_secs_f32());
                let player_sprites = self.game_runtime.player_sprites();
                let lighting = self.game_runtime.lighting_state();

                let render_result = self.renderer.as_ref().map(|renderer| {
                    renderer.render_frame(
                        stats.frame_index,
                        self.voxel_scene.as_ref(),
                        &player_sprites,
                        &self.static_prop_sprites,
                        Some(lighting),
                    )
                });

                if let Some(Err(error)) = render_result {
                    self.set_fatal_error("render failure", format!("{error:#}"));
                    event_loop.exit();
                    return;
                }

                if let Some(renderer) = &self.renderer {
                    if stats.frame_index == 1 || stats.frame_index % 300 == 0 {
                        log::info!(
                            "frame={} dt_ms={:.3} uptime_s={:.2}",
                            stats.frame_index,
                            stats.delta.as_secs_f64() * 1000.0,
                            stats.uptime.as_secs_f64(),
                        );
                    }
                    renderer.window().request_redraw();
                }
            }
            _ => {}
        }
    }
}
