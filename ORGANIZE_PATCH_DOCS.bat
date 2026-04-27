@echo off
setlocal
set ROOT=%~dp0
python "%ROOT%tools\organize_patch_docs.py"
if errorlevel 1 (
  echo Failed to organize patch docs. Make sure Python is installed or move PATCH/README_PATCH files into docs\patches manually.
  exit /b 1
)
echo Patch docs organized under docs\patches.
