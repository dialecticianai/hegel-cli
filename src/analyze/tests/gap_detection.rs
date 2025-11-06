use crate::analyze::gap_detection::ensure_cowboy_coverage;
use crate::storage::archive::{read_archives, write_archive};
use crate::test_helpers::{test_archive, test_git_commit, test_storage};

#[test]
fn test_gap_with_git_activity() {
    let (_tmp, storage) = test_storage();
    let state_dir = storage.state_dir();

    // Create two workflows with a gap
    let w1 = test_archive("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z");
    let w2 = test_archive("2025-01-01T12:00:00Z", "2025-01-01T12:30:00Z");
    write_archive(&w1, state_dir).unwrap();
    write_archive(&w2, state_dir).unwrap();

    // Git commit in the gap
    let commit = test_git_commit("2025-01-01T11:00:00Z");

    // Read archives and call ensure_cowboy_coverage
    let archives = read_archives(state_dir).unwrap();
    let (created, removed) =
        ensure_cowboy_coverage(state_dir, &archives, Some(&[commit]), false).unwrap();

    // Verify cowboy created
    assert_eq!(created, 1, "Expected 1 cowboy created");
    assert_eq!(removed, 0, "Expected 0 cowboys removed");

    // Verify cowboy spans the gap
    let archives_after = read_archives(state_dir).unwrap();
    let cowboys: Vec<_> = archives_after
        .iter()
        .filter(|a| a.is_synthetic && a.mode == "cowboy")
        .collect();
    assert_eq!(cowboys.len(), 1, "Expected exactly 1 cowboy archive");

    let cowboy = cowboys[0];
    // Parse timestamps to handle different RFC3339 formats (Z vs +00:00)
    let cowboy_start = chrono::DateTime::parse_from_rfc3339(&cowboy.workflow_id)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let cowboy_end = chrono::DateTime::parse_from_rfc3339(&cowboy.completed_at)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let expected_start = chrono::DateTime::parse_from_rfc3339("2025-01-01T10:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let expected_end = chrono::DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    assert_eq!(
        cowboy_start, expected_start,
        "Cowboy should start at gap start"
    );
    assert_eq!(cowboy_end, expected_end, "Cowboy should end at gap end");
}

#[test]
fn test_gap_without_git_activity() {
    let (_tmp, storage) = test_storage();
    let state_dir = storage.state_dir();

    // Create two workflows with a gap
    let w1 = test_archive("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z");
    let w2 = test_archive("2025-01-01T12:00:00Z", "2025-01-01T12:30:00Z");
    write_archive(&w1, state_dir).unwrap();
    write_archive(&w2, state_dir).unwrap();

    // No git commits
    let archives = read_archives(state_dir).unwrap();
    let (created, removed) =
        ensure_cowboy_coverage(state_dir, &archives, Some(&[]), false).unwrap();

    // Verify no cowboy created
    assert_eq!(created, 0, "Expected 0 cowboys created");
    assert_eq!(removed, 0, "Expected 0 cowboys removed");

    // Verify no cowboys exist
    let archives_after = read_archives(state_dir).unwrap();
    let cowboys: Vec<_> = archives_after
        .iter()
        .filter(|a| a.is_synthetic && a.mode == "cowboy")
        .collect();
    assert_eq!(cowboys.len(), 0, "Expected no cowboy archives");
}

#[test]
fn test_correct_cowboy_preserved() {
    let (_tmp, storage) = test_storage();
    let state_dir = storage.state_dir();

    // Create two workflows with a gap
    let w1 = test_archive("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z");
    let w2 = test_archive("2025-01-01T12:00:00Z", "2025-01-01T12:30:00Z");
    write_archive(&w1, state_dir).unwrap();
    write_archive(&w2, state_dir).unwrap();

    // Create a correctly-spanning cowboy
    use crate::test_helpers::ArchiveBuilder;
    let cowboy = ArchiveBuilder::new("2025-01-01T10:30:00Z", "2025-01-01T12:00:00Z")
        .mode("cowboy")
        .synthetic(true)
        .build();
    write_archive(&cowboy, state_dir).unwrap();

    // Git commit in the gap
    let commit = test_git_commit("2025-01-01T11:00:00Z");

    // Call ensure_cowboy_coverage
    let archives = read_archives(state_dir).unwrap();
    let (created, removed) =
        ensure_cowboy_coverage(state_dir, &archives, Some(&[commit]), false).unwrap();

    // Verify cowboy preserved (nothing created or removed)
    assert_eq!(created, 0, "Expected 0 cowboys created");
    assert_eq!(removed, 0, "Expected 0 cowboys removed");

    // Verify the original cowboy still exists
    let archives_after = read_archives(state_dir).unwrap();
    let cowboys: Vec<_> = archives_after
        .iter()
        .filter(|a| a.is_synthetic && a.mode == "cowboy")
        .collect();
    assert_eq!(cowboys.len(), 1, "Expected exactly 1 cowboy archive");
}

