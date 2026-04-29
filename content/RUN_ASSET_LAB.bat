@echo off
setlocal
cd /d "%~dp0"
echo ========================================
echo  Starlight Ridge Asset Lab
echo  Root: %CD%
echo ========================================
echo.
where python >nul 2>nul
if errorlevel 1 (
  echo Python was not found on PATH.
  echo Install Python or run: py tools\asset_lab_server.py
  pause
  exit /b 1
)
python tools\asset_lab_server.py
pause
