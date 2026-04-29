Starlight Ridge Hotfix 001

Fixes:
1. editor_inspector compile error E0063 by adding InspectorRow.value for the map ID row.
2. cargo run ambiguity by adding default-run = "app" to crates/app/Cargo.toml.

Install:
Extract this zip over your StarLight Ridge project root and overwrite files.

Verify:
cargo check
cargo run -p app
cargo run -p app --bin editor

If cargo run -p app still reports ambiguity on your local Cargo version, use:
cargo run -p app --bin app
