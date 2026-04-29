@echo off
setlocal EnableExtensions
cd /d "%~dp0"
set "STARLIGHT_WEB_HOST=0.0.0.0"
if "%STARLIGHT_WEB_PORT%"=="" set "STARLIGHT_WEB_PORT=8787"
echo Starting Starlight Ridge Web Editor on your LAN...
echo.
echo Tablet URL will be printed after the server starts.
echo If Windows Firewall prompts, allow private network access.
echo.
cargo run -p web_editor_server
echo.
echo Web editor exited.
pause
