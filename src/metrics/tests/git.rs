use crate::metrics::git::*;
use crate::metrics::PhaseMetrics;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_has_git_repository_detects_valid_repo() {
    // Create temporary directory with git repo
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    let state_dir = project_root.join(".hegel");
    std::fs::create_dir(&state_dir).unwrap();

    // Initialize git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(project_root)
        .output()
        .unwrap();

    assert!(has_git_repository(&state_dir));
}

#[test]
fn test_has_git_repository_returns_false_for_no_repo() {
    // Create temporary directory without git repo
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();
    let state_dir = project_root.join(".hegel");
    std::fs::create_dir(&state_dir).unwrap();

    assert!(!has_git_repository(&state_dir));
}

#[test]
fn test_has_git_repository_handles_invalid_path() {
    // Non-existent path
    let invalid_path = Path::new("/non/existent/path/.hegel");
    assert!(!has_git_repository(invalid_path));
}

#[test]
fn test_has_git_repository_handles_root_path() {
    // Path with no parent
    let root_path = Path::new("/");
    assert!(!has_git_repository(root_path));
}

fn setup_test_repo_with_commits() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Initialize git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(project_root)
        .output()
        .unwrap();

    // Configure git
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(project_root)
        .output()
        .unwrap();
    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(project_root)
        .output()
        .unwrap();

    // Create and commit a file
    std::fs::write(project_root.join("test.txt"), "hello\nworld\n").unwrap();
    Command::new("git")
        .args(&["add", "test.txt"])
        .current_dir(project_root)
        .output()
        .unwrap();
    Command::new("git")
        .args(&["commit", "-m", "initial commit"])
        .current_dir(project_root)
        .output()
        .unwrap();

    temp_dir
}

#[test]
fn test_parse_git_commits_from_repo() {
    let temp_dir = setup_test_repo_with_commits();
    let project_root = temp_dir.path();

    let commits = parse_git_commits(project_root, None).unwrap();

    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].message, "initial commit");
    // Author may vary based on git config, just check it's not empty
    assert!(!commits[0].author.is_empty());
    assert_eq!(commits[0].files_changed, 1);
    assert_eq!(commits[0].insertions, 2);
    assert_eq!(commits[0].deletions, 0);
    assert_eq!(commits[0].hash.len(), 7);
}

#[test]
fn test_parse_git_commits_with_timestamp_filter() {
    let temp_dir = setup_test_repo_with_commits();
    let project_root = temp_dir.path();

    // Get current time + 1 day (future)
    let future_timestamp = chrono::Utc::now().timestamp() + 86400;

    let commits = parse_git_commits(project_root, Some(future_timestamp)).unwrap();

    // No commits should match (all are in the past)
    assert_eq!(commits.len(), 0);
}

#[test]
fn test_parse_git_commits_empty_repo() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path();

    // Initialize empty git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(project_root)
        .output()
        .unwrap();

    let result = parse_git_commits(project_root, None);

    // Empty repo has no HEAD, should error
    assert!(result.is_err());
}

#[test]
fn test_parse_git_commits_invalid_path() {
    let invalid_path = Path::new("/non/existent/path");
    let result = parse_git_commits(invalid_path, None);

    assert!(result.is_err());
}

#[test]
fn test_timestamp_to_iso8601() {
    let timestamp = 1730563200; // 2024-11-02T18:00:00Z
    let iso = timestamp_to_iso8601(timestamp);

    // Should be valid ISO 8601 format with date
    assert!(iso.contains("2024-11-02"));
    assert!(iso.contains("T"));
    assert!(iso.contains("Z") || iso.contains("+") || iso.contains("-"));
}

#[test]
fn test_attribute_commits_single_phase() {
    let commits = vec![GitCommit {
        hash: "abc1234".to_string(),
        timestamp: "2025-01-01T10:05:00Z".to_string(),
        message: "test".to_string(),
        author: "Test".to_string(),
        files_changed: 1,
        insertions: 5,
        deletions: 2,
    }];

    use crate::test_helpers::test_phase_metrics_with;

    let mut phases = vec![PhaseMetrics {
        start_time: "2025-01-01T10:00:00Z".to_string(),
        end_time: Some("2025-01-01T10:15:00Z".to_string()),
        ..test_phase_metrics_with(true)
    }];

    attribute_commits_to_phases(commits, &mut phases);

    assert_eq!(phases[0].git_commits.len(), 1);
    assert_eq!(phases[0].git_commits[0].hash, "abc1234");
}

