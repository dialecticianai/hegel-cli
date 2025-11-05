use anyhow::{bail, Result};

/// Validate workflow_id for path safety
pub fn validate_workflow_id(workflow_id: &str) -> Result<()> {
    // Must not contain path separators
    if workflow_id.contains('/') || workflow_id.contains('\\') {
        bail!("Invalid workflow_id: contains path separator");
    }

    // Must not contain path traversal
    if workflow_id.contains("..") {
        bail!("Invalid workflow_id: contains path traversal");
    }

    // Must be valid ISO 8601 timestamp
    if chrono::DateTime::parse_from_rfc3339(workflow_id).is_err() {
        bail!("Invalid workflow_id: not a valid ISO 8601 timestamp");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_workflow_id() {
        // Valid ISO 8601 timestamp
        assert!(validate_workflow_id("2025-10-24T10:00:00Z").is_ok());

        // Invalid: contains slash
        assert!(validate_workflow_id("2025-10-24/foo").is_err());

        // Invalid: contains path traversal
        assert!(validate_workflow_id("../2025-10-24T10:00:00Z").is_err());

        // Invalid: not ISO 8601
        assert!(validate_workflow_id("not-a-timestamp").is_err());
    }
}
