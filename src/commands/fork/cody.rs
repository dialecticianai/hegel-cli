/// Cody agent implementation
///
/// Cody uses `cody chat -m` for non-interactive chat mode.
/// Supports passthrough args like --context-file, --context-repo, --stdin, etc.

/// Build command arguments for Cody
pub fn build_args(prompt: Option<&str>, passthrough_args: &[String]) -> Vec<String> {
    let mut args = vec!["chat".to_string()];

    // Add passthrough args first
    args.extend_from_slice(passthrough_args);

    // Add -m flag and prompt if provided
    if let Some(p) = prompt {
        args.push("-m".to_string());
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
        assert_eq!(args, vec!["chat", "-m", "test prompt"]);
    }

    #[test]
    fn test_build_args_with_passthrough() {
        let passthrough = vec!["--context-file".to_string(), "README.md".to_string()];
        let args = build_args(Some("test prompt"), &passthrough);
        assert_eq!(
            args,
            vec!["chat", "--context-file", "README.md", "-m", "test prompt"]
        );
    }

    #[test]
    fn test_build_args_with_stdin() {
        let passthrough = vec!["--stdin".to_string()];
        let args = build_args(Some("Explain this diff"), &passthrough);
        assert_eq!(args, vec!["chat", "--stdin", "-m", "Explain this diff"]);
    }
}
