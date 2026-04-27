# Starlight Ridge native Windows build utility
# This script intentionally avoids bash/WSL so double-click launchers work on Windows.

param(
    [string]$Command = "menu"
)

$ErrorActionPreference = "Continue"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
$LogDir = Join-Path $RootDir "logs"
$ArtifactDir = Join-Path $RootDir "artifacts"
New-Item -ItemType Directory -Force -Path $LogDir, $ArtifactDir | Out-Null

function Get-TimeStamp {
    return Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
}

function Safe-Name([string]$Name) {
    return ($Name -replace '[\\/:*?"<>| ]+', '_' -replace '[^A-Za-z0-9_.-]', '')
}

function Write-Header {
    Clear-Host
    Write-Host "========================================"
    Write-Host " Starlight Ridge Build Utility"
    Write-Host " Root: $RootDir"
    Write-Host " Logs: $LogDir"
    Write-Host " Artifacts: $ArtifactDir"
    Write-Host "========================================"
    Write-Host ""
}

function Wait-Key {
    Write-Host ""
    Read-Host "Press Enter to continue" | Out-Null
}

function Invoke-CmdLine([string]$CommandLine) {
    # Cargo/rustc commonly write normal progress and warnings to stderr.
    # Capturing native stderr directly in Windows PowerShell can format that normal
    # output as NativeCommandError even when the command exits successfully.
    # Let cmd.exe merge stderr into stdout first, then use the real exit code.
    cmd.exe /d /s /c "$CommandLine 2>&1"
    $global:LASTEXITCODE = $LASTEXITCODE
}

function Invoke-Logged {
    param(
        [Parameter(Mandatory=$true)][string]$Name,
        [Parameter(Mandatory=$true)][scriptblock]$Block
    )

    $stamp = Get-TimeStamp
    $clean = Safe-Name $Name
    $logFile = Join-Path $LogDir "$stamp`_$clean.log"
    $latestFile = Join-Path $LogDir "latest.log"

    $header = @(
        "========================================",
        " Starlight Ridge Build Utility",
        " Command: $Name",
        " Started: $stamp",
        " Root: $RootDir",
        "========================================",
        ""
    )
    $header | Tee-Object -FilePath $logFile | Out-Host

    Push-Location $RootDir
    try {
        $global:LASTEXITCODE = 0
        & $Block | Tee-Object -FilePath $logFile -Append | Out-Host
        $code = $global:LASTEXITCODE
        if ($null -eq $code) { $code = 0 }
    } catch {
        $_ | Tee-Object -FilePath $logFile -Append | Out-Host
        $code = 1
    } finally {
        Pop-Location
    }

    $footer = @(
        "",
        "========================================",
        " Finished with exit code: $code",
        " Log saved to: $logFile",
        "========================================"
    )
    $footer | Tee-Object -FilePath $logFile -Append | Out-Host

    Copy-Item -Force $logFile $latestFile
    Write-Host ""
    if ($code -eq 0) {
        Write-Host "Command completed successfully. Log saved to:"
    } else {
        Write-Host "Command failed. Log saved to:"
    }
    Write-Host $latestFile
    return $code
}

function Check-Tools {
    return Invoke-Logged "check_tools" {
        Write-Host "Rust/Cargo:"
        Invoke-CmdLine "cargo --version"
        Invoke-CmdLine "rustc --version"
        Write-Host ""
        Write-Host "PowerShell:"
        $PSVersionTable.PSVersion.ToString()
        Write-Host ""
        Write-Host "Folder checks:"
        foreach ($path in @("Cargo.toml", "crates", "content", "assets")) {
            if (Test-Path (Join-Path $RootDir $path)) {
                Write-Host "OK: $path"
            } else {
                Write-Host "MISSING: $path"
                $global:LASTEXITCODE = 1
            }
        }
    }
}

