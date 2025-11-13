# DDD Artifact Management Implementation Plan

TDD-focused plan for implementing DDD artifact formalization and management.

---

## Overview

**Goal:** Implement structured management of DDD artifacts with validation, improved visualization, programmatic creation, and automated repair.

**Scope:** Four integrated components:
1. Core types and validation (`src/ddd.rs`)
2. Enhanced markdown display with validation warnings (modify `src/commands/markdown.rs`)
3. `hegel new` command for artifact creation (`src/commands/new.rs`)
4. `hegel doctor` command for automated repair (`src/commands/doctor.rs`)

**Priorities:**
1. Reuse existing patterns from codebase (WalkBuilder, tree rendering, Command pattern)
2. TDD discipline where tests drive development
3. Test organization matching TESTING.md patterns (inline until >200 lines)
4. Agent-friendly output (clear messages, non-interactive)

**Methodology:**
- TDD discipline: write tests first for core validation and scanning logic
- Inline tests in `src/ddd.rs` initially (file will be <200 lines)
- Reuse `scan_markdown_files()` and `classify_file()` patterns from markdown.rs
- Use `Command::new("git")` pattern from metrics/tests/git.rs for git operations
- Extend `TreeNode` structure to support feat metadata (SPEC/PLAN indicators)
- Skip exhaustive edge cases, focus on essential behavior

---

## Existing Patterns to Reuse

**From src/commands/markdown.rs:**
- `scan_markdown_files()` - WalkBuilder with gitignore support
- `classify_file()` - DDD artifact detection (.ddd/, toys/, HANDOFF.md)
- `TreeNode` structure - hierarchical file tree representation
- `render_tree_child()` - tree drawing with proper connectors
- `Theme` usage - colored output for warnings and metrics

**From src/metrics/tests/git.rs:**
- `Command::new("git")` pattern for git operations
- `.args()`, `.current_dir()`, `.output()` chain
- Error handling for Command output

**From src/commands/config.rs:**
- Command handler pattern: `pub fn handle_*(args, storage) -> Result<()>`
- Simple match-based action routing
- Clear error messages with anyhow::bail!

**Command registration pattern (src/main.rs):**
- Add to Commands enum
- Route in match statement
- Pass storage reference

---

## Step 1: Core DDD Module and Types

### Goal
Establish foundational types and path derivation logic

### Step 1.a: Write Tests for Types
Test path derivation and type construction:
- FeatArtifact: directory name generation with and without index
- RefactorArtifact: file name generation with proper extension
- ReportArtifact: file name generation with proper extension
- Verify paths computed correctly from components

### Step 1.b: Implement Core Types
Create `src/ddd.rs` with:
- FeatArtifact struct with date, index, name, spec_exists, plan_exists fields
- RefactorArtifact struct with date, name fields
- ReportArtifact struct with date, name fields
- DddArtifact enum wrapping all three types
- Path derivation methods: dir_name(), dir_path(), file_name(), file_path()
- ValidationIssue and IssueType types
- DddScanResult type

### Success Criteria
- Types compile cleanly
- Path derivation tests pass
- All fields accessible
- Methods return correct PathBuf values

---

## Step 2: Name Parsing and Validation

### Goal
Parse artifact names and validate naming conventions

### Step 2.a: Write Parsing Tests
Test parsing logic for all formats:
- Valid feat with index: "20251104-1-non_phase_commits"
- Valid feat without index: "20251104-my-feature"
- Valid refactor/report: "20251104-large_files"
- Invalid: missing date, wrong date format, trailing dashes
- Edge case: detect when indices are needed

### Step 2.b: Implement Parsing
Add parsing functions to `src/ddd.rs`:
- parse_feat_name() extracts date, optional index, name
- parse_single_file_name() extracts date, name from .md files
- validate_date_format() checks YYYYMMDD format
- validate_name_format() checks lowercase-with-hyphens
- Return Result with descriptive errors

### Success Criteria
- Parser correctly extracts components
- Invalid formats return Err with clear messages
- All parsing tests pass

---

## Step 3: Artifact Scanning

### Goal
Scan .ddd/ directories and build artifact tree, reusing existing markdown.rs patterns

### Step 3.a: Write Scanning Tests
Test directory scanning:
- Empty .ddd/ returns empty results
- Mixed valid and invalid artifacts detected
- Feat artifacts check SPEC.md and PLAN.md existence
- Validation issues collected for malformed artifacts
- Respects directory structure (feat/, refactor/, report/)

### Step 3.b: Implement Scanning
Add scan_ddd_artifacts() function to `src/ddd.rs`:
- Filter existing MarkdownFile results from markdown.rs scan for .ddd/ paths only
- For feat directories: scan children for SPEC.md and PLAN.md files
- Parse directory/file names using parsing functions from Step 2
- Collect validation issues for malformed names
- Return DddScanResult with artifacts and issues

**Pattern:** Similar to categorize_files() in markdown.rs but with parsing and validation

### Success Criteria
- Scanning tests pass
- Reuses markdown.rs WalkBuilder pattern (no duplicate directory walking)
- Valid artifacts parsed correctly
- Invalid artifacts generate appropriate validation issues
- SPEC/PLAN existence correctly tracked

**Commit Point:** Core ddd module with types, parsing, and scanning

---

## Step 4: Enhanced Markdown Display

### Goal
Update hegel md --ddd to show consolidated format with warnings, extending TreeNode

### Step 4.a: Write Display Integration Tests
Test markdown command output:
- Feat artifacts show "SPEC ✓ PLAN ✓" format
- Malformed artifacts show ⚠️ indicator
- Footer message appears when issues exist
- JSON mode still works correctly
- Non-DDD sections unchanged

