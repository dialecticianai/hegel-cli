mod aggregation;
mod builder;
mod validation;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::metrics::git::GitCommit;

// Reverse-aggregation helpers (archive summaries -> individual hook records).
pub use aggregation::{expand_bash_commands, expand_file_modifications};

/// Archived workflow with pre-computed aggregates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowArchive {
    pub workflow_id: String,
    pub mode: String,
    pub completed_at: String,
    pub phases: Vec<PhaseArchive>,
    pub transitions: Vec<TransitionArchive>,
    pub totals: WorkflowTotals,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Whether this archive was auto-detected from inter-workflow activity
    #[serde(default)]
    pub is_synthetic: bool,
}

/// Per-phase metrics in archive
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseArchive {
    pub phase_name: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: u64,
    pub tokens: TokenTotals,
    pub bash_commands: Vec<BashCommandSummary>,
    pub file_modifications: Vec<FileModificationSummary>,
    #[serde(default)]
    pub git_commits: Vec<GitCommit>,
}

/// State transition in archive
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransitionArchive {
    pub from_node: String,
    pub to_node: String,
    pub timestamp: String,
}

/// Token usage totals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TokenTotals {
    pub input: u64,
    pub output: u64,
    pub cache_creation: u64,
    pub cache_read: u64,
    pub assistant_turns: usize,
}

/// Bash command summary (command + frequency)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BashCommandSummary {
    pub command: String,
    pub count: usize,
    pub timestamps: Vec<String>,
}

/// File modification summary (file + tool + frequency)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileModificationSummary {
    pub file_path: String,
    pub tool: String,
    pub count: usize,
    pub timestamps: Vec<String>,
}

/// Workflow-level totals
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WorkflowTotals {
    pub tokens: TokenTotals,
    pub bash_commands: usize,
    pub file_modifications: usize,
    pub unique_files: usize,
    pub unique_commands: usize,
    #[serde(default)]
    pub git_commits: usize,
}

impl std::ops::AddAssign<&TokenTotals> for TokenTotals {
    fn add_assign(&mut self, other: &TokenTotals) {
        self.input += other.input;
        self.output += other.output;
        self.cache_creation += other.cache_creation;
        self.cache_read += other.cache_read;
        self.assistant_turns += other.assistant_turns;
    }
}

impl std::ops::AddAssign<&WorkflowTotals> for WorkflowTotals {
    fn add_assign(&mut self, other: &WorkflowTotals) {
        self.tokens += &other.tokens;
        self.bash_commands += other.bash_commands;
        self.file_modifications += other.file_modifications;
        self.unique_files += other.unique_files;
        self.unique_commands += other.unique_commands;
        self.git_commits += other.git_commits;
    }
}

impl From<&TokenTotals> for crate::metrics::TokenMetrics {
    fn from(t: &TokenTotals) -> Self {
        crate::metrics::TokenMetrics {
            total_input_tokens: t.input,
            total_output_tokens: t.output,
            total_cache_creation_tokens: t.cache_creation,
            total_cache_read_tokens: t.cache_read,
            assistant_turns: t.assistant_turns,
        }
    }
}

/// Write archive to disk with atomic operation
pub fn write_archive(archive: &WorkflowArchive, state_dir: &Path) -> Result<()> {
    let archive_dir = state_dir.join("archive");
    fs::create_dir_all(&archive_dir)
        .with_context(|| format!("Failed to create archive directory: {:?}", archive_dir))?;

    let archive_path = archive_dir.join(format!("{}.json", archive.workflow_id));

    // Check for existing archive
    if archive_path.exists() {
        bail!("Archive already exists: {}", archive.workflow_id);
    }

    crate::storage::atomic_write_json(&archive_path, archive)
}