function Cargo-Check { return Invoke-Logged "cargo_check" { Invoke-CmdLine "cargo check" } }
function Cargo-Build { return Invoke-Logged "cargo_build_debug" { Invoke-CmdLine "cargo build" } }
function Cargo-BuildRelease { return Invoke-Logged "cargo_build_release_game" { Invoke-CmdLine "cargo build -p app --bin app --release" } }
function Cargo-BuildEditorRelease { return Invoke-Logged "cargo_build_release_editor" { Invoke-CmdLine "cargo build -p app --bin editor --release" } }
function Cargo-BuildReleaseAll { return Invoke-Logged "cargo_build_release_all_app_bins" { Invoke-CmdLine "cargo build -p app --bins --release" } }
function Cargo-All { return Invoke-Logged "cargo_check_then_build" { Invoke-CmdLine "cargo check && cargo build" } }
function Cargo-Clean { return Invoke-Logged "cargo_clean" { Invoke-CmdLine "cargo clean" } }

function Run-GameDebug {
    return Invoke-Logged "run_game_debug" {
        $env:RUST_LOG = "info"
        $env:RUST_BACKTRACE = "1"
        Invoke-CmdLine "cargo run -p app --bin app"
    }
}

function Run-EditorDebug {
    return Invoke-Logged "run_editor_debug" {
        $env:RUST_LOG = "info"
        $env:RUST_BACKTRACE = "1"
        Invoke-CmdLine "cargo run -p app --bin editor"
    }
}

function Run-WebEditorLan {
    return Invoke-Logged "run_web_editor_lan" {
        $env:STARLIGHT_WEB_HOST = "0.0.0.0"
        if (!$env:STARLIGHT_WEB_PORT) { $env:STARLIGHT_WEB_PORT = "8787" }
        Invoke-CmdLine "cargo run -p web_editor_server"
    }
}

function Run-WebEditorLanWrite {
    return Invoke-Logged "run_web_editor_lan_write" {
        $env:STARLIGHT_WEB_HOST = "0.0.0.0"
        if (!$env:STARLIGHT_WEB_PORT) { $env:STARLIGHT_WEB_PORT = "8787" }
        $env:STARLIGHT_WEB_ALLOW_WRITE = "1"
        Invoke-CmdLine "cargo run -p web_editor_server"
    }
}

function Run-GameRelease {
    return Invoke-Logged "run_game_release_exe" {
        $env:RUST_LOG = "info"
        $env:RUST_BACKTRACE = "1"
        $exe = Join-Path $RootDir "target\release\app.exe"
        if (!(Test-Path $exe)) {
            Write-Host "Missing release exe: $exe"
            Write-Host "Run option 4 first."
            $global:LASTEXITCODE = 1
            return
        }
        Invoke-CmdLine "`"$exe`""
    }
}

function Run-EditorRelease {
    return Invoke-Logged "run_editor_release_exe" {
        $env:RUST_LOG = "info"
        $env:RUST_BACKTRACE = "1"
        $exe = Join-Path $RootDir "target\release\editor.exe"
        if (!(Test-Path $exe)) {
            Write-Host "Missing release exe: $exe"
            Write-Host "Run option 15 first, or use option 17 to build both release executables."
            $global:LASTEXITCODE = 1
            return
        }
        Invoke-CmdLine "`"$exe`""
    }
}

function Sanitize-Root {
    return Invoke-Logged "sanitize_repo_root" {
        $badNames = @(".fingerprint", "deps", "examples", "incremental", "build")
        foreach ($name in $badNames) {
            $path = Join-Path $RootDir $name
            if (Test-Path $path) {
                Write-Host "Removing misplaced root build artifact: $name"
                Remove-Item -Recurse -Force $path
            }
        }
        Get-ChildItem -Path $RootDir -Filter "*.rlib" -File -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host "Removing misplaced root rlib: $($_.Name)"
            Remove-Item -Force $_.FullName
        }
        Get-ChildItem -Path $RootDir -Filter "*.rmeta" -File -ErrorAction SilentlyContinue | ForEach-Object {
            Write-Host "Removing misplaced root rmeta: $($_.Name)"
            Remove-Item -Force $_.FullName
        }
        $staleContentFiles = @(
            "content\\tiles\\base_autotile_rules.ron"
        )
        foreach ($name in $staleContentFiles) {
            $path = Join-Path $RootDir $name
            if (Test-Path $path) {
                Write-Host "Removing stale content file from older patch: $name"
                Remove-Item -Force $path
            }
        }

        Write-Host "Sanitize complete."
    }
}


