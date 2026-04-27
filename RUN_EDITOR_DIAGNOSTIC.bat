@echo off
setlocal
cd /d "%~dp0"
echo ========================================
echo  Starlight Ridge Native Editor Diagnostic Launch
echo  Root: %CD%
echo ========================================
echo.

if exist "logs\latest_runtime_failure.log" del /q "logs\latest_runtime_failure.log"
if exist "logs\editor_runtime_failure_latest.log" del /q "logs\editor_runtime_failure_latest.log"

if exist "target\release\editor.exe" (
    echo Launching target\release\editor.exe ...
    "target\release\editor.exe"
    set EXIT_CODE=%ERRORLEVEL%
) else if exist "target\debug\editor.exe" (
    echo Launching target\debug\editor.exe ...
    "target\debug\editor.exe"
    set EXIT_CODE=%ERRORLEVEL%
) else (
    echo No built editor executable found.
    echo Build first with your menu, then run this again.
    set EXIT_CODE=9009
)

echo.
echo Editor exited with code: %EXIT_CODE%
if exist "logs\latest_runtime_failure.log" (
    echo.
    echo ---- latest_runtime_failure.log ----
    type "logs\latest_runtime_failure.log"
) else if exist "logs\editor_runtime_failure_latest.log" (
    echo.
    echo ---- editor_runtime_failure_latest.log ----
    type "logs\editor_runtime_failure_latest.log"
) else if not "%EXIT_CODE%"=="0" if exist "logs\latest.log" (
    echo.
    echo ---- latest.log ----
    type "logs\latest.log"
) else (
    echo No fresh runtime failure log found.
)

echo.
pause
exit /b %EXIT_CODE%
