# Windows Build Launchers

Use `BUILD_MENU.bat` or `build.bat` on Windows.

These launchers use the native Windows PowerShell build utility at:

```text
tools/build_menu.ps1
```

This avoids the WSL `C:\Windows\System32\bash.exe` trap where Windows opens Bash but then exits because no WSL Linux distribution is installed.

## Recommended workflow

Double-click:

```text
BUILD_MENU.bat
```

Then use:

```text
2) cargo check
4) cargo build --release game exe
5) Run game debug
6) Run editor debug
16) Create diagnostic bundle
```

## Shortcut launchers

```text
RUN_GAME_DEBUG.bat
RUN_EDITOR_DEBUG.bat
RUN_GAME_RELEASE.bat
RUN_EDITOR_RELEASE.bat
BUILD_EDITOR_RELEASE.bat
BUILD_RELEASE_ALL.bat
CREATE_DIAGNOSTICS.bat
```

## Notes

Cargo prints normal compile progress and warnings to stderr. Earlier PowerShell menu builds could display a misleading `NativeCommandError` block even when Cargo succeeded. The PowerShell utility now routes native command stderr through `cmd.exe` first, so the log should show normal Cargo output and rely on the real exit code.

`build.sh` is still available for Git Bash users, but it is no longer required for the normal Windows workflow.

If you specifically want the Git Bash version, use:

```text
BUILD_MENU_GIT_BASH.bat
```

That launcher deliberately checks Git for Windows locations first and does not use the WSL bash path.