function Organize-PatchDocs {
    return Invoke-Logged "organize_patch_docs" {
        $patchDir = Join-Path $RootDir "docs\patches"
        New-Item -ItemType Directory -Force -Path $patchDir | Out-Null
        $patterns = @("PATCH_*.md", "PATCH_*.txt", "README_PATCH*.md", "README_PATCH*.txt", "README_PHASE*.txt", "MANIFEST_phase*.txt")
        foreach ($pattern in $patterns) {
            Get-ChildItem -Path $RootDir -Filter $pattern -File -ErrorAction SilentlyContinue | ForEach-Object {
                $target = Join-Path $patchDir $_.Name
                Write-Host "Moving patch doc: $($_.Name) -> docs\patches"
                Move-Item -Force $_.FullName $target
            }
        }
        Write-Host "Patch documentation organization complete."
    }
}

function Consolidate-RootScripts {
    return Invoke-Logged "consolidate_root_scripts" {
        $legacyDir = Join-Path $RootDir "tools\legacy_launchers"
        New-Item -ItemType Directory -Force -Path $legacyDir | Out-Null
        $keep = @("build.bat", "BUILD_MENU.bat", "BUILD_MENU_GIT_BASH.bat", "build.sh")
        Get-ChildItem -Path $RootDir -File -Include "*.bat", "*.cmd", "*.ps1", "*.sh" -ErrorAction SilentlyContinue | ForEach-Object {
            if ($keep -contains $_.Name) { return }
            if ($_.Name -like "BUILD_*" -or $_.Name -like "CREATE_*" -or $_.Name -like "RUN_*" -or $_.Name -like "CLEAN_*" -or $_.Name -like "ORGANIZE_*") {
                Write-Host "Moving legacy root launcher: $($_.Name) -> tools\legacy_launchers"
                Move-Item -Force $_.FullName (Join-Path $legacyDir $_.Name)
            }
        }
        Write-Host "Root launcher consolidation complete. Preferred entry: BUILD_MENU.bat or build.bat."
    }
}

function Self-HealProject {
    return Invoke-Logged "self_heal_project" {
        Write-Host "Creating required folders..."
        foreach ($dir in @("logs", "artifacts", "saves", "docs\patches", "tools\legacy_launchers")) {
            New-Item -ItemType Directory -Force -Path (Join-Path $RootDir $dir) | Out-Null
        }

        Write-Host "Checking required root files..."
        foreach ($path in @("Cargo.toml", "Cargo.lock", "content", "assets", "tools\web_editor", "crates\app", "crates\web_editor_server")) {
            $full = Join-Path $RootDir $path
            if (Test-Path $full) { Write-Host "OK: $path" } else { Write-Host "MISSING: $path"; $global:LASTEXITCODE = 1 }
        }

        Write-Host "Repairing misplaced build artifacts..."
        $badNames = @(".fingerprint", "deps", "examples", "incremental", "build")
        foreach ($name in $badNames) {
            $path = Join-Path $RootDir $name
            if (Test-Path $path) { Remove-Item -Recurse -Force $path; Write-Host "Removed: $name" }
        }
        Get-ChildItem -Path $RootDir -Filter "*.rlib" -File -ErrorAction SilentlyContinue | Remove-Item -Force
        Get-ChildItem -Path $RootDir -Filter "*.rmeta" -File -ErrorAction SilentlyContinue | Remove-Item -Force

        Write-Host "Organizing patch docs..."
        $patchDir = Join-Path $RootDir "docs\patches"
        $patterns = @("PATCH_*.md", "PATCH_*.txt", "README_PATCH*.md", "README_PATCH*.txt", "README_PHASE*.txt", "MANIFEST_phase*.txt")
        foreach ($pattern in $patterns) {
            Get-ChildItem -Path $RootDir -Filter $pattern -File -ErrorAction SilentlyContinue | ForEach-Object {
                Move-Item -Force $_.FullName (Join-Path $patchDir $_.Name)
                Write-Host "Moved patch doc: $($_.Name)"
            }
        }

        Write-Host "Checking Cargo metadata..."
        Invoke-CmdLine "cargo metadata --no-deps --format-version 1"
        Write-Host "Self-heal pass complete. Run cargo check next if Cargo metadata succeeded."
    }
}

