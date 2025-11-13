use super::*;
use crate::doctor::migrations::{self, StateMigration};
use crate::storage::{State, WorkflowState};
use crate::test_helpers::*;

fn default_args() -> DoctorArgs {
    DoctorArgs {
        dry_run: false,
        apply: false,
        verbose: false,
        json: false,
    }
}

#[test]
fn test_doctor_no_state_file() {
    // Empty state directory - should not error
    let (_temp_dir, storage) = test_storage_with_files(None, None);

    // Remove state.json if it exists
    let state_path = storage.state_dir().join("state.json");
    if state_path.exists() {
        std::fs::remove_file(&state_path).unwrap();
    }

    let result = doctor_command(default_args(), &storage);
    assert!(result.is_ok());
}

#[test]
fn test_doctor_healthy_state() {
    // State with small, healthy state.json
    let (_temp_dir, storage) = test_storage_with_files(None, None);

    // Create a small workflow state
    let state = State {
        workflow: Some(WorkflowState {
            workflow_id: Some("test-id".to_string()),
            mode: "discovery".to_string(),
            current_node: "spec".to_string(),
            history: vec![],
            meta_mode: None,
            phase_start_time: None,
            is_handlebars: false,
        }),
        cumulative_totals: None,
        session_metadata: None,
        git_info: None,
    };

    storage.save(&state).unwrap();

    let result = doctor_command(default_args(), &storage);
    assert!(result.is_ok());
}

#[test]
fn test_doctor_dry_run() {
    // Test dry-run mode doesn't modify state
    let (_temp_dir, storage) = test_storage_with_files(None, None);

    // Create a state
    let state = State {
        workflow: Some(WorkflowState {
            workflow_id: Some("test-id".to_string()),
            mode: "discovery".to_string(),
            current_node: "spec".to_string(),
            history: vec![],
            meta_mode: None,
            phase_start_time: None,
            is_handlebars: false,
        }),
        cumulative_totals: None,
        session_metadata: None,
        git_info: None,
    };

    storage.save(&state).unwrap();

    // Get original modification time
    let state_path = storage.state_dir().join("state.json");
    let original_modified = std::fs::metadata(&state_path).unwrap().modified().unwrap();

    // Sleep briefly to ensure timestamp would change if file was modified
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Run doctor in dry-run mode
    let mut args = default_args();
    args.dry_run = true;

    let result = doctor_command(args, &storage);
    assert!(result.is_ok());

    // Verify file wasn't modified
    let new_modified = std::fs::metadata(&state_path).unwrap().modified().unwrap();
    assert_eq!(original_modified, new_modified);
}

#[test]
fn test_doctor_verbose_output() {
    // Test verbose flag
    let (_temp_dir, storage) = test_storage_with_files(None, None);

    let state = State {
        workflow: Some(WorkflowState {
            workflow_id: Some("test-id".to_string()),
            mode: "discovery".to_string(),
            current_node: "spec".to_string(),
            history: vec![],
            meta_mode: None,
            phase_start_time: None,
            is_handlebars: false,
        }),
        cumulative_totals: None,
        session_metadata: None,
        git_info: None,
    };

    storage.save(&state).unwrap();

    let mut args = default_args();
    args.verbose = true;

    let result = doctor_command(args, &storage);
    assert!(result.is_ok());
}

#[test]
fn test_doctor_json_output() {
    // Test JSON output format
    let (_temp_dir, storage) = test_storage_with_files(None, None);

    let state = State {
        workflow: Some(WorkflowState {
            workflow_id: Some("test-id".to_string()),
            mode: "discovery".to_string(),
            current_node: "spec".to_string(),
            history: vec![],
            meta_mode: None,
            phase_start_time: None,
            is_handlebars: false,
        }),
        cumulative_totals: None,
        session_metadata: None,
        git_info: None,
    };

    storage.save(&state).unwrap();

    let mut args = default_args();
    args.json = true;

    let result = doctor_command(args, &storage);
    assert!(result.is_ok());
}

#[test]
fn test_workflow_def_migration_check_small_file() {
    // Small state.json should not trigger migration
    let (_temp_dir, storage) = test_storage_with_files(None, None);

    let state = State {
        workflow: Some(WorkflowState {
            workflow_id: Some("test-id".to_string()),
            mode: "discovery".to_string(),
            current_node: "spec".to_string(),
            history: vec![],
            meta_mode: None,
            phase_start_time: None,
            is_handlebars: false,
        }),
        cumulative_totals: None,
        session_metadata: None,
        git_info: None,
    };

    storage.save(&state).unwrap();

    let migration = migrations::WorkflowDefMigration;
    let issue = migration.check(&state, &storage).unwrap();

    assert!(issue.is_none());
}

#[test]
fn test_workflow_def_migration_check_large_file() {
    // Large state.json should trigger migration
    let (_temp_dir, storage) = test_storage_with_files(None, None);

    // Create a bloated state by adding lots of history
    let mut history = vec![];
    for i in 0..1000 {
        history.push(format!("transition_{}", i));
    }

    let state = State {
        workflow: Some(WorkflowState {
            workflow_id: Some("test-id".to_string()),
            mode: "discovery".to_string(),
            current_node: "spec".to_string(),
            history,
            meta_mode: None,
            phase_start_time: None,
            is_handlebars: false,
        }),
        cumulative_totals: None,
        session_metadata: None,
        git_info: None,
    };

    storage.save(&state).unwrap();

    // Verify file is large enough to trigger check
    let state_path = storage.state_dir().join("state.json");
    let metadata = std::fs::metadata(&state_path).unwrap();
    assert!(metadata.len() > 10_000);

    let migration = migrations::WorkflowDefMigration;
    let issue = migration.check(&state, &storage).unwrap();

    assert!(issue.is_some());
    let issue = issue.unwrap();
    assert!(issue.description.contains("state.json size"));
}

#[test]
fn test_workflow_def_migration_migrate() {
    // Test migration preserves state
    let (_temp_dir, storage) = setup_workflow_env();

    let original_state = State {
        workflow: Some(WorkflowState {
            workflow_id: Some("test-id".to_string()),
            mode: "test_workflow".to_string(),
            current_node: "start".to_string(),
            history: vec!["START".to_string()],
            meta_mode: None,
            phase_start_time: None,
            is_handlebars: false,
        }),
        cumulative_totals: None,
        session_metadata: None,
        git_info: None,
    };

    let mut migration = migrations::WorkflowDefMigration;
    let migrated = migration.migrate(original_state.clone(), &storage).unwrap();

    // Verify workflow state is preserved
    assert_eq!(migrated.workflow, original_state.workflow);
}
