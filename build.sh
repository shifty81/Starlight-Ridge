#!/usr/bin/env bash
set -u

# Starlight Ridge root build utility
# Usage:
#   ./build.sh                 # interactive menu
#   build.bat                  # Windows launcher that keeps the window open
#   BUILD_MENU.bat             # Windows menu launcher
#   ./build.sh check           # cargo check
#   ./build.sh build           # cargo build
#   ./build.sh release         # cargo build -p app --bin app --release
#   ./build.sh run             # cargo run -p app --bin app with runtime diagnostics
#   ./build.sh editor          # cargo run -p app --bin editor with runtime diagnostics
#   ./build.sh run-release     # run target/release/app.exe with runtime diagnostics
#   ./build.sh editor-release  # run target/release/editor.exe with runtime diagnostics
#   ./build.sh diagnostics     # package logs and core project metadata for upload
#   ./build.sh all             # cargo check, then cargo build
#   ./build.sh clean           # cargo clean
#   ./build.sh logs            # open/show logs folder
#   ./build.sh sanitize        # remove bad root extraction artifacts
#   ./build.sh package         # create source zip under artifacts/

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="$ROOT_DIR/logs"
ARTIFACT_DIR="$ROOT_DIR/artifacts"
mkdir -p "$LOG_DIR" "$ARTIFACT_DIR"

print_header() {
    clear 2>/dev/null || true
    echo "========================================"
    echo " Starlight Ridge Build Utility"
    echo " Root: $ROOT_DIR"
    echo " Logs: $LOG_DIR"
    echo " Artifacts: $ARTIFACT_DIR"
    echo "========================================"
    echo
}

pause() {
    echo
    read -r -p "Press Enter to continue..." _
}

now_stamp() {
    date +"%Y-%m-%d_%H-%M-%S"
}

safe_name() {
    echo "$1" | tr ' /:' '___' | tr -cd '[:alnum:]_\.-'
}

run_logged() {
    local name="$1"
    shift

    mkdir -p "$LOG_DIR"
    local stamp
    stamp="$(now_stamp)"
    local clean_name
    clean_name="$(safe_name "$name")"
    local log_file="$LOG_DIR/${stamp}_${clean_name}.log"
    local latest_file="$LOG_DIR/latest.log"

    echo "========================================" | tee "$log_file"
    echo " Starlight Ridge Build Utility" | tee -a "$log_file"
    echo " Command: $name" | tee -a "$log_file"
    echo " Started: $stamp" | tee -a "$log_file"
    echo " Root: $ROOT_DIR" | tee -a "$log_file"
    echo "========================================" | tee -a "$log_file"
    echo | tee -a "$log_file"

    (
        cd "$ROOT_DIR" || exit 1
        "$@"
    ) 2>&1 | tee -a "$log_file"

    local code=${PIPESTATUS[0]}
    echo | tee -a "$log_file"
    echo "========================================" | tee -a "$log_file"
    echo " Finished with exit code: $code" | tee -a "$log_file"
    echo " Log saved to: $log_file" | tee -a "$log_file"
    echo "========================================" | tee -a "$log_file"

    cp "$log_file" "$latest_file" 2>/dev/null || true

    if [ "$code" -ne 0 ]; then
        echo
        echo "Command failed. Upload this file for diagnosis:"
        echo "$latest_file"
    else
        echo
        echo "Command completed successfully. Log saved to:"
        echo "$latest_file"
    fi

    return "$code"
}

check_tools() {
    echo "Checking required tools..."
    echo

    if command -v rustc >/dev/null 2>&1; then
        echo "Rust:"
        rustc --version
    else
        echo "MISSING: rustc"
    fi

    echo

    if command -v cargo >/dev/null 2>&1; then
        echo "Cargo:"
        cargo --version
    else
        echo "MISSING: cargo"
    fi

    echo

    if command -v zip >/dev/null 2>&1; then
        echo "Zip: $(command -v zip)"
    elif command -v 7z >/dev/null 2>&1; then
        echo "7-Zip: $(command -v 7z)"
    elif command -v powershell.exe >/dev/null 2>&1; then
        echo "PowerShell Compress-Archive: available"
    else
        echo "WARNING: no zip, 7z, or powershell.exe archiver found"
    fi

    echo

    if [ -f "$ROOT_DIR/Cargo.toml" ]; then
        echo "OK: Cargo.toml found"
    else
        echo "WARNING: Cargo.toml not found. Keep build.sh in the repo root."
    fi
}