function New-SourceZip {
    return Invoke-Logged "create_clean_source_zip" {
        $stamp = Get-TimeStamp
        $zipPath = Join-Path $ArtifactDir "Starlight_Ridge_source_$stamp.zip"
        $stage = Join-Path $ArtifactDir "_source_stage"
        if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
        New-Item -ItemType Directory -Force -Path $stage | Out-Null

        $excludeDirs = @(".git", "target", "logs", "artifacts")
        Get-ChildItem -Path $RootDir -Force | Where-Object { $excludeDirs -notcontains $_.Name } | ForEach-Object {
            Copy-Item -Path $_.FullName -Destination $stage -Recurse -Force
        }

        if (Test-Path $zipPath) { Remove-Item -Force $zipPath }
        Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zipPath -Force
        Remove-Item -Recurse -Force $stage
        Write-Host "Created source zip: $zipPath"
    }
}

function New-DiagnosticBundle {
    return Invoke-Logged "create_diagnostic_bundle" {
        $stamp = Get-TimeStamp
        $bundle = Join-Path $ArtifactDir "Starlight_Ridge_diagnostics_$stamp.zip"
        $stage = Join-Path $ArtifactDir "_diagnostics_stage"
        if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
        New-Item -ItemType Directory -Force -Path $stage | Out-Null

        foreach ($name in @("Cargo.toml", "Cargo.lock", "README.md", "UPDATED_SOURCE_NOTES.md", "build.sh", "build.bat", "BUILD_MENU.bat", "tools\build_menu.ps1")) {
            $src = Join-Path $RootDir $name
            if (Test-Path $src) {
                $dest = Join-Path $stage $name
                $parent = Split-Path -Parent $dest
                if ($parent -and !(Test-Path $parent)) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
                Copy-Item -Force $src $dest
            }
        }

        foreach ($dir in @("logs", "content", "assets", "docs")) {
            $src = Join-Path $RootDir $dir
            if (Test-Path $src) { Copy-Item -Recurse -Force $src (Join-Path $stage $dir) }
        }

        $targetInfo = Join-Path $stage "target_locations.txt"
        @(
            "Expected debug app: target\debug\app.exe",
            "Expected release app: target\release\app.exe",
            "Expected debug editor: target\debug\editor.exe",
            "Expected release editor: target\release\editor.exe"
        ) | Set-Content -Path $targetInfo

        if (Test-Path $bundle) { Remove-Item -Force $bundle }
        Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $bundle -Force
        Remove-Item -Recurse -Force $stage
        Write-Host "Created diagnostic bundle: $bundle"
    }
}

function Open-Folder([string]$Path) {
    New-Item -ItemType Directory -Force -Path $Path | Out-Null
    Start-Process explorer.exe $Path
    return 0
}

