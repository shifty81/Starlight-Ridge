#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
echo "========================================"
echo " Starlight Ridge Asset Lab"
echo " Root: $(pwd)"
echo "========================================"
python3 tools/asset_lab_server.py