cmd_check() {
    run_logged "cargo_check" cargo check
}

cmd_build() {
    run_logged "cargo_build_debug_game" cargo build -p app --bin app
}

cmd_release() {
    run_logged "cargo_build_release_game" cargo build -p app --bin app --release
}

cmd_build_editor() {
    run_logged "cargo_build_debug_editor" cargo build -p app --bin editor
}

cmd_release_editor() {
    run_logged "cargo_build_release_editor" cargo build -p app --bin editor --release
}

cmd_run() {
    run_logged "cargo_run_game" env RUST_LOG="${RUST_LOG:-info}" RUST_BACKTRACE="${RUST_BACKTRACE:-1}" cargo run -p app --bin app
}

cmd_editor() {
    run_logged "cargo_run_editor" env RUST_LOG="${RUST_LOG:-info}" RUST_BACKTRACE="${RUST_BACKTRACE:-1}" cargo run -p app --bin editor
}

cmd_web_editor_lan() {
    run_logged "cargo_run_web_editor_lan" env STARLIGHT_WEB_HOST="${STARLIGHT_WEB_HOST:-0.0.0.0}" STARLIGHT_WEB_PORT="${STARLIGHT_WEB_PORT:-8787}" cargo run -p web_editor_server
}

cmd_web_editor_lan_write() {
    run_logged "cargo_run_web_editor_lan_write" env STARLIGHT_WEB_HOST="${STARLIGHT_WEB_HOST:-0.0.0.0}" STARLIGHT_WEB_PORT="${STARLIGHT_WEB_PORT:-8787}" STARLIGHT_WEB_ALLOW_WRITE=1 cargo run -p web_editor_server
}

cmd_run_release() {
    if [ ! -f "$ROOT_DIR/target/release/app.exe" ] && [ ! -f "$ROOT_DIR/target/release/app" ]; then
        echo "Release game executable not found. Run option 4 first."
        return 1
    fi

    if [ -f "$ROOT_DIR/target/release/app.exe" ]; then
        run_logged "run_release_game_exe" env RUST_LOG="${RUST_LOG:-info}" RUST_BACKTRACE="${RUST_BACKTRACE:-1}" "$ROOT_DIR/target/release/app.exe"
    else
        run_logged "run_release_game_exe" env RUST_LOG="${RUST_LOG:-info}" RUST_BACKTRACE="${RUST_BACKTRACE:-1}" "$ROOT_DIR/target/release/app"
    fi
}

cmd_editor_release() {
    if [ ! -f "$ROOT_DIR/target/release/editor.exe" ] && [ ! -f "$ROOT_DIR/target/release/editor" ]; then
        echo "Release editor executable not found. Run option 14 to build it first."
        return 1
    fi

    if [ -f "$ROOT_DIR/target/release/editor.exe" ]; then
        run_logged "run_release_editor_exe" env RUST_LOG="${RUST_LOG:-info}" RUST_BACKTRACE="${RUST_BACKTRACE:-1}" "$ROOT_DIR/target/release/editor.exe"
    else
        run_logged "run_release_editor_exe" env RUST_LOG="${RUST_LOG:-info}" RUST_BACKTRACE="${RUST_BACKTRACE:-1}" "$ROOT_DIR/target/release/editor"
    fi
}

cmd_all() {
    cmd_check
    local check_code=$?
    if [ "$check_code" -ne 0 ]; then
        return "$check_code"
    fi

    cmd_build
}

cmd_clean() {
    run_logged "cargo_clean" cargo clean
}

cmd_sanitize() {
    run_logged "sanitize_project_root" bash -lc '
        set -u
        echo "Scanning repo root for bad extraction/copy artifacts..."
        found=0
        while IFS= read -r -d "" file; do
            found=1
            echo "Removing: $file"
            rm -f -- "$file"
        done < <(find . -maxdepth 1 -type f \( \
            -name "cratesappsrcbin*.rs*" -o \
            -name "cratesappsrcbineditor.rs*" -o \
            -name "cratesapp*" \
        \) -print0)

        if [ "$found" -eq 0 ]; then
            echo "No bad root artifacts found."
        fi

        mkdir -p logs artifacts saves
        touch logs/.gitkeep artifacts/.gitkeep saves/.gitkeep
        echo "Root sanitize complete."
    '
}

