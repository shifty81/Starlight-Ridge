@echo off
pushd "%~dp0\..\..\.."
cargo run -p voxel_generator -- --all --project-root .
popd
