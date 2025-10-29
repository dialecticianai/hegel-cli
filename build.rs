use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Tell Cargo to rerun this build script if the vendor directory changes
    println!("cargo:rerun-if-changed=vendor/ast-grep");

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