cmd_organize_patch_docs() {
    run_logged "organize_patch_docs" bash -lc '
        set -u
        mkdir -p docs/patches
        shopt -s nullglob
        for file in PATCH_*.md PATCH_*.txt README_PATCH*.md README_PATCH*.txt README_PHASE*.txt MANIFEST_phase*.txt; do
            [ -f "$file" ] || continue
            echo "Moving patch doc: $file -> docs/patches/"
            mv -f "$file" docs/patches/
        done
        echo "Patch documentation organization complete."
    '
}

cmd_consolidate_root_scripts() {
    run_logged "consolidate_root_scripts" bash -lc '
        set -u
        mkdir -p tools/legacy_launchers
        shopt -s nullglob
        for file in *.bat *.cmd *.ps1 *.sh; do
            case "$file" in
                build.bat|BUILD_MENU.bat|BUILD_MENU_GIT_BASH.bat|build.sh) continue ;;
            esac
            case "$file" in
                BUILD_*|CREATE_*|RUN_*|CLEAN_*|ORGANIZE_*)
                    echo "Moving legacy root launcher: $file -> tools/legacy_launchers/"
                    mv -f "$file" tools/legacy_launchers/
                    ;;
            esac
        done
        echo "Root launcher consolidation complete. Preferred entry: BUILD_MENU.bat or build.bat."
    '
}

cmd_self_heal() {
    cmd_sanitize || return $?
    cmd_organize_patch_docs || return $?
    run_logged "self_heal_project" bash -lc '
        set -u
        mkdir -p logs artifacts saves docs/patches tools/legacy_launchers
        for path in Cargo.toml Cargo.lock content assets tools/web_editor crates/app crates/web_editor_server; do
            if [ -e "$path" ]; then echo "OK: $path"; else echo "MISSING: $path"; fi
        done
        cargo metadata --no-deps --format-version 1
    '
}

cmd_open_logs() {
    mkdir -p "$LOG_DIR"
    echo "Logs folder: $LOG_DIR"

    if command -v explorer.exe >/dev/null 2>&1; then
        explorer.exe "$(cygpath -w "$LOG_DIR" 2>/dev/null || echo "$LOG_DIR")" >/dev/null 2>&1 || true
    elif command -v xdg-open >/dev/null 2>&1; then
        xdg-open "$LOG_DIR" >/dev/null 2>&1 || true
    elif command -v open >/dev/null 2>&1; then
        open "$LOG_DIR" >/dev/null 2>&1 || true
    fi
}

cmd_package_source_inner() {
    set -euo pipefail
    rm -f "$SR_ARCHIVE_PATH"
    mkdir -p artifacts

    echo "Creating source archive: $SR_ARCHIVE_PATH"
    echo "Excluding: target, .git, logs, artifacts, existing zip files, temporary root artifacts"

    if command -v zip >/dev/null 2>&1; then
        find . \
            \( -path "./target" -o -path "./.git" -o -path "./logs" -o -path "./artifacts" \) -prune -o \
            -type f \
            ! -name "*.zip" \
            ! -name "cratesappsrcbin*.rs*" \
            ! -name "cratesappsrcbineditor.rs*" \
            -print | zip -@ "$SR_ARCHIVE_PATH"
    elif command -v 7z >/dev/null 2>&1; then
        7z a -tzip "$SR_ARCHIVE_PATH" . \
            -xr!target -xr!.git -xr!logs -xr!artifacts -xr!*.zip \
            -xr!cratesappsrcbin*.rs* -xr!cratesappsrcbineditor.rs*
    elif command -v powershell.exe >/dev/null 2>&1; then
        powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& {
            \$ErrorActionPreference = 'Stop'
            \$Root = (Resolve-Path '.').Path
            \$Archive = \$env:SR_ARCHIVE_PATH
            \$Stage = Join-Path \$env:TEMP ('starlight_ridge_zip_stage_' + [guid]::NewGuid().ToString('N'))
            Remove-Item -LiteralPath \$Stage -Recurse -Force -ErrorAction SilentlyContinue
            New-Item -ItemType Directory -Path \$Stage | Out-Null
            \$Exclude = @('target', '.git', 'logs', 'artifacts')
            Get-ChildItem -LiteralPath \$Root -Force | Where-Object {
                \$Exclude -notcontains \$_.Name -and
                \$_.Name -notlike '*.zip' -and
                \$_.Name -notlike 'cratesappsrcbin*.rs*' -and
                \$_.Name -notlike 'cratesappsrcbineditor.rs*'
            } | ForEach-Object {
                Copy-Item -LiteralPath \$_.FullName -Destination \$Stage -Recurse -Force
            }
            Compress-Archive -Path (Join-Path \$Stage '*') -DestinationPath \$Archive -Force
            Remove-Item -LiteralPath \$Stage -Recurse -Force -ErrorAction SilentlyContinue
            Write-Host ('Archive created: ' + \$Archive)
        }"
    else
        echo "No archiver found. Install zip/7z or use PowerShell."
        exit 1
    fi

    echo
    echo "Archive ready: $SR_ARCHIVE_PATH"
}

