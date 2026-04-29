@echo off
setlocal EnableExtensions
cd /d "%~dp0"
set "STARLIGHT_WEB_HOST=0.0.0.0"
set "STARLIGHT_WEB_ALLOW_WRITE=1"
if "%STARLIGHT_WEB_PORT%"=="" set "STARLIGHT_WEB_PORT=8787"
echo Starting Starlight Ridge Web Editor on your LAN with repo save enabled...
echo.
echo Tablet URL will be printed after the server starts.
echo If Windows Firewall prompts, allow private network access.
echo.
echo WARNING: Browser edits can overwrite content/maps/*/layers.ron.
echo A layers.ron.web_backup file is written before save.
echo.
cargo run -p web_editor_server
echo.
echo Web editor exited.
pause