/// Read all archives from archive directory
pub fn read_archives(state_dir: &Path) -> Result<Vec<WorkflowArchive>> {
    let archive_dir = state_dir.join("archive");

    // Return empty if archive directory doesn't exist
    if !archive_dir.exists() {
        return Ok(Vec::new());
    }

    let mut archives = Vec::new();

    for entry in fs::read_dir(&archive_dir)
        .with_context(|| format!("Failed to read archive directory: {:?}", archive_dir))?
    {
        let entry = entry?;
        let path = entry.path();

        // Only process .json files
        if path.extension().is_some_and(|e| e == "json") {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<WorkflowArchive>(&content) {
                    Ok(archive) => archives.push(archive),
                    Err(e) => {
                        eprintln!("Warning: skipping corrupted archive {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    eprintln!("Warning: failed to read archive {:?}: {}", path, e);
                }
            }
        }
    }

    // Sort archives by workflow_id (chronological order)
    archives.sort_by(|a, b| a.workflow_id.cmp(&b.workflow_id));

    Ok(archives)
}

/// Whether a phase name is a terminal node (kept even when empty).
fn is_terminal_phase(name: &str) -> bool {
    name == "done" || name == "aborted"
}

/// Whether a phase recorded any activity (tokens, commands, files, or commits).
fn phase_has_activity(p: &PhaseArchive) -> bool {
    p.tokens.input > 0
        || p.tokens.output > 0
        || p.tokens.cache_creation > 0
        || p.tokens.cache_read > 0
        || !p.bash_commands.is_empty()
        || !p.file_modifications.is_empty()
        || !p.git_commits.is_empty()
}

/// Outcome of [`WorkflowArchive::compact_phases`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PhaseCompaction {
    /// Duplicate phase records removed (same name + start_time).
    pub deduped: usize,
    /// Non-terminal, activity-free phases pruned.
    pub pruned: usize,
}

impl PhaseCompaction {
    /// Whether anything was removed.
    pub fn changed(&self) -> bool {
        self.deduped > 0 || self.pruned > 0
    }
}

impl WorkflowArchive {
    /// Compact phases in place: drop exact duplicates (same `phase_name` +
    /// `start_time`, keeping the highest-token copy) and prune non-terminal
    /// phases that recorded no activity at all. Terminal (`done`/`aborted`)
    /// phases are always kept. Token and git-commit totals are recomputed from
    /// the surviving phases (other session-derived totals are left untouched).
    pub fn compact_phases(&mut self) -> PhaseCompaction {
        use std::collections::HashMap;

        // 1. Dedup by (phase_name, start_time), keeping the max-token copy.
        let original = self.phases.len();
        let mut index: HashMap<(String, String), usize> = HashMap::new();
        let mut kept: Vec<PhaseArchive> = Vec::with_capacity(original);
        for phase in std::mem::take(&mut self.phases) {
            let key = (phase.phase_name.clone(), phase.start_time.clone());
            if let Some(&i) = index.get(&key) {
                let existing = &kept[i];
                if phase.tokens.input + phase.tokens.output
                    > existing.tokens.input + existing.tokens.output
                {
                    kept[i] = phase;
                }
            } else {
                index.insert(key, kept.len());
                kept.push(phase);
            }
        }
        let deduped = original - kept.len();

        // 2. Prune non-terminal phases with no activity at all.
        let before_prune = kept.len();
        kept.retain(|p| is_terminal_phase(&p.phase_name) || phase_has_activity(p));
        let pruned = before_prune - kept.len();

        self.phases = kept;

        // 3. Recompute phase-derived totals (tokens + git-commit count).
        let mut tokens = TokenTotals::default();
        for p in &self.phases {
            tokens += &p.tokens;
        }
        self.totals.tokens = tokens;
        self.totals.git_commits = self.phases.iter().map(|p| p.git_commits.len()).sum();

        PhaseCompaction { deduped, pruned }
    }
}

/// Outcome of [`dedup_across_archives`].
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CrossArchiveDedup {
    /// Phase records removed because the same `(phase_name, start_time)` lived
    /// in another archive.
    pub phases_removed: usize,
    /// Transition records removed because the same `(from, to, timestamp)`
    /// lived in another archive.
    pub transitions_removed: usize,
    /// `workflow_id`s whose phases or transitions were modified.
    pub affected: Vec<String>,
}