#[test]
fn test_attribute_commits_multiple_phases() {
    let commits = vec![
        GitCommit {
            hash: "aaa0000".to_string(),
            timestamp: "2025-01-01T10:05:00Z".to_string(), // spec phase
            message: "in spec".to_string(),
            author: "Test".to_string(),
            files_changed: 1,
            insertions: 5,
            deletions: 2,
        },
        GitCommit {
            hash: "bbb1111".to_string(),
            timestamp: "2025-01-01T10:20:00Z".to_string(), // plan phase
            message: "in plan".to_string(),
            author: "Test".to_string(),
            files_changed: 2,
            insertions: 10,
            deletions: 3,
        },
    ];

    use crate::test_helpers::test_phase_metrics_with;

    let mut phases = vec![
        PhaseMetrics {
            start_time: "2025-01-01T10:00:00Z".to_string(),
            end_time: Some("2025-01-01T10:15:00Z".to_string()),
            ..test_phase_metrics_with(true)
        },
        PhaseMetrics {
            phase_name: "plan".to_string(),
            start_time: "2025-01-01T10:15:00Z".to_string(),
            end_time: Some("2025-01-01T10:30:00Z".to_string()),
            ..test_phase_metrics_with(true)
        },
    ];

    attribute_commits_to_phases(commits, &mut phases);

    assert_eq!(phases[0].git_commits.len(), 1);
    assert_eq!(phases[0].git_commits[0].hash, "aaa0000");
    assert_eq!(phases[1].git_commits.len(), 1);
    assert_eq!(phases[1].git_commits[0].hash, "bbb1111");
}

#[test]
fn test_attribute_commits_outside_range_ignored() {
    let commits = vec![GitCommit {
        hash: "zzz9999".to_string(),
        timestamp: "2025-01-01T11:00:00Z".to_string(), // After all phases
        message: "too late".to_string(),
        author: "Test".to_string(),
        files_changed: 1,
        insertions: 5,
        deletions: 2,
    }];

    use crate::test_helpers::test_phase_metrics_with;

    let mut phases = vec![PhaseMetrics {
        start_time: "2025-01-01T10:00:00Z".to_string(),
        end_time: Some("2025-01-01T10:15:00Z".to_string()),
        ..test_phase_metrics_with(true)
    }];

    attribute_commits_to_phases(commits, &mut phases);

    assert_eq!(phases[0].git_commits.len(), 0);
}

#[test]
fn test_attribute_commits_active_phase() {
    let commits = vec![GitCommit {
        hash: "ccc2222".to_string(),
        timestamp: "2025-01-01T10:20:00Z".to_string(),
        message: "in active".to_string(),
        author: "Test".to_string(),
        files_changed: 1,
        insertions: 5,
        deletions: 2,
    }];

    let mut phases = vec![crate::metrics::PhaseMetrics {
        phase_name: "code".to_string(),
        start_time: "2025-01-01T10:15:00Z".to_string(),
        end_time: None, // Active phase
        duration_seconds: 0,
        token_metrics: Default::default(),
        bash_commands: vec![],
        file_modifications: vec![],
        git_commits: vec![],
        is_synthetic: false,
        workflow_id: Some("2025-01-01T10:00:00Z".to_string()),
    }];

    attribute_commits_to_phases(commits, &mut phases);

    assert_eq!(phases[0].git_commits.len(), 1);
}

#[test]
fn test_git_commit_construction() {
    let commit = GitCommit {
        hash: "a310c04".to_string(),
        timestamp: "2025-11-02T17:50:00Z".to_string(),
        message: "fix(lib): include test_helpers module".to_string(),
        author: "Emily Madum".to_string(),
        files_changed: 4,
        insertions: 21,
        deletions: 15,
    };

    assert_eq!(commit.hash, "a310c04");
    assert_eq!(commit.hash.len(), 7);
    assert_eq!(commit.files_changed, 4);
    assert_eq!(commit.insertions, 21);
    assert_eq!(commit.deletions, 15);
}

#[test]
fn test_git_commit_serialization() {
    let commit = GitCommit {
        hash: "abc1234".to_string(),
        timestamp: "2025-01-01T10:00:00Z".to_string(),
        message: "test commit".to_string(),
        author: "Test Author".to_string(),
        files_changed: 2,
        insertions: 10,
        deletions: 5,
    };

    let json = serde_json::to_string(&commit).unwrap();
    let deserialized: GitCommit = serde_json::from_str(&json).unwrap();

    assert_eq!(commit, deserialized);
}

#[test]
fn test_git_commit_default_values() {
    let commit = GitCommit {
        hash: String::new(),
        timestamp: String::new(),
        message: String::new(),
        author: String::new(),
        files_changed: 0,
        insertions: 0,
        deletions: 0,
    };

    assert_eq!(commit.files_changed, 0);
    assert_eq!(commit.insertions, 0);
    assert_eq!(commit.deletions, 0);
}
