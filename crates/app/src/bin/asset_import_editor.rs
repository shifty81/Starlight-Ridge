fn main() {
    app::init_runtime_logging("asset_editor");

    if let Err(error) = app::run_native_editor_app("asset_editor") {
        app::write_runtime_failure("asset_editor", &error);
        eprintln!("Starlight Ridge Asset Editor failed: {error:#}");
        std::process::exit(1);
    }
}
