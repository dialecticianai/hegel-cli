use anyhow::Result;
use std::collections::HashSet;

/// Claim aliases for ergonomic workflow transitions
#[derive(Debug, Clone, PartialEq)]
pub enum ClaimAlias {
    /// Happy-path: {current_node}_complete
    Next,
    /// Restart workflow cycle: restart_cycle
    Restart,
    /// Custom claim name
    Custom(String),
}

impl ClaimAlias {
    /// Convert claim alias to HashSet for engine consumption
    pub fn to_claims(&self, current_node: &str) -> Result<HashSet<String>> {
        match self {
            Self::Next => Ok(HashSet::from([format!("{}_complete", current_node)])),
            Self::Restart => Ok(HashSet::from(["restart_cycle".to_string()])),
            Self::Custom(claim) => Ok(HashSet::from([claim.to_string()])),
        }
    }
}
