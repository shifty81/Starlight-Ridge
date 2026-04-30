fn main() {
    app::init_runtime_logging("data_editor");

    if let Err(error) = app::run_native_editor_app("data_editor") {
        app::write_runtime_failure("data_editor", &error);
        eprintln!("Starlight Ridge Data Editor failed: {error:#}");
        std::process::exit(1);
    }
}
