use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut project_root = PathBuf::from(".");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--project-root" {
            if let Some(value) = args.next() {
                project_root = PathBuf::from(value);
            }
        }
    }
    let written = voxel_generator::generate_phase53b_templates(project_root)?;
    println!("[voxel_generator] generated {} template(s)", written.len());
    for path in written {
        println!("  {}", path.display());
    }
    Ok(())
}
