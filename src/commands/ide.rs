use super::external_bin::ExternalNpmPackage;
use anyhow::Result;

const IDE_PACKAGE: ExternalNpmPackage = ExternalNpmPackage {
    name: "hegel-ide",
    adjacent_repo_path: "../hegel-ide",
    build_instructions: "Please install hegel-ide first:\n\
        cd ../hegel-ide && npm install && npx electron-rebuild",
};

/// Launch hegel-ide (Electron-based IDE)
pub fn run_ide(args: &[String]) -> Result<()> {
    IDE_PACKAGE.execute(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ide_package_configuration() {
        // This test documents the expected configuration
        assert_eq!(IDE_PACKAGE.name, "hegel-ide");
        assert_eq!(IDE_PACKAGE.adjacent_repo_path, "../hegel-ide");
        assert!(IDE_PACKAGE.build_instructions.contains("npm install"));
    }
}
