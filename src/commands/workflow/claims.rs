use anyhow::{Context, Result};
use std::collections::HashMap;

/// Claim aliases for ergonomic workflow transitions
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimAlias {
    /// Happy-path: {"{current_node}_complete": true}
    Next,
    /// Restart workflow cycle: {"restart_cycle": true}
    Restart,
    /// Custom claim JSON
    Custom(String),
}

impl ClaimAlias {
    /// Convert claim alias to HashMap for engine consumption
    pub fn to_claims(&self, current_node: &str) -> Result<HashMap<String, bool>> {
        match self {
            Self::Next => Ok(HashMap::from([(
                format!("{}_complete", current_node),
                true,
            )])),
            Self::Restart => Ok(HashMap::from([("restart_cycle".to_string(), true)])),
            Self::Custom(json) => serde_json::from_str(json)
                .context("Failed to parse claims JSON. Expected format: {\"claim_name\": true}"),
        }
    }
}
