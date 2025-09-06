use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Read VERSION file and set it as an environment variable for compilation
    let version = if Path::new("VERSION").exists() {
        fs::read_to_string("VERSION")
            .unwrap_or_else(|_| "0.1.0".to_string())
            .trim()
            .to_string()
    } else {
        // Fallback to Cargo.toml version
        env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
    };

    println!("cargo:rustc-env=REALM_VERSION={}", version);

    // Rerun if VERSION file changes
    println!("cargo:rerun-if-changed=VERSION");
}
