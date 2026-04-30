fn main() {
    app::init_runtime_logging("character_animation_editor");

    if let Err(error) = app::run_native_editor_app("character_animation_editor") {
        app::write_runtime_failure("character_animation_editor", &error);
        eprintln!("Starlight Ridge Character / Animation Editor failed: {error:#}");
        std::process::exit(1);
    }
}
