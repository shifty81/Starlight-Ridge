fn main() {
    app::init_runtime_logging("voxel_panel_editor");

    if let Err(error) = app::run_native_editor_app("voxel_panel_editor") {
        app::write_runtime_failure("voxel_panel_editor", &error);
        eprintln!("Starlight Ridge Voxel Panel Editor failed: {error:#}");
        std::process::exit(1);
    }
}
