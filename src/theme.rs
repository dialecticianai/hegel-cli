//! Terminal color theme
//!
//! Centralized theme system for consistent terminal output styling.
//! All color decisions should go through this module to enable:
//! - Consistent visual language across commands
//! - Easy theme switching (light/dark/no-color)
//! - Semantic color naming (not "cyan" but "metric_value")

use colored::{ColoredString, Colorize};

/// Default Hegel theme
pub struct Theme;

impl Theme {
    // Semantic tokens for metrics/data display

    /// Primary metric values (counts, numbers)
    pub fn metric_value(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().cyan()
    }

    /// Total/summary values (emphasis)
    pub fn metric_total(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().bold().green()
    }

    /// Secondary/dimmed information
    pub fn secondary(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().bright_black()
    }

    // Status indicators

    /// Success messages
    pub fn success(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().green()
    }

    /// Warning messages
    pub fn warning(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().yellow()
    }

    /// Error messages
    pub fn error(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().red()
    }

    // UI elements

    /// Section headers
    pub fn header(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().bold().cyan()
    }

    /// Labels (keys in key-value pairs)
    pub fn label(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().bold()
    }

    /// Highlighted values in context
    pub fn highlight(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().cyan()
    }

    /// Active/current state indicator
    pub fn active(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().green()
    }

    /// Completed/past state
    pub fn completed(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().bright_black()
    }

    // Workflow-specific

    /// Workflow mode display
    pub fn mode(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().cyan()
    }

    /// Node/phase names
    pub fn node(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().cyan()
    }

    /// Transition arrows/connectors
    pub fn connector(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().bright_black()
    }

    /// Prompt content indicator
    pub fn prompt_label(text: impl AsRef<str>) -> ColoredString {
        text.as_ref().bold().cyan()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_tokens_return_colored_strings() {
        assert_eq!(
            Theme::metric_value("100").to_string(),
            "100".cyan().to_string()
        );
        assert_eq!(Theme::success("OK").to_string(), "OK".green().to_string());
        assert_eq!(
            Theme::header("Title").to_string(),
            "Title".bold().cyan().to_string()
        );
    }

    #[test]
    fn test_theme_accepts_string_types() {
        // Should accept &str
        let _ = Theme::metric_value("test");

        // Should accept String
        let s = String::from("test");
        let _ = Theme::metric_value(&s);

        // Should accept format! output
        let _ = Theme::metric_value(format!("{}", 42));
    }
}
