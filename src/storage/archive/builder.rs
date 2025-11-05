use anyhow::Result;

use crate::metrics::UnifiedMetrics;

use super::{
    aggregation::{aggregate_bash_commands, aggregate_file_modifications, compute_totals},
    validation::validate_workflow_id,
    PhaseArchive, TokenTotals, TransitionArchive, WorkflowArchive,
};

impl WorkflowArchive {
    /// Create archive from unified metrics
    pub fn from_metrics(
        metrics: &UnifiedMetrics,
        workflow_id: &str,
        is_synthetic: bool,
    ) -> Result<Self> {
        validate_workflow_id(workflow_id)?;

        // Extract mode from first transition
        let mode = metrics
            .state_transitions
            .first()
            .map(|t| t.mode.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Completed_at is the last transition timestamp
        let completed_at = metrics
            .state_transitions
            .last()
            .map(|t| t.timestamp.clone())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        // Convert phases
        let phases: Vec<PhaseArchive> = metrics
            .phase_metrics
            .iter()
            .map(|phase| {
                let bash_commands = aggregate_bash_commands(&phase.bash_commands);
                let file_modifications = aggregate_file_modifications(&phase.file_modifications);

                PhaseArchive {
                    phase_name: phase.phase_name.clone(),
                    start_time: phase.start_time.clone(),
                    end_time: phase.end_time.clone(),
                    duration_seconds: phase.duration_seconds,
                    tokens: TokenTotals {
                        input: phase.token_metrics.total_input_tokens,
                        output: phase.token_metrics.total_output_tokens,
                        cache_creation: phase.token_metrics.total_cache_creation_tokens,
                        cache_read: phase.token_metrics.total_cache_read_tokens,
                        assistant_turns: phase.token_metrics.assistant_turns,
                    },
                    bash_commands,
                    file_modifications,
                    git_commits: phase.git_commits.clone(),
                }
            })
            .collect();

        // Convert transitions
        let transitions: Vec<TransitionArchive> = metrics
            .state_transitions
            .iter()
            .map(|t| TransitionArchive {
                from_node: t.from_node.clone(),
                to_node: t.to_node.clone(),
                timestamp: t.timestamp.clone(),
            })
            .collect();

        // Compute totals
        let totals = compute_totals(&phases, &metrics.hook_metrics);

        Ok(Self {
            workflow_id: workflow_id.to_string(),
            mode,
            completed_at,
            session_id: metrics.session_id.clone(),
            phases,
            transitions,
            totals,
            is_synthetic,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{HookMetrics, StateTransitionEvent, TokenMetrics};
    use crate::test_helpers::test_phase_metrics;

    #[test]
    fn test_from_metrics() {
        let metrics = UnifiedMetrics {
            hook_metrics: HookMetrics::default(),
            token_metrics: TokenMetrics::default(),
            state_transitions: vec![StateTransitionEvent {
                timestamp: "2025-10-24T10:00:00Z".to_string(),
                workflow_id: Some("2025-10-24T10:00:00Z".to_string()),
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                phase: "spec".to_string(),
                mode: "discovery".to_string(),
            }],
            session_id: Some("test-session".to_string()),
            phase_metrics: vec![test_phase_metrics()],
            git_commits: vec![],
        };

        let archive =
            WorkflowArchive::from_metrics(&metrics, "2025-10-24T10:00:00Z", false).unwrap();

        assert_eq!(archive.workflow_id, "2025-10-24T10:00:00Z");
        assert_eq!(archive.mode, "discovery");
        assert_eq!(archive.session_id, Some("test-session".to_string()));
        assert_eq!(archive.phases.len(), 1);
        assert_eq!(archive.transitions.len(), 1);
    }

    #[test]
    fn test_from_metrics_with_is_synthetic() {
        let metrics = UnifiedMetrics {
            hook_metrics: HookMetrics::default(),
            token_metrics: TokenMetrics::default(),
            state_transitions: vec![StateTransitionEvent {
                timestamp: "2025-10-24T10:00:00Z".to_string(),
                workflow_id: Some("2025-10-24T10:00:00Z".to_string()),
                from_node: "START".to_string(),
                to_node: "ride".to_string(),
                phase: "ride".to_string(),
                mode: "cowboy".to_string(),
            }],
            session_id: None,
            phase_metrics: vec![],
            git_commits: vec![],
        };

        // Test explicit workflow (is_synthetic=false)
        let explicit_archive =
            WorkflowArchive::from_metrics(&metrics, "2025-10-24T10:00:00Z", false).unwrap();
        assert_eq!(explicit_archive.is_synthetic, false);

        // Test synthetic workflow (is_synthetic=true)
        let synthetic_archive =
            WorkflowArchive::from_metrics(&metrics, "2025-10-24T10:00:00Z", true).unwrap();
        assert_eq!(synthetic_archive.is_synthetic, true);
        assert_eq!(synthetic_archive.mode, "cowboy");
    }
}
