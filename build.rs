//! Build script for Sway.

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("examples.rs");
    
    let examples_dir = Path::new("examples");
    let mut entries = Vec::new();

    if let Ok(read_dir) = fs::read_dir(examples_dir) {
        for entry in read_dir {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    println!("cargo:rerun-if-changed={}", path.display());
                    let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
                    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
                    entries.push((name, file_name));
                }
            }
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut content = String::new();
    content.push_str("pub const EXAMPLES: &[(&str, &str)] = &[\n");
    for (name, file_name) in &entries {
        content.push_str(&format!(
            "    (\"{}\", include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/examples/{}\"))),\n",
            name, file_name
        ));
    }
    content.push_str("];\n");

    let should_write = match fs::read_to_string(&dest_path) {
        Ok(existing_content) => existing_content != content,
        Err(_) => true,
    };

    if should_write {
        fs::write(&dest_path, content).unwrap();
    }
}
