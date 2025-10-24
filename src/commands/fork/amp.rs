/// Amp agent implementation
///
/// Amp uses `amp -x` or `amp --execute` for non-interactive execute mode.
/// Supports passthrough args like --dangerously-allow-all, --ide, --no-notifications, etc.

/// Build command arguments for Amp
pub fn build_args(prompt: Option<&str>, passthrough_args: &[String]) -> Vec<String> {
    let mut args = vec!["-x".to_string()];

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
        assert_eq!(args, vec!["-x", "test prompt"]);
    }

    #[test]
    fn test_build_args_with_passthrough() {
        let passthrough = vec!["--dangerously-allow-all".to_string()];
        let args = build_args(Some("test prompt"), &passthrough);
        assert_eq!(args, vec!["-x", "--dangerously-allow-all", "test prompt"]);
    }

    #[test]
    fn test_build_args_no_prompt() {
        let args = build_args(None, &["--help".to_string()]);
        assert_eq!(args, vec!["-x", "--help"]);
    }
}
