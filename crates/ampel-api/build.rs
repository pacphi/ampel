use std::env;
use std::path::PathBuf;

fn main() {
    // Get the directory of the current crate
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let locales_dir = PathBuf::from(manifest_dir).join("locales");

    // Tell Cargo to rerun this build script if the locales directory changes
    println!("cargo:rerun-if-changed={}", locales_dir.display());

    // Also watch individual locale files to trigger rebuilds when translations change
    if locales_dir.exists() {
        // Recursively watch all files in locales directory
        if let Ok(entries) = std::fs::read_dir(&locales_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Watch language subdirectories
                    println!("cargo:rerun-if-changed={}", path.display());

                    // Watch YAML files in subdirectories
                    if let Ok(files) = std::fs::read_dir(&path) {
                        for file in files.flatten() {
                            let file_path = file.path();
                            if file_path.extension().and_then(|s| s.to_str()) == Some("yml") {
                                println!("cargo:rerun-if-changed={}", file_path.display());
                            }
                        }
                    }
                }
            }
        }
    }
}
