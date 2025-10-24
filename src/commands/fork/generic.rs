/// Generic agent implementation
///
/// Used for agents without specific implementations.
/// Simply passes prompt and args as-is.

/// Build command arguments for generic agent
pub fn build_args(prompt: Option<&str>, passthrough_args: &[String]) -> Vec<String> {
    let mut args = Vec::new();

    // Add passthrough args first
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
        assert_eq!(args, vec!["test prompt"]);
    }

    #[test]
    fn test_build_args_with_passthrough() {
        let passthrough = vec!["--flag".to_string()];
        let args = build_args(Some("test prompt"), &passthrough);
        assert_eq!(args, vec!["--flag", "test prompt"]);
    }
}