cmd_package_source() {
    cmd_sanitize || return $?

    local stamp archive
    stamp="$(now_stamp)"
    archive="$ARTIFACT_DIR/starlight_ridge_source_${stamp}.zip"
    export SR_ARCHIVE_PATH="$archive"

    run_logged "package_source_zip" cmd_package_source_inner
}

cmd_diagnostics_inner() {
    set -euo pipefail
    rm -f "$SR_DIAG_ARCHIVE_PATH"
    mkdir -p artifacts

    local stage
    stage="$ARTIFACT_DIR/diagnostic_stage_$$_$(now_stamp)"
    rm -rf "$stage"
    mkdir -p "$stage"
    export SR_DIAG_STAGE="$stage"

    cleanup_diag_stage() {
        rm -rf "$stage" 2>/dev/null || true
    }
    trap cleanup_diag_stage EXIT

    echo "Creating diagnostic archive: $SR_DIAG_ARCHIVE_PATH"
    echo "Including a staged copy of logs plus core source/build metadata."

    local diag_items=(
        "logs"
        "build.sh"
        "Cargo.toml"
        "Cargo.lock"
        "crates/app/Cargo.toml"
        "crates/app/src"
        "crates/engine_render_gl/src"
        "crates/engine_window/src"
        "content/maps"
        "content/tiles"
        "docs/RUNTIME_DIAGNOSTICS.md"
        "UPDATED_SOURCE_NOTES.md"
    )

    local item dest_parent
    for item in "${diag_items[@]}"; do
        if [ -e "$item" ]; then
            dest_parent="$stage/$(dirname "$item")"
            mkdir -p "$dest_parent"
            cp -a "$item" "$dest_parent/"
            echo "Staged: $item"
        else
            echo "Skipping missing diagnostic item: $item"
        fi
    done

    if command -v zip >/dev/null 2>&1; then
        (cd "$stage" && zip -r "$SR_DIAG_ARCHIVE_PATH" .)
    elif command -v 7z >/dev/null 2>&1; then
        (cd "$stage" && 7z a -tzip "$SR_DIAG_ARCHIVE_PATH" .)
    elif command -v powershell.exe >/dev/null 2>&1; then
        powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& {
            \$ErrorActionPreference = 'Stop'
            \$Archive = \$env:SR_DIAG_ARCHIVE_PATH
            \$Stage = \$env:SR_DIAG_STAGE
            Compress-Archive -Path (Join-Path \$Stage '*') -DestinationPath \$Archive -Force
            Write-Host ('Diagnostic archive created: ' + \$Archive)
        }"
    else
        echo "No archiver found. Upload the logs folder manually."
        exit 1
    fi

    echo
    echo "Diagnostic archive ready: $SR_DIAG_ARCHIVE_PATH"
}

cmd_diagnostics() {
    local stamp archive
    stamp="$(now_stamp)"
    archive="$ARTIFACT_DIR/starlight_ridge_diagnostics_${stamp}.zip"
    export SR_DIAG_ARCHIVE_PATH="$archive"

    run_logged "create_diagnostic_bundle" cmd_diagnostics_inner
}