/// Remove phases and transitions that are duplicated *across* archives.
///
/// The pre-Nov-3 re-archiving bug wrote nested cumulative snapshots: each
/// re-archive captured every prior phase/transition plus the new ones, so the
/// same record ends up in many archive files (inflating phase lists, token
/// totals, and the workflow graph). This keeps a single canonical copy of each
/// record and strips the rest:
/// - phases: canonical = the highest-token copy (ties → earliest archive)
/// - transitions: canonical = the earliest archive containing it
///
/// `archives` is assumed sorted by `workflow_id` (as [`read_archives`] returns
/// it), so "earliest" means lowest index. Per-archive token/git totals are
/// **not** recomputed here — call [`WorkflowArchive::compact_phases`] on each
/// archive afterward to rebuild those from the survivors.
pub fn dedup_across_archives(archives: &mut [WorkflowArchive]) -> CrossArchiveDedup {
    use std::collections::{HashMap, HashSet};

    // Pass 1 (immutable): pick the canonical archive index for each phase key.
    let mut phase_owner: HashMap<(String, String), (usize, u64)> = HashMap::new();
    for (idx, archive) in archives.iter().enumerate() {
        for phase in &archive.phases {
            let key = (phase.phase_name.clone(), phase.start_time.clone());
            let tokens = phase.tokens.input + phase.tokens.output;
            phase_owner
                .entry(key)
                .and_modify(|(best_idx, best_tokens)| {
                    // Strictly greater so the earliest archive wins ties.
                    if tokens > *best_tokens {
                        *best_idx = idx;
                        *best_tokens = tokens;
                    }
                })
                .or_insert((idx, tokens));
        }
    }

    // Pass 2 (mutable): retain only canonically-owned phases; dedup transitions
    // by keeping the first archive (lowest index) that contains each.
    let mut seen_transitions: HashSet<(String, String, String)> = HashSet::new();
    let mut result = CrossArchiveDedup::default();
    for (idx, archive) in archives.iter_mut().enumerate() {
        let phases_before = archive.phases.len();
        archive.phases.retain(|p| {
            let key = (p.phase_name.clone(), p.start_time.clone());
            phase_owner.get(&key).map(|&(owner, _)| owner) == Some(idx)
        });
        let phases_dropped = phases_before - archive.phases.len();

        let transitions_before = archive.transitions.len();
        archive.transitions.retain(|t| {
            let key = (t.from_node.clone(), t.to_node.clone(), t.timestamp.clone());
            seen_transitions.insert(key)
        });
        let transitions_dropped = transitions_before - archive.transitions.len();

        if phases_dropped > 0 || transitions_dropped > 0 {
            result.phases_removed += phases_dropped;
            result.transitions_removed += transitions_dropped;
            result.affected.push(archive.workflow_id.clone());
        }
    }

    result
}

/// Parse an RFC3339 timestamp to UTC, or `None` if it doesn't parse.
fn parse_utc(ts: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(ts)
        .ok()
        .map(|d| d.with_timezone(&Utc))
}

/// Latest captured-activity timestamp across the given phases (bash commands,
/// file modifications, and git commits). `None` if no activity has a timestamp.
fn phases_last_activity(phases: &[PhaseArchive]) -> Option<DateTime<Utc>> {
    phases
        .iter()
        .flat_map(|p| {
            let bash = p.bash_commands.iter().flat_map(|b| b.timestamps.iter());
            let files = p
                .file_modifications
                .iter()
                .flat_map(|f| f.timestamps.iter());
            let commits = p.git_commits.iter().map(|c| &c.timestamp);
            bash.chain(files).chain(commits)
        })
        .filter_map(|ts| parse_utc(ts))
        .max()
}

