# Patch Phase 45 — Character Studio Real Assets + Offsets

Extract over the project root.

This patch adds:

- Character Studio mannequin PNG sheets.
- Paperdoll overlay PNG sheets.
- Editor-only character asset contracts.
- Foot anchor and tool socket contracts.
- Phase 45 documentation.

Important compile fix included:

- `crates/editor_core/src/atlas_pipeline.rs`
- `crates/editor_core/src/export_pipeline.rs`

These files are required because `editor_core/src/lib.rs` declares those modules.