### Step 4.b: Integrate with Markdown Command
Modify `src/commands/markdown.rs`:
- Extend TreeNode to include optional metadata fields: spec_exists, plan_exists, is_malformed
- When building tree for DDD files, call scan_ddd_artifacts() and attach metadata to TreeNodes
- Update render_tree_child() to check if node is feat directory and render SPEC/PLAN indicators
- Add ⚠️ suffix for nodes with is_malformed = true
- Show footer warning if any ValidationIssue found
- Preserve existing --json, --no-ddd, --ddd flags behavior
- Update JSON output schema to include validation_issues field

**Pattern:** Extend existing tree rendering, don't rewrite it

### Success Criteria
- Display tests pass
- hegel md --ddd shows new format for feat directories
- Single files (refactor/report) still show line counts
- Warnings appear for malformed artifacts
- JSON output includes validation_issues array
- No regressions in regular markdown or --no-ddd modes

---

## Step 5: New Artifact Creation Command

### Goal
Implement hegel new command for creating artifacts

### Step 5.a: Write Creation Tests
Test artifact creation:
- hegel new feat creates directory with correct date
- Multiple same-day feats get auto-indexed
- Output message guides agent to correct path
- Refactor/report output path without creating file
- Error on duplicate names
- Error on invalid names

### Step 5.b: Implement New Command
Add to `src/main.rs` Commands enum:
- New subcommand with [feat|refactor|report] and name argument
Add `src/commands/new.rs`:
- run_new() function routes to artifact-specific handlers
- create_feat() creates directory, determines index, outputs message
- create_refactor() and create_report() output paths only
- Validation for name format and duplicates
- Agent-friendly stdout messages

Update `src/commands/mod.rs` to export new command

### Success Criteria
- Creation tests pass
- hegel new feat creates directory
- Auto-indexing works for same-day artifacts
- Refactor/report output paths without creating files
- Output messages guide agent correctly

**Commit Point:** New command implementation

---

## Step 6: Doctor Command - Detection

### Goal
Implement hegel doctor dry-run mode for detecting issues, using git Command pattern

### Step 6.a: Write Detection Tests
Test issue detection:
- Detects missing date prefixes
- Detects missing indices for same-day feats
- Uses git log to determine artifact addition dates
- Displays planned fixes clearly
- Handles untracked files gracefully (skips with warning)

### Step 6.b: Implement Doctor Detection
Add to `src/main.rs` Commands enum:
- Doctor subcommand with optional --apply flag
Add `src/commands/doctor.rs`:
- run_doctor() function routes to detection or apply logic
- detect_issues() calls scan_ddd_artifacts() for malformed list
- lookup_git_date() uses Command::new("git").args(&["log", "--follow", "--format=%ad", "--date=short", path]) pattern
- Parse output, take last line (tail -1 equivalent)
- suggest_fixes() generates new names with dates from git
- format_dry_run_output() displays planned fixes with old → new format

**Pattern:** Similar to metrics/tests/git.rs Command usage

Update `src/commands/mod.rs` to export doctor command

### Success Criteria
- Detection tests pass
- hegel doctor shows planned fixes in "old → new" format
- Git dates correctly determined using Command pattern
- Untracked files warned and skipped
- Clear, actionable output with counts

---

## Step 7: Doctor Command - Repair

### Goal
Implement hegel doctor --apply mode for fixing issues with git mv

### Step 7.a: Write Repair Tests
Test fix application:
- Renames artifacts with correct dates
- Adds indices to resolve conflicts
- Uses git mv for tracked files (check with git ls-files first)
- Uses std::fs::rename for untracked files
- Displays results clearly with success indicators

### Step 7.b: Implement Doctor Repair
Extend `src/commands/doctor.rs`:
- apply_fixes() executes planned fixes from Step 6
- is_tracked() helper uses Command::new("git").args(&["ls-files", path]) to check tracking status
- rename_artifact() uses Command::new("git").args(&["mv", old, new]) for tracked files
- Falls back to std::fs::rename() for untracked
- add_indices() resolves same-day conflicts by detecting and numbering
- format_apply_output() displays results with ✓ indicators

**Pattern:** Check tracking with git ls-files (like archive-oneoff.pl does)

### Success Criteria
- Repair tests pass
- hegel doctor --apply renames artifacts correctly
- Git history preserved for tracked files
- Untracked files moved with std::fs::rename
- Indices added to resolve conflicts
- Success/failure clearly reported with counts

**Commit Point:** Doctor command implementation

---

## Step 8: Integration and Final Validation

### Goal
Verify all components work together end-to-end

### Step 8.a: Write Integration Tests
Test complete workflows:
- Create artifact with new, display with md, repair with doctor
- Mixed valid/invalid artifacts handled correctly
- All output messages agent-friendly
- Error cases handled gracefully

### Step 8.b: Final Integration
- Verify all commands registered in main.rs
- Test error paths and edge cases
- Ensure consistent error messages
- Verify no regressions in existing commands

### Success Criteria
- All integration tests pass
- Full test suite passes (cargo test)
- Build succeeds with no warnings
- Commands work correctly in realistic scenarios
- Agent-friendly output verified

**Commit Point:** Integration tests and final polish

---

## Out of Scope

**Deferred to future work:**
- SPEC.md/PLAN.md stub content generation
- Automatic migration scripts for legacy artifacts
- TUI interface for doctor
- Validation of document content structure
- Integration with workflow state machine
