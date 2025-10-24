/// Gemini agent implementation
///
/// Gemini uses positional arguments for prompts.
/// Supports passthrough args like -o json, --model, etc.

/// Build command arguments for Gemini
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
        let passthrough = vec!["-o".to_string(), "json".to_string()];
        let args = build_args(Some("test prompt"), &passthrough);
        assert_eq!(args, vec!["-o", "json", "test prompt"]);
    }

    #[test]
    fn test_build_args_no_prompt() {
        let args = build_args(None, &["--help".to_string()]);
        assert_eq!(args, vec!["--help"]);
    }
}
