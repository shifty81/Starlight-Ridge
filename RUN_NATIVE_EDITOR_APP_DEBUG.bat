@echo off
setlocal EnableExtensions
cd /d "%~dp0"

set "EDITOR_BIN=%~1"
set "EDITOR_LABEL=%~2"
if "%EDITOR_BIN%"=="" (
    echo Missing editor binary name.
    echo Usage: RUN_NATIVE_EDITOR_APP_DEBUG.bat bin_name "Display Name"
    pause
    exit /b 2
)
if "%EDITOR_LABEL%"=="" set "EDITOR_LABEL=%EDITOR_BIN%"

set "EDITOR_EXE=%~dp0target\debug\%EDITOR_BIN%.exe"
echo Running Starlight Ridge native editor app: %EDITOR_LABEL%
echo Binary: %EDITOR_BIN%
echo.

if not exist "%EDITOR_EXE%" (
    echo Debug executable is missing; building %EDITOR_BIN% first.
    cargo build -p app --bin %EDITOR_BIN%
    if errorlevel 1 (
        echo.
        echo Build failed for %EDITOR_BIN%.
        pause
        exit /b %ERRORLEVEL%
    )
)

set RUST_LOG=info
set RUST_BACKTRACE=1
"%EDITOR_EXE%"

echo.
echo Native editor app exited with code %ERRORLEVEL%.
pause
exit /b %ERRORLEVEL%
