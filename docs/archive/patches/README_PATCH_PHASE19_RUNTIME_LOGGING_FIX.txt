Starlight Ridge Phase 19 Runtime Logging Compile Fix

Drop this zip over the project root and overwrite existing files.

Fixes:
- Adds app::init_runtime_logging(label)
- Adds app::write_runtime_failure(label, &anyhow::Error)
- Creates/uses the project logs folder for latest runtime failure mirrors

Why:
- crates/app/src/main.rs and crates/app/src/bin/editor.rs already call these helpers.
- The helpers were missing from crates/app/src/lib.rs, causing E0425 compile failures.

Notes:
- The same_transition_group_at / TerrainGroup / transition_group / transition_column messages are warnings only.
- They do not block the build.
- Cargo was not available in the container used to generate this patch, so run your root build script after applying.
