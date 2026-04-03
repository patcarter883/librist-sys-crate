use std::env;
use std::path::PathBuf;

fn main() {
    // Minimum version requirement for librist
    const MIN_VERSION: &str = "0.2.10";

    let mut config = pkg_config::Config::new();
    config.atleast_version(MIN_VERSION);

    match config.probe("rist") {
        Ok(library) => {
            // Handle static vs dynamic linking based on feature flag
            if env::var("CARGO_FEATURE_STATIC").is_ok() {
                println!("cargo:rustc-link-lib=static=rist");
            } else {
                println!("cargo:rustc-link-lib=rist");
            }

            // Propagate include paths to clang for potential bindgen usage
            for include_path in &library.include_paths {
                println!("cargo:clang_arg=-I{}", include_path.display());
            }

            // Set rerun-if-changed on pkg-config variables
            println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
            println!("cargo:rerun-if-env-changed=RIST_STATIC");
        }
        Err(e) => {
            // Provide helpful error message
            eprintln!("warning: pkg-config could not find librist: {}", e);
            eprintln!("warning: librist may need to be installed on your system");
            eprintln!("warning: on Ubuntu/Debian: sudo apt install librist-dev");
            eprintln!("warning: on macOS: brew install librist");
            
            // Attempt to link anyway (may work if library is in non-standard location)
            println!("cargo:rustc-link-lib=rist");
        }
    }

    // Create output directory for generated files
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = out_dir.join("include");
    
    if let Err(e) = std::fs::create_dir_all(&include_dir) {
        eprintln!("warning: could not create include directory: {}", e);
    }

    // Tell cargo when to rebuild
    println!("cargo:rerun-if-changed=build.rs");
}