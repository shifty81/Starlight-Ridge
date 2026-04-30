fn main() {
    app::init_runtime_logging("world_editor");

    if let Err(error) = app::run_native_editor_app("world_editor") {
        app::write_runtime_failure("world_editor", &error);
        eprintln!("Starlight Ridge World Editor failed: {error:#}");
        std::process::exit(1);
    }
}
