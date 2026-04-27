fn main() {
    app::init_runtime_logging("game");

    if let Err(error) = app::run() {
        app::write_runtime_failure("game", &error);
        eprintln!("Starlight Ridge game failed: {error:#}");
        std::process::exit(1);
    }
}
