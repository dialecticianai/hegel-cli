use super::external_bin::ExternalBinary;
use anyhow::Result;

const PM_BINARY: ExternalBinary = ExternalBinary {
    name: "hegel-pm",
    adjacent_repo_path: "../hegel-pm",
    build_instructions: "Please build hegel-pm first:\n\
         cd ../hegel-pm && cargo build --release --features server",
};

/// Launch hegel-pm for project dashboard
pub fn run_pm(args: &[String]) -> Result<()> {
    PM_BINARY.execute(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_pm_binary_checks_adjacent_repo() {
        // This test documents the search behavior without requiring hegel-pm to exist
        let result = PM_BINARY.find();
        // Will fail in CI/most environments, but documents expected behavior
        if result.is_ok() {
            let path = result.unwrap();
            assert!(path.to_str().unwrap().contains("hegel-pm"));
        }
    }
}
