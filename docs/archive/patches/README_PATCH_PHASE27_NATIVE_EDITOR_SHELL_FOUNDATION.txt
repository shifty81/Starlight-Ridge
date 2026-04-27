Starlight Ridge Phase 27 — Native Editor Shell Foundation

Apply over the project root and overwrite.

Run cargo check after applying because this patch changes Rust code.

Main goals:
- stop app/editor from silently opening and closing on startup failure
- show runtime failure dialogs and write logs/latest_runtime_failure.log
- add RUN_EDITOR_DIAGNOSTIC.bat
- add the first native Rust editor dock/toolbar shell over the game viewport
- define native Asset Studio / Animation Studio / Character Studio workflow contracts
- add male/female base mannequin assets and metadata

After build:
1. Launch the editor normally.
2. If it still closes immediately, run RUN_EDITOR_DIAGNOSTIC.bat.
3. Upload logs/latest_runtime_failure.log or logs/editor_runtime_failure_latest.log.
