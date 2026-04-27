# Getting Started

## Prerequisites

| Requirement | Version | Notes |
|---|---|---|
| Rust | stable | `rustup show` must report `stable` as active |
| Cargo | ships with Rust | — |
| OpenGL | 3.3 core profile | Any modern GPU |
| Linux only | `libGL`, `libasound2-dev`, `libxkbcommon-dev` | `sudo apt install libgl1-mesa-dev libasound2-dev libxkbcommon-dev` |
| Windows | none extra | MSVC or GNU toolchain both work |
| macOS | not tested | OpenGL deprecated on macOS; may need a compatibility layer |

The workspace is pinned to the **stable** channel via `rust-toolchain.toml`. Running any `cargo` command will automatically download the right toolchain via rustup.

---

## Clone and verify

```sh
git clone https://github.com/shifty81/Starlight-Ridge.git
cd Starlight-Ridge

# Check everything compiles (fastest validation, no binary produced)
cargo check
```

A clean `cargo check` with no errors means your environment is correctly set up.

---

## Build and run

### Game (debug)

```sh
cargo run -p app
```

The window opens showing the `starter_farm` map. WASD / arrow keys are captured (player movement is not yet fully wired — see [ROADMAP.md](ROADMAP.md)).

### Game (release / optimised)

```sh
cargo build --release -p app
./target/release/app          # Linux/macOS
target\release\app.exe        # Windows
```

### Native editor

```sh
cargo run -p app --bin editor
```

Opens the same window with the egui editor overlay active.

### Web / LAN editor (read-only)

```sh
cargo run -p web_editor_server
# Output shows your local IP — open http://127.0.0.1:8787/ in a browser
```

To enable saving back to disk:

```sh
STARLIGHT_WEB_ALLOW_WRITE=1 cargo run -p web_editor_server
```

### Windows batch shortcuts

The repo root contains `.bat` / `.ps1` shortcuts for common operations:

| File | What it does |
|---|---|
| `BUILD_MENU.bat` | Interactive build/run menu |
| `RUN_GAME_DEBUG.bat` | `cargo run -p app` |
| `RUN_GAME_RELEASE.bat` | Release game binary |
| `RUN_EDITOR_DEBUG.bat` | Native editor (debug) |
| `RUN_EDITOR_RELEASE.bat` | Native editor (release) |
| `RUN_WEB_EDITOR_LAN.bat` | Web editor (read-only) |
| `RUN_WEB_EDITOR_LAN_WRITE.bat` | Web editor (write mode) |
| `CREATE_DIAGNOSTICS.bat` | Dump runtime logs and system info |

---

## Runtime logs

On startup the app writes logs to `logs/runtime_latest.log` relative to the project root. If the game window closes immediately, check this file first.

```sh
cat logs/runtime_latest.log
```

---

## Common errors

| Error | Fix |
|---|---|
| `assets folder not found` | Run from the repo root, not `target/debug/` |
| `content folder not found` | Same as above |
| `failed to bind web editor server to 0.0.0.0:8787` | Port in use — set `STARLIGHT_WEB_PORT=8788` |
| OpenGL context failure on Linux | Install Mesa: `sudo apt install libgl1-mesa-dev` |
| Texture load failure | Check `assets/textures/` for `terrain_atlas_phase15_contract.png` and `player_walk.png` |

---

## Linting and formatting

```sh
cargo fmt --all          # Format all crates
cargo clippy --all       # Lint (stable channel)
```

---

## Project root discovery

The `app` crate walks upward from the executable location until it finds a directory that contains both `Cargo.toml` and a `content/` folder. This means you can run the binary from `target/debug/`, `target/release/`, or the repo root and it will find assets correctly.
