fn main() {
    app::init_runtime_logging("editor_content_check");

    match locate_project_root().and_then(|root| {
        let registry = game_data::load_registry(&root)?;
        Ok((root, registry.summary()))
    }) {
        Ok((root, summary)) => {
            println!("content ok: {}", root.display());
            println!("{summary}");
        }
        Err(error) => {
            app::write_runtime_failure("editor_content_check", &error);
            eprintln!("content check failed: {error:#}");
            std::process::exit(1);
        }
    }
}

fn locate_project_root() -> anyhow::Result<std::path::PathBuf> {
    let mut candidate = std::env::current_dir()?.canonicalize()?;
    loop {
        if candidate.join("content").exists() && candidate.join("crates").exists() {
            return Ok(candidate);
        }
        if !candidate.pop() {
            anyhow::bail!("could not locate Starlight Ridge project root from current directory");
        }
    }
}
