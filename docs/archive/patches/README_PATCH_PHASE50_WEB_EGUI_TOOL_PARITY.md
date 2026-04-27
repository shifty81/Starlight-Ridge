# Patch Phase 50 — Web / egui Tool Parity

Extract over the project root.

Changed files:

- `tools/web_editor/index.html`
- `tools/web_editor/app.css`
- `tools/web_editor/app.js`
- `content/editor_web/web_editor_parity_phase50.ron`
- `docs/editor/PHASE50_WEB_EGUI_TOOL_PARITY.md`
- `docs/roadmap/CHAT_ROLLUP_PHASE50.md`

Verification:

1. Run the LAN web editor from the build menu.
2. Open PC mode and tablet mode.
3. Test keybinds: `P`, `E`, `F`, `I`, `B`, `C`, `V`, `H`, `J`, `R`, `G`, `[`, `]`.
4. In write-enabled mode, test `Ctrl+S`.

Notes:

This patch changes the browser companion UI/JS and does not require a Rust compile to validate the web behavior. Running `cargo check` should still remain valid because no Rust code is changed.
