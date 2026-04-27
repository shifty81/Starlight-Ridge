@echo off
setlocal EnableExtensions
cd /d "%~dp0"

set "BASH_EXE="
if exist "%ProgramFiles%\Git\bin\bash.exe" set "BASH_EXE=%ProgramFiles%\Git\bin\bash.exe"
if not defined BASH_EXE if exist "%ProgramFiles(x86)%\Git\bin\bash.exe" set "BASH_EXE=%ProgramFiles(x86)%\Git\bin\bash.exe"
if not defined BASH_EXE if exist "%LocalAppData%\Programs\Git\bin\bash.exe" set "BASH_EXE=%LocalAppData%\Programs\Git\bin\bash.exe"
if not defined BASH_EXE if exist "%ProgramFiles%\Git\usr\bin\bash.exe" set "BASH_EXE=%ProgramFiles%\Git\usr\bin\bash.exe"

if not defined BASH_EXE (
    echo Git Bash was not found. Use BUILD_MENU.bat instead.
    echo.
    pause
    exit /b 1
)

echo Using Git Bash: %BASH_EXE%
echo Project: %cd%
echo.
"%BASH_EXE%" "%~dp0build.sh" menu
set "EXITCODE=%ERRORLEVEL%"
echo.
echo Git Bash build utility exited with code %EXITCODE%.
echo.
pause
exit /b %EXITCODE%
