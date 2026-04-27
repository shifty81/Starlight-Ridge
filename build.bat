@echo off
setlocal EnableExtensions
cd /d "%~dp0"

set "CMD_ARG=%~1"
if "%CMD_ARG%"=="" set "CMD_ARG=menu"

if not exist "%~dp0tools\build_menu.ps1" (
    echo Missing tools\build_menu.ps1
    echo Re-extract the full project zip and try again.
    echo.
    pause
    exit /b 1
)

echo Using native Windows PowerShell build utility.
echo Project: %cd%
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\build_menu.ps1" "%CMD_ARG%"
set "EXITCODE=%ERRORLEVEL%"

echo.
echo Build utility exited with code %EXITCODE%.
echo This window is intentionally held open so errors are visible.
echo.
pause
exit /b %EXITCODE%
