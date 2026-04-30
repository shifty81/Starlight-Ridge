fn main() {
    app::init_runtime_logging("game_gui_editor");

    if let Err(error) = app::run_native_editor_app("game_gui_editor") {
        app::write_runtime_failure("game_gui_editor", &error);
        eprintln!("Starlight Ridge Game GUI Editor failed: {error:#}");
        std::process::exit(1);
    }
}
