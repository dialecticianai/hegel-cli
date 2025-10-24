/// Codex agent implementation
///
/// Codex uses `codex exec` for non-interactive mode.
/// Supports passthrough args like --full-auto, --model, etc.

/// Build command arguments for Codex
pub fn build_args(prompt: Option<&str>, passthrough_args: &[String]) -> Vec<String> {
    let mut args = vec!["exec".to_string()];

    // Add passthrough args first (before prompt)
    args.extend_from_slice(passthrough_args);

    // Add prompt if provided
    if let Some(p) = prompt {
        args.push(p.to_string());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_args_with_prompt() {
        let args = build_args(Some("test prompt"), &[]);
        assert_eq!(args, vec!["exec", "test prompt"]);
    }

    #[test]
    fn test_build_args_with_passthrough() {
        let passthrough = vec!["--full-auto".to_string(), "--model=gpt-4".to_string()];
        let args = build_args(Some("test prompt"), &passthrough);
        assert_eq!(
            args,
            vec!["exec", "--full-auto", "--model=gpt-4", "test prompt"]
        );
    }

    #[test]
    fn test_build_args_no_prompt() {
        let args = build_args(None, &["--help".to_string()]);
        assert_eq!(args, vec!["exec", "--help"]);
    }
}