function Show-Menu {
    while ($true) {
        Write-Header
        Write-Host "1) Check tools"
        Write-Host "2) cargo check"
        Write-Host "3) cargo build"
        Write-Host "4) cargo build --release game exe"
        Write-Host "5) Run game debug"
        Write-Host "6) Run editor debug"
        Write-Host "7) Check + build"
        Write-Host "8) cargo clean"
        Write-Host "9) Open/show logs folder"
        Write-Host "10) Sanitize repo root"
        Write-Host "11) Create clean source zip"
        Write-Host "12) Open/show artifacts folder"
        Write-Host "13) Run game release exe"
        Write-Host "14) Run editor release exe"
        Write-Host "15) Build editor release exe"
        Write-Host "16) Create diagnostic bundle"
        Write-Host "17) Build all release app binaries"
        Write-Host "18) Run web editor on LAN (read-only)"
        Write-Host "19) Run web editor on LAN (save enabled)"
        Write-Host "20) Self-heal project root + docs"
        Write-Host "21) Consolidate legacy root scripts"
        Write-Host "22) Organize patch docs"
        Write-Host "0) Exit"
        Write-Host ""
        $choice = Read-Host "Select an option"
        switch ($choice) {
            "1" { Check-Tools | Out-Null; Wait-Key }
            "2" { Cargo-Check | Out-Null; Wait-Key }
            "3" { Cargo-Build | Out-Null; Wait-Key }
            "4" { Cargo-BuildRelease | Out-Null; Wait-Key }
            "5" { Run-GameDebug | Out-Null; Wait-Key }
            "6" { Run-EditorDebug | Out-Null; Wait-Key }
            "7" { Cargo-All | Out-Null; Wait-Key }
            "8" { Cargo-Clean | Out-Null; Wait-Key }
            "9" { Open-Folder $LogDir | Out-Null; Wait-Key }
            "10" { Sanitize-Root | Out-Null; Wait-Key }
            "11" { New-SourceZip | Out-Null; Wait-Key }
            "12" { Open-Folder $ArtifactDir | Out-Null; Wait-Key }
            "13" { Run-GameRelease | Out-Null; Wait-Key }
            "14" { Run-EditorRelease | Out-Null; Wait-Key }
            "15" { Cargo-BuildEditorRelease | Out-Null; Wait-Key }
            "16" { New-DiagnosticBundle | Out-Null; Wait-Key }
            "17" { Cargo-BuildReleaseAll | Out-Null; Wait-Key }
            "18" { Run-WebEditorLan | Out-Null; Wait-Key }
            "19" { Run-WebEditorLanWrite | Out-Null; Wait-Key }
            "20" { Self-HealProject | Out-Null; Wait-Key }
            "21" { Consolidate-RootScripts | Out-Null; Wait-Key }
            "22" { Organize-PatchDocs | Out-Null; Wait-Key }
            "0" { return 0 }
            default { Write-Host "Unknown option: $choice"; Wait-Key }
        }
    }
}

switch ($Command.ToLowerInvariant()) {
    "menu" { Show-Menu; exit 0 }
    "check-tools" { exit (Check-Tools) }
    "check" { exit (Cargo-Check) }
    "build" { exit (Cargo-Build) }
    "release" { exit (Cargo-BuildRelease) }
    "release-all" { exit (Cargo-BuildReleaseAll) }
    "build-editor-release" { exit (Cargo-BuildEditorRelease) }
    "run" { exit (Run-GameDebug) }
    "editor" { exit (Run-EditorDebug) }
    "web-editor" { exit (Run-WebEditorLan) }
    "web-editor-lan" { exit (Run-WebEditorLan) }
    "web-editor-write" { exit (Run-WebEditorLanWrite) }
    "web-editor-lan-write" { exit (Run-WebEditorLanWrite) }
    "run-release" { exit (Run-GameRelease) }
    "editor-release" { exit (Run-EditorRelease) }
    "all" { exit (Cargo-All) }
    "clean" { exit (Cargo-Clean) }
    "logs" { exit (Open-Folder $LogDir) }
    "artifacts" { exit (Open-Folder $ArtifactDir) }
    "sanitize" { exit (Sanitize-Root) }
    "package" { exit (New-SourceZip) }
    "diagnostics" { exit (New-DiagnosticBundle) }
    "self-heal" { exit (Self-HealProject) }
    "consolidate-root-scripts" { exit (Consolidate-RootScripts) }
    "organize-patch-docs" { exit (Organize-PatchDocs) }
    default {
        Write-Host "Unknown command: $Command"
        Write-Host "Use: menu, check, build, release, release-all, run, editor, web-editor, web-editor-write, run-release, editor-release, build-editor-release, diagnostics, self-heal, consolidate-root-scripts, organize-patch-docs"
        exit 1
    }
}
