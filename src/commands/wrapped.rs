use crate::guardrails::{load_guardrails, RuleMatch};
use crate::storage::FileStorage;
use crate::theme::Theme;
use anyhow::{Context, Result};
use colored::Colorize;
use std::process::Command;

/// Run a wrapped command with guardrails and audit logging
///
/// This generic wrapper works for any command configured in guardrails.yaml
pub fn run_wrapped_command(
    command_name: &str,
    args: &[String],
    storage: &FileStorage,
) -> Result<()> {
    // Load guardrails
    let guardrails = load_guardrails(storage.state_dir())?;

    // Evaluate rules if command has guardrails configured
    if let Some(cmd_rules) = guardrails.get_command_guardrails(command_name) {
        match cmd_rules.evaluate(args) {
            RuleMatch::Blocked(reason) => {
                // Log blocked command
                storage.log_command(command_name, args, false, Some(&reason))?;

                // Print error and exit
                eprintln!(
                    "{}",
                    Theme::error("⛔ Command blocked by guardrails").bold()
                );
                eprintln!();
                eprintln!(
                    "{}: {} {}",
                    Theme::label("Command"),
                    command_name,
                    Theme::secondary(args.join(" "))
                );
                eprintln!("{}: {}", Theme::label("Reason"), Theme::warning(&reason));
                eprintln!();
                eprintln!(
                    "{}",
                    Theme::secondary("Edit .hegel/guardrails.yaml to modify rules.")
                );
                std::process::exit(1);
            }
            RuleMatch::Allowed | RuleMatch::NoMatch => {
                // Command is allowed, proceed
            }
        }
    }

    // Execute command
    let status = Command::new(command_name)
        .args(args)
        .status()
        .with_context(|| format!("Failed to execute command: {}", command_name))?;

    // Log successful execution
    storage.log_command(command_name, args, status.success(), None)?;

    // Exit with command's exit code
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wrapped_command_with_no_guardrails() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path()).unwrap();

        // Use git --version instead of git status (doesn't need a repo)
        let result = run_wrapped_command("git", &["--version".to_string()], &storage);
        assert!(result.is_ok());

        // Check audit log
        let log = storage.read_command_log().unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].command, "git");
        assert_eq!(log[0].args, vec!["--version"]);
        assert!(log[0].success);
    }

    #[test]
    fn test_wrapped_command_respects_guardrails() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf()).unwrap();

        // Create guardrails file
        let guardrails_yaml = r#"
git:
  blocked:
    - pattern: "clean -fd"
      reason: "Destructive operation"
"#;
        std::fs::write(temp_dir.path().join("guardrails.yaml"), guardrails_yaml).unwrap();

        // Test that guardrails are loaded and evaluated
        let guardrails = load_guardrails(temp_dir.path()).unwrap();
        let git_rules = guardrails.get_command_guardrails("git").unwrap();
        let result = git_rules.evaluate(&["clean".to_string(), "-fd".to_string()]);
        assert!(matches!(result, RuleMatch::Blocked(_)));
    }

    #[test]
    fn test_wrapped_command_with_docker() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::new(temp_dir.path().to_path_buf()).unwrap();

        // Create guardrails for docker
        let guardrails_yaml = r#"
docker:
  blocked:
    - pattern: "rm -f"
      reason: "Force remove blocked"
"#;
        std::fs::write(temp_dir.path().join("guardrails.yaml"), guardrails_yaml).unwrap();

        let guardrails = load_guardrails(temp_dir.path()).unwrap();
        let docker_rules = guardrails.get_command_guardrails("docker").unwrap();
        let result = docker_rules.evaluate(&["rm".to_string(), "-f".to_string()]);
        assert!(matches!(result, RuleMatch::Blocked(_)));
    }
}