cmd_open_artifacts() {
    mkdir -p "$ARTIFACT_DIR"
    echo "Artifacts folder: $ARTIFACT_DIR"

    if command -v explorer.exe >/dev/null 2>&1; then
        explorer.exe "$(cygpath -w "$ARTIFACT_DIR" 2>/dev/null || echo "$ARTIFACT_DIR")" >/dev/null 2>&1 || true
    elif command -v xdg-open >/dev/null 2>&1; then
        xdg-open "$ARTIFACT_DIR" >/dev/null 2>&1 || true
    elif command -v open >/dev/null 2>&1; then
        open "$ARTIFACT_DIR" >/dev/null 2>&1 || true
    fi
}

menu() {
    while true; do
        print_header
        echo "1) Check tools"
        echo "2) cargo check"
        echo "3) cargo build"
        echo "4) cargo build --release"
        echo "5) Run game"
        echo "6) Run editor"
        echo "7) Check + build"
        echo "8) cargo clean"
        echo "9) Open/show logs folder"
        echo "10) Sanitize repo root"
        echo "11) Create clean source zip"
        echo "12) Open/show artifacts folder"
        echo "13) Run release game exe"
        echo "14) Build release editor"
        echo "15) Run release editor exe"
        echo "16) Create diagnostic bundle"
        echo "17) Run web editor on LAN (read-only)"
        echo "18) Run web editor on LAN (save enabled)"
        echo "19) Self-heal project root + docs"
        echo "20) Consolidate legacy root scripts"
        echo "21) Organize patch docs"
        echo "0) Exit"
        echo
        if ! read -r -p "Select an option: " choice; then
            echo "No input was received. If this window opened and closed on Windows, use BUILD_MENU.bat or build.bat."
            exit 1
        fi
        echo

        case "$choice" in
            1) check_tools; pause ;;
            2) cmd_check; pause ;;
            3) cmd_build; pause ;;
            4) cmd_release; pause ;;
            5) cmd_run; pause ;;
            6) cmd_editor; pause ;;
            7) cmd_all; pause ;;
            8) cmd_clean; pause ;;
            9) cmd_open_logs; pause ;;
            10) cmd_sanitize; pause ;;
            11) cmd_package_source; pause ;;
            12) cmd_open_artifacts; pause ;;
            13) cmd_run_release; pause ;;
            14) cmd_release_editor; pause ;;
            15) cmd_editor_release; pause ;;
            16) cmd_diagnostics; pause ;;
            17) cmd_web_editor_lan; pause ;;
            18) cmd_web_editor_lan_write; pause ;;
            19) cmd_self_heal; pause ;;
            20) cmd_consolidate_root_scripts; pause ;;
            21) cmd_organize_patch_docs; pause ;;
            0) exit 0 ;;
            *) echo "Invalid option."; pause ;;
        esac
    done
}

case "${1:-menu}" in
    menu) menu ;;
    tools|check-tools) check_tools ;;
    check) cmd_check ;;
    build) cmd_build ;;
    release) cmd_release ;;
    run|game) cmd_run ;;
    editor) cmd_editor ;;
    run-release|release-game) cmd_run_release ;;
    build-editor|editor-build) cmd_build_editor ;;
    release-editor|editor-release-build) cmd_release_editor ;;
    editor-release|run-editor-release) cmd_editor_release ;;
    diagnostics|diag) cmd_diagnostics ;;
    all) cmd_all ;;
    clean) cmd_clean ;;
    sanitize|repair-root) cmd_sanitize ;;
    package|zip|archive) cmd_package_source ;;
    artifacts|open-artifacts) cmd_open_artifacts ;;
    web-editor|web-editor-lan) cmd_web_editor_lan ;;
    web-editor-write|web-editor-lan-write) cmd_web_editor_lan_write ;;
    self-heal|repair) cmd_self_heal ;;
    consolidate-root-scripts) cmd_consolidate_root_scripts ;;
    organize-patch-docs) cmd_organize_patch_docs ;;
    logs|open-logs) cmd_open_logs ;;
    help|-h|--help)
        echo "Starlight Ridge Build Utility"
        echo
        echo "Usage: ./build.sh [menu|tools|check|build|release|run|editor|web-editor|web-editor-write|run-release|build-editor|release-editor|editor-release|diagnostics|all|clean|sanitize|self-heal|consolidate-root-scripts|organize-patch-docs|package|artifacts|logs]"
        echo "Windows: use BUILD_MENU.bat or build.bat if double-clicking build.sh closes immediately."
        ;;
    *)
        echo "Unknown command: $1"
        echo "Use: ./build.sh help"
        exit 2
        ;;
esac
