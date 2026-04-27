# Starlight Ridge Runtime Diagnostics Pass

This source zip adds runtime diagnostics for the case where the game/editor window opens and immediately closes without a visible error.

## What changed

- `crates/app/src/main.rs` no longer returns errors silently; it writes runtime failures and exits with code `1`.
- `crates/app/src/bin/editor.rs` uses the same explicit failure handling.
- `crates/app/src/lib.rs` now writes startup milestones, runtime failures, renderer/window startup failures, panic details, and event-loop exit reasons.
- `build.sh` now runs game/editor commands with `RUST_LOG=info` and `RUST_BACKTRACE=1` by default.
- `build.sh` option 4 now builds the game executable directly: `cargo build -p app --bin app --release`.
- New menu options can run release executables and create a diagnostic bundle under `artifacts/`.

## Runtime log files

After running the game or editor, check:

```text
logs/runtime_game.log
logs/runtime_editor.log
logs/runtime_latest.log
logs/latest.log
```

If the window closes immediately, upload `logs/latest.log` and `logs/runtime_latest.log`, or use option `16) Create diagnostic bundle` and upload the generated zip from `artifacts/`.

## Current known gap

The release build now compiles successfully, so the remaining issue is runtime startup behavior. This pass is intentionally diagnostic-first; it does not change terrain rendering or auto-tiling yet.
