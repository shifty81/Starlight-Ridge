Starlight Ridge Phase 36 - egui Editor Conversion

Apply over the project root and overwrite.

Changed:
- crates/app/Cargo.toml
- crates/app/src/lib.rs

Added:
- crates/app/src/egui_editor.rs
- docs/PATCH_README_PHASE36_EGUI_EDITOR_CONVERSION.md

Build:
cargo check
cargo build --release --bin editor

Run:
RUN_EDITOR_DIAGNOSTIC.bat
or
target\release\editor.exe

Expected:
editor.exe now launches the egui/eframe editor shell instead of the old custom OpenGL overlay editor UI.

Note:
The previous OpenGL editor path is preserved as app::run_legacy_gl_editor() for renderer debugging only.
