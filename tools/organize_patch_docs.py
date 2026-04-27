from pathlib import Path
root = Path(__file__).resolve().parents[1]
docs = root / "docs"
patches = docs / "patches"
patches.mkdir(parents=True, exist_ok=True)
markers = (
    "PATCH",
    "HOTFIX",
    "MANIFEST_phase",
    "PATCH_MANIFEST",
    "README_PHASE18",
    "README_ARTPASS",
)
for path in docs.iterdir():
    if path.is_file() and any(marker in path.name for marker in markers):
        target = patches / path.name
        if target.exists():
            target.unlink()
        path.rename(target)
print(f"Organized patch docs under {patches}")
