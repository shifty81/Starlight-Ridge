#!/usr/bin/env python3
import subprocess, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
raise SystemExit(subprocess.call(["cargo", "run", "-p", "voxel_generator", "--", "--all", "--project-root", str(ROOT)], cwd=ROOT))