#[test]
fn test_wrong_timestamp_cowboy_replaced() {
    let (_tmp, storage) = test_storage();
    let state_dir = storage.state_dir();

    // Create two workflows with a gap (10:30 to 12:00)
    let w1 = test_archive("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z");
    let w2 = test_archive("2025-01-01T12:00:00Z", "2025-01-01T12:30:00Z");
    write_archive(&w1, state_dir).unwrap();
    write_archive(&w2, state_dir).unwrap();

    // Create a cowboy with WRONG timestamps (doesn't span the gap correctly)
    use crate::test_helpers::ArchiveBuilder;
    let wrong_cowboy = ArchiveBuilder::new("2025-01-01T10:45:00Z", "2025-01-01T11:45:00Z")
        .mode("cowboy")
        .synthetic(true)
        .build();
    write_archive(&wrong_cowboy, state_dir).unwrap();

    // Git commit in the gap
    let commit = test_git_commit("2025-01-01T11:00:00Z");

    // Call ensure_cowboy_coverage
    let archives = read_archives(state_dir).unwrap();
    let (created, removed) =
        ensure_cowboy_coverage(state_dir, &archives, Some(&[commit]), false).unwrap();

    // Verify old cowboy removed and new one created
    assert_eq!(created, 1, "Expected 1 cowboy created");
    assert_eq!(removed, 1, "Expected 1 cowboy removed");

    // Verify the new cowboy has correct timestamps
    let archives_after = read_archives(state_dir).unwrap();
    let cowboys: Vec<_> = archives_after
        .iter()
        .filter(|a| a.is_synthetic && a.mode == "cowboy")
        .collect();
    assert_eq!(cowboys.len(), 1, "Expected exactly 1 cowboy archive");

    let cowboy = cowboys[0];
    let cowboy_start = chrono::DateTime::parse_from_rfc3339(&cowboy.workflow_id)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let cowboy_end = chrono::DateTime::parse_from_rfc3339(&cowboy.completed_at)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let expected_start = chrono::DateTime::parse_from_rfc3339("2025-01-01T10:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let expected_end = chrono::DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    assert_eq!(
        cowboy_start, expected_start,
        "New cowboy should start at gap start"
    );
    assert_eq!(cowboy_end, expected_end, "New cowboy should end at gap end");
}