impl WorkflowArchive {
    /// Trim a synthetic cowboy ride so it ends at its last *captured activity*
    /// (the latest bash/file/commit timestamp) rather than the moment detection
    /// happened to run — which for an overnight gap inflates the duration to
    /// many idle hours. The anchored start (`workflow_id`) is preserved, so the
    /// archive's identity and gap-coverage matching are unchanged.
    ///
    /// No-op (returns `false`) for non-cowboy archives, when there is no
    /// timestamped activity to trim to, or when the ride already ends at/before
    /// its last activity. Transcript-only rides (no bash/file/commit) can't be
    /// trimmed since transcripts aren't stored in the archive.
    pub fn trim_cowboy_to_activity(&mut self) -> bool {
        if !self.is_synthetic || self.mode != "cowboy" {
            return false;
        }
        let Some(last) = phases_last_activity(&self.phases) else {
            return false;
        };
        let Some(start) = parse_utc(&self.workflow_id) else {
            return false;
        };
        if last <= start {
            return false;
        }
        // Already tight enough?
        if parse_utc(&self.completed_at).is_some_and(|end| last >= end) {
            return false;
        }

        let last_rfc = last.to_rfc3339();

        // Drop phases that begin after the last activity (e.g. the trailing
        // zero-width active-phase placeholder).
        self.phases
            .retain(|p| parse_utc(&p.start_time).is_none_or(|s| s < last));

        // Clamp surviving phase ends into the trimmed window and recompute
        // durations.
        for p in &mut self.phases {
            let overruns = match p.end_time.as_deref().and_then(parse_utc) {
                Some(end) => end > last,
                None => true, // active phase: close it at the last activity
            };
            if overruns {
                p.end_time = Some(last_rfc.clone());
            }
            if let Some(s) = parse_utc(&p.start_time) {
                let e = p.end_time.as_deref().and_then(parse_utc).unwrap_or(last);
                p.duration_seconds = (e - s).num_seconds().max(0) as u64;
            }
        }

        // Clamp any transition (e.g. ride->done) that fell at the old end.
        for t in &mut self.transitions {
            if parse_utc(&t.timestamp).is_some_and(|ts| ts > last) {
                t.timestamp = last_rfc.clone();
            }
        }

        self.completed_at = last_rfc;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Build a phase with the given name/start, input tokens, and bash-command count.
    fn phase(name: &str, start: &str, input: u64, bash: usize) -> PhaseArchive {
        PhaseArchive {
            phase_name: name.to_string(),
            start_time: start.to_string(),
            end_time: Some(start.to_string()),
            duration_seconds: 0,
            tokens: TokenTotals {
                input,
                output: 0,
                cache_creation: 0,
                cache_read: 0,
                assistant_turns: if input > 0 { 1 } else { 0 },
            },
            bash_commands: (0..bash)
                .map(|i| BashCommandSummary {
                    command: format!("cmd{i}"),
                    count: 1,
                    timestamps: vec![],
                })
                .collect(),
            file_modifications: vec![],
            git_commits: vec![],
        }
    }

    fn archive_with(phases: Vec<PhaseArchive>) -> WorkflowArchive {
        WorkflowArchive {
            workflow_id: "wf".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-01-01T00:00:00Z".to_string(),
            phases,
            transitions: vec![],
            totals: WorkflowTotals::default(),
            session_id: None,
            is_synthetic: false,
        }
    }

    #[test]
    fn test_compact_dedups_keeping_max_tokens_and_recomputes_totals() {
        let mut a = archive_with(vec![
            phase("plan", "T1", 0, 0),   // duplicate, zero tokens
            phase("plan", "T1", 100, 0), // duplicate, has tokens -> kept
            phase("code", "T2", 50, 0),
        ]);
        let r = a.compact_phases();
        assert_eq!(r.deduped, 1);
        assert_eq!(r.pruned, 0);
        assert_eq!(a.phases.len(), 2);
        let plan = a.phases.iter().find(|p| p.phase_name == "plan").unwrap();
        assert_eq!(plan.tokens.input, 100); // kept the higher-token copy
        assert_eq!(a.totals.tokens.input, 150); // totals recomputed (100 + 50)
    }

    #[test]
    fn test_compact_prunes_empty_nonterminal_keeps_terminal_and_active() {
        let mut a = archive_with(vec![
            phase("spec", "T1", 0, 0), // empty non-terminal -> pruned
            phase("code", "T2", 0, 2), // has bash activity -> kept
            phase("done", "T3", 0, 0), // terminal empty -> kept
        ]);
        let r = a.compact_phases();
        assert_eq!(r.pruned, 1);
        assert_eq!(r.deduped, 0);
        assert_eq!(a.phases.len(), 2);
        assert!(a.phases.iter().any(|p| p.phase_name == "code"));
        assert!(a.phases.iter().any(|p| p.phase_name == "done"));
        assert!(!a.phases.iter().any(|p| p.phase_name == "spec"));
    }

    #[test]
    fn test_compact_is_noop_on_clean_archive() {
        let mut a = archive_with(vec![phase("spec", "T1", 10, 0), phase("done", "T2", 0, 0)]);
        let r = a.compact_phases();
        assert!(!r.changed());
        assert_eq!(a.phases.len(), 2);
    }

    /// Build a named archive with phases + transitions for cross-archive tests.
    fn archive_named(
        id: &str,
        phases: Vec<PhaseArchive>,
        transitions: Vec<TransitionArchive>,
    ) -> WorkflowArchive {
        WorkflowArchive {
            workflow_id: id.to_string(),
            mode: "discovery".to_string(),
            completed_at: id.to_string(),
            phases,
            transitions,
            totals: WorkflowTotals::default(),
            session_id: None,
            is_synthetic: false,
        }
    }

    fn transition(from: &str, to: &str, ts: &str) -> TransitionArchive {
        TransitionArchive {
            from_node: from.to_string(),
            to_node: to.to_string(),
            timestamp: ts.to_string(),
        }
    }

    #[test]
    fn test_dedup_across_archives_keeps_max_token_phase_in_one_archive() {
        // Nested-snapshot shape: "spec@T1" appears in all three archives; the
        // later snapshot carries more tokens, so it owns the canonical copy.
        let mut archives = vec![
            archive_named("A", vec![phase("spec", "T1", 10, 0)], vec![]),
            archive_named(
                "B",
                vec![phase("spec", "T1", 50, 0), phase("code", "T2", 5, 0)],
                vec![],
            ),
            archive_named("C", vec![phase("spec", "T1", 30, 0)], vec![]),
        ];
        let r = dedup_across_archives(&mut archives);

        assert_eq!(r.phases_removed, 2); // spec dropped from A and C
        assert_eq!(r.transitions_removed, 0);
        // "spec" survives only in B (max tokens), "code" only in B.
        assert!(archives[0].phases.is_empty());
        assert_eq!(archives[1].phases.len(), 2);
        assert!(archives[2].phases.is_empty());
        let spec = archives[1]
            .phases
            .iter()
            .find(|p| p.phase_name == "spec")
            .unwrap();
        assert_eq!(spec.tokens.input, 50);
        assert_eq!(r.affected, vec!["A".to_string(), "C".to_string()]);
    }

    #[test]
    fn test_dedup_across_archives_ties_go_to_earliest() {
        // Equal tokens → the earliest archive (lowest index) keeps it.
        let mut archives = vec![
            archive_named("A", vec![phase("spec", "T1", 10, 0)], vec![]),
            archive_named("B", vec![phase("spec", "T1", 10, 0)], vec![]),
        ];
        let r = dedup_across_archives(&mut archives);
        assert_eq!(r.phases_removed, 1);
        assert_eq!(archives[0].phases.len(), 1);
        assert!(archives[1].phases.is_empty());
        assert_eq!(r.affected, vec!["B".to_string()]);
    }

    #[test]
    fn test_dedup_across_archives_dedups_transitions_keeping_earliest() {
        let t = || transition("spec", "code", "T1");
        let mut archives = vec![
            archive_named("A", vec![], vec![t(), transition("code", "done", "T2")]),
            archive_named("B", vec![], vec![t()]), // duplicate of A's first transition
        ];
        let r = dedup_across_archives(&mut archives);
        assert_eq!(r.transitions_removed, 1);
        assert_eq!(archives[0].transitions.len(), 2); // earliest keeps both
        assert!(archives[1].transitions.is_empty());
        assert_eq!(r.affected, vec!["B".to_string()]);
    }

    #[test]
    fn test_dedup_across_archives_noop_on_clean_set() {
        let mut archives = vec![
            archive_named(
                "A",
                vec![phase("spec", "T1", 10, 0)],
                vec![transition("START", "spec", "T1")],
            ),
            archive_named(
                "B",
                vec![phase("code", "T2", 20, 0)],
                vec![transition("spec", "code", "T2")],
            ),
        ];
        let r = dedup_across_archives(&mut archives);
        assert_eq!(r.phases_removed, 0);
        assert_eq!(r.transitions_removed, 0);
        assert!(r.affected.is_empty());
        assert_eq!(archives[0].phases.len(), 1);
        assert_eq!(archives[1].phases.len(), 1);
    }

    /// Build a synthetic cowboy ride spanning [start, end] with optional commit
    /// and bash timestamps, plus START->ride / ride->done transitions.
    fn cowboy_ride(
        start: &str,
        end: &str,
        commit_ts: Option<&str>,
        bash_ts: &[&str],
    ) -> WorkflowArchive {
        let git_commits = commit_ts
            .map(|ts| {
                vec![GitCommit {
                    hash: "abc".into(),
                    author: "a".into(),
                    timestamp: ts.to_string(),
                    message: "m".into(),
                    files_changed: 1,
                    insertions: 1,
                    deletions: 0,
                }]
            })
            .unwrap_or_default();
        let bash_commands = if bash_ts.is_empty() {
            vec![]
        } else {
            vec![BashCommandSummary {
                command: "echo".into(),
                count: bash_ts.len(),
                timestamps: bash_ts.iter().map(|s| s.to_string()).collect(),
            }]
        };
        WorkflowArchive {
            workflow_id: start.to_string(),
            mode: "cowboy".to_string(),
            completed_at: end.to_string(),
            phases: vec![PhaseArchive {
                phase_name: "ride".to_string(),
                start_time: start.to_string(),
                end_time: Some(end.to_string()),
                duration_seconds: 999_999,
                tokens: TokenTotals::default(),
                bash_commands,
                file_modifications: vec![],
                git_commits,
            }],
            transitions: vec![
                transition("START", "ride", start),
                transition("ride", "done", end),
            ],
            totals: WorkflowTotals::default(),
            session_id: None,
            is_synthetic: true,
        }
    }

    #[test]
    fn test_trim_cowboy_ends_at_last_activity() {
        // Overnight gap: detection ran the next morning, but the last activity
        // was a 10:15 commit.
        let mut a = cowboy_ride(
            "2025-01-01T10:00:00Z",
            "2025-01-02T05:00:00Z",
            Some("2025-01-01T10:15:00Z"),
            &["2025-01-01T10:05:00Z", "2025-01-01T10:10:00Z"],
        );
        assert!(a.trim_cowboy_to_activity());

        let expect = parse_utc("2025-01-01T10:15:00Z").unwrap();
        assert_eq!(parse_utc(&a.completed_at), Some(expect));
        assert_eq!(
            parse_utc(a.phases[0].end_time.as_deref().unwrap()),
            Some(expect)
        );
        assert_eq!(a.phases[0].duration_seconds, 900); // 10:00 -> 10:15
                                                       // ride->done transition follows the trimmed end.
        let done = a.transitions.iter().find(|t| t.to_node == "done").unwrap();
        assert_eq!(parse_utc(&done.timestamp), Some(expect));
    }

    #[test]
    fn test_trim_cowboy_drops_trailing_placeholder_phase() {
        let mut a = cowboy_ride(
            "2025-01-01T10:00:00Z",
            "2025-01-01T12:00:00Z",
            Some("2025-01-01T10:30:00Z"),
            &[],
        );
        // Append a zero-width active placeholder phase at the old end.
        a.phases.push(PhaseArchive {
            phase_name: "ride".to_string(),
            start_time: "2025-01-01T12:00:00Z".to_string(),
            end_time: None,
            duration_seconds: 0,
            tokens: TokenTotals::default(),
            bash_commands: vec![],
            file_modifications: vec![],
            git_commits: vec![],
        });

        assert!(a.trim_cowboy_to_activity());
        assert_eq!(a.phases.len(), 1); // trailing placeholder dropped
        let expect = parse_utc("2025-01-01T10:30:00Z").unwrap();
        assert_eq!(parse_utc(&a.completed_at), Some(expect));
        assert_eq!(a.phases[0].duration_seconds, 1800); // 10:00 -> 10:30
    }

    #[test]
    fn test_trim_cowboy_noop_without_timestamped_activity() {
        // No commits and no bash timestamps -> nothing to trim to.
        let mut a = cowboy_ride("2025-01-01T10:00:00Z", "2025-01-02T05:00:00Z", None, &[]);
        assert!(!a.trim_cowboy_to_activity());
        assert_eq!(a.completed_at, "2025-01-02T05:00:00Z");
    }

    #[test]
    fn test_trim_cowboy_noop_for_non_cowboy() {
        let mut a = archive_with(vec![phase("spec", "2025-01-01T10:00:00Z", 10, 0)]);
        // archive_with uses mode "discovery"; trim must not touch it.
        assert!(!a.trim_cowboy_to_activity());
    }

    /// Helper to create test archive with default values
    fn test_archive() -> WorkflowArchive {
        WorkflowArchive {
            workflow_id: "2025-10-24T10:00:00Z".to_string(),
            mode: "discovery".to_string(),
            completed_at: "2025-10-24T12:00:00Z".to_string(),
            session_id: None,
            is_synthetic: false,
            phases: vec![],
            transitions: vec![],
            totals: WorkflowTotals::default(),
        }
    }

    #[test]
    fn test_archive_serialization() {
        let archive = WorkflowArchive {
            session_id: Some("test-session".to_string()),
            phases: vec![PhaseArchive {
                phase_name: "spec".to_string(),
                start_time: "2025-10-24T10:00:00Z".to_string(),
                end_time: Some("2025-10-24T10:15:00Z".to_string()),
                duration_seconds: 900,
                tokens: TokenTotals {
                    input: 1000,
                    output: 500,
                    cache_creation: 200,
                    cache_read: 300,
                    assistant_turns: 5,
                },
                bash_commands: vec![],
                file_modifications: vec![],
                git_commits: vec![],
            }],
            transitions: vec![TransitionArchive {
                from_node: "START".to_string(),
                to_node: "spec".to_string(),
                timestamp: "2025-10-24T10:00:00Z".to_string(),
            }],
            totals: WorkflowTotals {
                tokens: TokenTotals {
                    input: 1000,
                    output: 500,
                    cache_creation: 200,
                    cache_read: 300,
                    assistant_turns: 5,
                },
                bash_commands: 0,
                file_modifications: 0,
                unique_files: 0,
                unique_commands: 0,
                git_commits: 0,
            },
            ..test_archive()
        };

        // Serialize
        let json = serde_json::to_string(&archive).unwrap();

        // Deserialize
        let deserialized: WorkflowArchive = serde_json::from_str(&json).unwrap();

        // Verify round-trip
        assert_eq!(archive, deserialized);
    }

    #[test]
    fn test_write_archive() {
        let temp_dir = TempDir::new().unwrap();
        let archive = test_archive();

        write_archive(&archive, temp_dir.path()).unwrap();

        let archive_path = temp_dir
            .path()
            .join("archive")
            .join("2025-10-24T10:00:00Z.json");
        assert!(archive_path.exists());

        // Verify content
        let content = fs::read_to_string(&archive_path).unwrap();
        let loaded: WorkflowArchive = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded, archive);
    }

