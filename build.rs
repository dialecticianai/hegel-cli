fn main() {
    // Tell Cargo to ONLY rerun this build script if these paths change
    // This prevents rebuilding ast-grep when just the version changes
    println!("cargo:rerun-if-changed=vendor/ast-grep");
    println!("cargo:rerun-if-changed=build.rs");

    // Only build ast-grep if bundle-ast-grep feature is enabled
    #[cfg(feature = "bundle-ast-grep")]
    {
        use std::path::{Path, PathBuf};
        use std::process::Command;

        let vendor_path = Path::new("vendor/ast-grep");
        let target_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("ast-grep-build");
        let ast_grep_bin = target_dir.join("release/ast-grep");

        // Only build if binary doesn't exist
        if !ast_grep_bin.exists() {
            eprintln!("Building ast-grep (this happens once during hegel compilation)...");

            // Create target directory
            std::fs::create_dir_all(&target_dir).expect("Failed to create target directory");

            // Build ast-grep with release profile
            let status = Command::new("cargo")
                .args(&[
                    "build",
                    "--release",
                    "--package",
                    "ast-grep",
                    "--target-dir",
                    target_dir.to_str().unwrap(),
                ])
                .current_dir(vendor_path)
                .status()
                .expect("Failed to execute cargo build for ast-grep");

            if !status.success() {
                panic!("Failed to build ast-grep");
            }

            eprintln!("ast-grep built successfully");
        }

        // Emit the path to the binary for the main code to access
        println!(
            "cargo:rustc-env=AST_GREP_BIN_PATH={}",
            ast_grep_bin.display()
        );
    }

    #[cfg(not(feature = "bundle-ast-grep"))]
    {
        // When not bundling, just set a placeholder path
        // The runtime code will fall back to system ast-grep
        println!("cargo:rustc-env=AST_GREP_BIN_PATH=");
    }
}
