use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());

    let target_dir = Path::new(&out_dir)
        .ancestors()
        .nth(4) // ../target/{debug|release}
        .expect("Couldn't find target directory")
        .join(&profile)
        .join("rotations");

    fs::create_dir_all(&target_dir).expect("Failed to create target dir");

    let source_dir = Path::new("rotations");

    for entry in fs::read_dir(source_dir).expect("Failed to read rotations") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_name = path.file_name().unwrap();
            let dest_path = target_dir.join(file_name);
            fs::copy(&path, &dest_path).unwrap_or_else(|e| panic!("Failed to copy file: {}", e));
        }
    }

    cargo_emit::rerun_if_changed!(
        "/rotations",
        "/rotations/Enchanter.json",
        "/rotations/Warlock.json",
        "/rotations/Warrior.json",
    );
}
