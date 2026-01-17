use std::{
    env, fs,
    path::{Path, PathBuf},
};

const COMMANDS: &[&str] = &[
    "load_translations",
    "translate",
    "set_locale",
    "get_locale",
    "get_available_locales",
];

fn main() {
    // Only bundle locales if we can find them (i.e., when used as dependency)
    // Skip during `cargo publish` or standalone builds
    if should_bundle_locales() {
        bundle_locales();
    } else {
        // Generate empty bundled_locales.rs for standalone builds
        generate_empty_bundled_locales();
    }
    tauri_plugin::Builder::new(COMMANDS).build();
}

fn should_bundle_locales() -> bool {
    let out_dir = env::var("OUT_DIR").unwrap();
    find_workspace_root(Path::new(&out_dir)).is_some()
}

fn generate_empty_bundled_locales() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("bundled_locales.rs");

    println!("cargo:warning=No locales found - generating empty bundle (this is normal during cargo publish)");

    let code = "pub fn get_bundled_data() -> Vec<(&'static str, &'static str, &'static str)> {\n    vec![]\n}\n";

    fs::write(dest_path, code).expect("Failed to write bundled_locales.rs");
}

fn find_workspace_root(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir;

    // Walk up the directory tree looking for src-tauri
    while let Some(parent) = current.parent() {
        let src_tauri = parent.join("src-tauri");
        if src_tauri.exists() && src_tauri.is_dir() {
            println!("cargo:info=Found workspace root: {}", parent.display());
            return Some(parent.to_path_buf());
        }
        current = parent;
    }
    None
}

fn bundle_locales() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("bundled_locales.rs");

    println!("cargo:info=OUT_DIR: {}", out_dir);

    // Find workspace root by walking up from OUT_DIR
    let workspace_root = find_workspace_root(Path::new(&out_dir))
        .expect("Could not find workspace root (looking for src-tauri directory)");

    let locales_path = workspace_root.join("src-tauri").join("locales");

    println!(
        "cargo:info=Looking for locales at: {}",
        locales_path.display()
    );

    if !locales_path.exists() {
        panic!(
            "Locales directory does not exist: {}",
            locales_path.display()
        );
    }

    println!("cargo:rerun-if-changed={}", locales_path.display());

    let mut code = String::from(
        "pub fn get_bundled_data() -> Vec<(&'static str, &'static str, &'static str)> {\n    vec![\n"
    );

    match fs::read_dir(&locales_path) {
        Ok(entries) => {
            let mut count = 0;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let (Some(stem), Some(ext)) = (
                        path.file_stem().and_then(|s| s.to_str()),
                        path.extension().and_then(|s| s.to_str()),
                    ) {
                        count += 1;
                        println!("cargo:info=  Bundling: {}.{}", stem, ext);
                        code.push_str(&format!(
                            "        ({:?}, {:?}, include_str!(r#\"{}\"#)),\n",
                            stem,
                            ext,
                            path.display()
                        ));
                    }
                }
            }
            println!("cargo:info=Successfully bundled {} locale file(s)", count);
        }
        Err(e) => {
            panic!(
                "Failed to read locales directory at {}: {}",
                locales_path.display(),
                e
            );
        }
    }

    code.push_str("    ]\n}\n");

    fs::write(dest_path, code).expect("Failed to write bundled_locales.rs");
}