    #[test]
    fn test_write_archive_duplicate() {
        let temp_dir = TempDir::new().unwrap();
        let archive = test_archive();

        // First write succeeds
        write_archive(&archive, temp_dir.path()).unwrap();

        // Second write fails
        let result = write_archive(&archive, temp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_read_archives() {
        let temp_dir = TempDir::new().unwrap();

        // Write 2 archives
        let archive1 = test_archive();
        write_archive(&archive1, temp_dir.path()).unwrap();

        let archive2 = WorkflowArchive {
            workflow_id: "2025-10-24T14:00:00Z".to_string(),
            mode: "execution".to_string(),
            completed_at: "2025-10-24T16:00:00Z".to_string(),
            ..test_archive()
        };
        write_archive(&archive2, temp_dir.path()).unwrap();

        // Read all archives
        let archives = read_archives(temp_dir.path()).unwrap();
        assert_eq!(archives.len(), 2);
    }

    #[test]
    fn test_read_archives_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        // No archives - should return empty vec
        let archives = read_archives(temp_dir.path()).unwrap();
        assert!(archives.is_empty());
    }

    #[test]
    fn test_read_archives_skip_corrupted() {
        let temp_dir = TempDir::new().unwrap();

        // Create valid archive
        let archive = test_archive();
        write_archive(&archive, temp_dir.path()).unwrap();

        // Create corrupted archive
        let archive_dir = temp_dir.path().join("archive");
        fs::write(archive_dir.join("corrupted.json"), "not valid json").unwrap();

        // Should read 1 valid archive, skip corrupted
        let archives = read_archives(temp_dir.path()).unwrap();
        assert_eq!(archives.len(), 1);
    }

    #[test]
    fn test_is_synthetic_default_false() {
        // Test backward compatibility: archives without is_synthetic load as is_synthetic=false
        let json = r#"{
            "workflow_id": "2025-10-24T10:00:00Z",
            "mode": "discovery",
            "completed_at": "2025-10-24T12:00:00Z",
            "phases": [],
            "transitions": [],
            "totals": {
                "tokens": {"input": 0, "output": 0, "cache_creation": 0, "cache_read": 0, "assistant_turns": 0},
                "bash_commands": 0,
                "file_modifications": 0,
                "unique_files": 0,
                "unique_commands": 0,
                "git_commits": 0
            }
        }"#;

        let archive: WorkflowArchive = serde_json::from_str(json).unwrap();
        assert_eq!(archive.is_synthetic, false);
    }

    #[test]
    fn test_is_synthetic_serialization() {
        // Test is_synthetic=true serializes and deserializes correctly
        let archive = WorkflowArchive {
            is_synthetic: true,
            ..test_archive()
        };

        let json = serde_json::to_string(&archive).unwrap();
        let deserialized: WorkflowArchive = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.is_synthetic, true);
    }
}