#[test]
fn test_multiple_cowboys_one_correct() {
    let (_tmp, storage) = test_storage();
    let state_dir = storage.state_dir();

    // Create two workflows with a gap (10:30 to 12:00)
    let w1 = test_archive("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z");
    let w2 = test_archive("2025-01-01T12:00:00Z", "2025-01-01T12:30:00Z");
    write_archive(&w1, state_dir).unwrap();
    write_archive(&w2, state_dir).unwrap();

    // Create multiple cowboys: one correct, two incorrect
    use crate::test_helpers::ArchiveBuilder;
    let correct_cowboy = ArchiveBuilder::new("2025-01-01T10:30:00Z", "2025-01-01T12:00:00Z")
        .mode("cowboy")
        .synthetic(true)
        .build();
    let wrong_cowboy1 = ArchiveBuilder::new("2025-01-01T10:45:00Z", "2025-01-01T11:45:00Z")
        .mode("cowboy")
        .synthetic(true)
        .build();
    let wrong_cowboy2 = ArchiveBuilder::new("2025-01-01T11:00:00Z", "2025-01-01T11:30:00Z")
        .mode("cowboy")
        .synthetic(true)
        .build();

    write_archive(&correct_cowboy, state_dir).unwrap();
    write_archive(&wrong_cowboy1, state_dir).unwrap();
    write_archive(&wrong_cowboy2, state_dir).unwrap();

    // Git commit in the gap
    let commit = test_git_commit("2025-01-01T11:00:00Z");

    // Call ensure_cowboy_coverage
    let archives = read_archives(state_dir).unwrap();
    let (created, removed) =
        ensure_cowboy_coverage(state_dir, &archives, Some(&[commit]), false).unwrap();

    // Verify correct cowboy preserved, incorrect ones removed
    assert_eq!(created, 0, "Expected 0 cowboys created");
    assert_eq!(removed, 2, "Expected 2 cowboys removed");

    // Verify only the correct cowboy remains
    let archives_after = read_archives(state_dir).unwrap();
    let cowboys: Vec<_> = archives_after
        .iter()
        .filter(|a| a.is_synthetic && a.mode == "cowboy")
        .collect();
    assert_eq!(cowboys.len(), 1, "Expected exactly 1 cowboy archive");

    let cowboy = cowboys[0];
    let cowboy_start = chrono::DateTime::parse_from_rfc3339(&cowboy.workflow_id)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let cowboy_end = chrono::DateTime::parse_from_rfc3339(&cowboy.completed_at)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let expected_start = chrono::DateTime::parse_from_rfc3339("2025-01-01T10:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let expected_end = chrono::DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    assert_eq!(cowboy_start, expected_start, "Correct cowboy should remain");
    assert_eq!(
        cowboy_end, expected_end,
        "Correct cowboy should span full gap"
    );
}

#[test]
fn test_spurious_cowboy_removed() {
    let (_tmp, storage) = test_storage();
    let state_dir = storage.state_dir();

    // Create two workflows with a gap
    let w1 = test_archive("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z");
    let w2 = test_archive("2025-01-01T12:00:00Z", "2025-01-01T12:30:00Z");
    write_archive(&w1, state_dir).unwrap();
    write_archive(&w2, state_dir).unwrap();

    // Create a cowboy in the gap (but no git activity)
    use crate::test_helpers::ArchiveBuilder;
    let spurious_cowboy = ArchiveBuilder::new("2025-01-01T10:30:00Z", "2025-01-01T12:00:00Z")
        .mode("cowboy")
        .synthetic(true)
        .build();
    write_archive(&spurious_cowboy, state_dir).unwrap();

    // NO git commits (empty list)
    let archives = read_archives(state_dir).unwrap();
    let (created, removed) =
        ensure_cowboy_coverage(state_dir, &archives, Some(&[]), false).unwrap();

    // Verify cowboy removed
    assert_eq!(created, 0, "Expected 0 cowboys created");
    assert_eq!(removed, 1, "Expected 1 cowboy removed");

    // Verify no cowboys remain
    let archives_after = read_archives(state_dir).unwrap();
    let cowboys: Vec<_> = archives_after
        .iter()
        .filter(|a| a.is_synthetic && a.mode == "cowboy")
        .collect();
    assert_eq!(cowboys.len(), 0, "Expected no cowboy archives");
}

#[test]
fn test_multiple_gaps_independent() {
    let (_tmp, storage) = test_storage();
    let state_dir = storage.state_dir();

    // Create three workflows creating two gaps
    // Gap 1: 10:30 to 12:00 (has activity)
    // Gap 2: 12:30 to 14:00 (no activity)
    let w1 = test_archive("2025-01-01T10:00:00Z", "2025-01-01T10:30:00Z");
    let w2 = test_archive("2025-01-01T12:00:00Z", "2025-01-01T12:30:00Z");
    let w3 = test_archive("2025-01-01T14:00:00Z", "2025-01-01T14:30:00Z");
    write_archive(&w1, state_dir).unwrap();
    write_archive(&w2, state_dir).unwrap();
    write_archive(&w3, state_dir).unwrap();

    // Git commit only in first gap
    let commit = test_git_commit("2025-01-01T11:00:00Z");

    // Call ensure_cowboy_coverage
    let archives = read_archives(state_dir).unwrap();
    let (created, removed) =
        ensure_cowboy_coverage(state_dir, &archives, Some(&[commit]), false).unwrap();

    // Verify cowboy created only for first gap
    assert_eq!(created, 1, "Expected 1 cowboy created");
    assert_eq!(removed, 0, "Expected 0 cowboys removed");

    // Verify exactly one cowboy exists, spanning first gap
    let archives_after = read_archives(state_dir).unwrap();
    let cowboys: Vec<_> = archives_after
        .iter()
        .filter(|a| a.is_synthetic && a.mode == "cowboy")
        .collect();
    assert_eq!(cowboys.len(), 1, "Expected exactly 1 cowboy archive");

    let cowboy = cowboys[0];
    let cowboy_start = chrono::DateTime::parse_from_rfc3339(&cowboy.workflow_id)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let cowboy_end = chrono::DateTime::parse_from_rfc3339(&cowboy.completed_at)
        .unwrap()
        .with_timezone(&chrono::Utc);
    let expected_start = chrono::DateTime::parse_from_rfc3339("2025-01-01T10:30:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let expected_end = chrono::DateTime::parse_from_rfc3339("2025-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    assert_eq!(cowboy_start, expected_start, "Cowboy should span first gap");
    assert_eq!(cowboy_end, expected_end, "Cowboy should span first gap");
}
