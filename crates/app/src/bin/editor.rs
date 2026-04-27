fn main() {
    app::init_runtime_logging("editor");

    if let Err(error) = app::run_editor() {
        app::write_runtime_failure("editor", &error);
        eprintln!("Starlight Ridge editor failed: {error:#}");
        std::process::exit(1);
    }
}
