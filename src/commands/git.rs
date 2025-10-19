use crate::commands::wrapped::run_wrapped_command;
use crate::storage::FileStorage;
use anyhow::Result;

/// Wrap git command with guardrails and audit logging
pub fn run_git(args: &[String], storage: &FileStorage) -> Result<()> {
    run_wrapped_command("git", args, storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_run_git_delegates_to_wrapped_command() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Test that run_git properly delegates to run_wrapped_command
        // Use --version which doesn't require a git repo
        let result = run_git(&["--version".to_string()], &storage);
        assert!(result.is_ok());

        // Verify command was logged
        let log = storage.read_command_log().unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].command, "git");
        assert_eq!(log[0].args, vec!["--version"]);
        assert!(log[0].success);
    }

    #[test]
    fn test_run_git_with_multiple_args() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Test with multiple arguments (--version --build-options)
        let result = run_git(
            &["--version".to_string(), "--build-options".to_string()],
            &storage,
        );
        assert!(result.is_ok());

        // Verify all args were passed through
        let log = storage.read_command_log().unwrap();
        assert_eq!(log[0].args, vec!["--version", "--build-options"]);
    }
}
