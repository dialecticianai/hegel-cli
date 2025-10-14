use crate::commands::wrapped::run_wrapped_command;
use crate::storage::FileStorage;
use anyhow::Result;

/// Wrap git command with guardrails and audit logging
pub fn run_git(args: &[String], storage: &FileStorage) -> Result<()> {
    run_wrapped_command("git", args, storage)
}

// Tests are in wrapped.rs
