# Workflow Stash Implementation Plan

## Overview

**Goal**: Implement workflow stash functionality to enable pausing and resuming workflows, similar to git stash.

**Scope**: Four commands (stash, list, pop, drop) with stack-based storage in `.hegel/stashes/` directory. Focus on core functionality with clean error handling.

**Priorities**:
1. Correct state isolation (workflow state only, not session metadata)
2. Automatic index management (0 = most recent)
3. Clear, actionable error messages
4. Expressive test DSL following TESTING.md philosophy

## Methodology

**TDD Approach**: Write tests that express intent clearly, build implementation to satisfy them. Tests serve as executable documentation, not audit requirements.

**Test Philosophy**: Following TESTING.md, create expressive helper functions that read like English. No comments explaining tests - code structure expresses intent.

**What to Test**:
- Core operations: stash creation, listing, pop, drop
- State round-tripping: workflow state preserved exactly
- Error cases: missing workflow, invalid index, active workflow conflicts
- File operations: stash files created/deleted correctly, indices updated

**What NOT to Test**:
- Internal helper functions (private implementation details)
- Hypothetical edge cases (unusual combinations not in SPEC)
- Performance characteristics (not a concern for this feature)

## Step 1: Storage Layer - Stash Management

### Goal
Add methods to FileStorage for managing stash files. Enable creating, reading, listing, and deleting stashes with automatic index management.

### Step 1.a: Write Tests

Create test module at `src/storage/tests/stash.rs` with expressive helpers:
- Helper to create stash entries with message/timestamp
- Helper to assert stash count at specific path
- Helper to read and verify stash contents
- Helper to verify index ordering after operations

Test scenarios to cover:
- Save stash creates file with correct structure and index zero
- Save multiple stashes assigns sequential indices (newest is zero)
- List stashes returns entries sorted newest first
- Delete stash removes file and reindexes remaining stashes
- Load stash by index retrieves correct entry
- Load invalid index returns helpful error

Error cases:
- List when directory doesn't exist returns empty vector
- Load from empty stashes returns appropriate error
- Load invalid index provides error with valid range

### Step 1.b: Implement

Add stash-related methods to FileStorage in `src/storage/mod.rs`:
- Method to save stash: accepts workflow and workflow_state, generates timestamp, creates stash entry, writes to file, reindexes all stashes
- Method to list stashes: reads stash directory, parses JSON files, sorts by timestamp descending, returns vector of stash entries
- Method to load stash by index: finds stash file with matching index, parses and returns entry
- Method to delete stash: removes file by index, reindexes remaining stashes
- Helper method to reindex stashes: reads all stash files, sorts by timestamp, updates index field in each file

Define StashEntry struct to hold index, message, timestamp, workflow, and workflow_state.

File operations use existing patterns: atomic writes, proper error context, create directories if missing.

Index management ensures after any operation, stashes are numbered 0 to N-1 where 0 is most recent.

### Success Criteria

- FileStorage has public methods for stash operations
- Tests pass for all save, list, load, delete scenarios
- Stash files use JSON format matching SPEC schema
- Index reindexing works correctly after add and delete operations
- Error messages include helpful context about valid ranges
- Stashes directory created automatically if missing

**Commit Point: Storage layer and tests**

## Step 2: Command Layer - Stash and List

### Goal
Implement stash and list commands, building on storage layer. Commands handle workflow state extraction, user-facing output formatting, and validation.

### Step 2.a: Write Tests

Create test module at `src/commands/workflow/tests/stash.rs` with expressive DSL following existing patterns in `tests/mod.rs`.

Build helper functions:
- Helper to stash current workflow with optional message
- Helper to assert stash exists with specific properties
- Helper to verify stash count
- Helper to capture and verify list output format

Test scenarios for stash command:
- Stash active workflow creates file and clears state
- Stash with message includes message in stash entry
- Stash without message works (message is optional)
- Stash displays confirmation with stash reference
- Multiple stashes create sequential indices

Test scenarios for list command:
- List with no stashes shows "No stashes found"
- List with one stash shows formatted entry
- List with multiple stashes shows newest first
- List output format matches SPEC (stash@{N}: mode/node "message")
- List shows relative timestamps

Error cases:
- Stash with no active workflow shows clear error message
- Error message matches SPEC wording exactly

### Step 2.b: Implement

Add stash and list functions to `src/commands/workflow/mod.rs`:

Stash function workflow:
- Load current state from storage
- Validate workflow state exists, error if none
- Extract workflow and workflow_state fields
- Call storage layer to save stash with optional message
- Clear workflow state while preserving session metadata and other global fields
- Display confirmation message with stash reference

List function workflow:
- Call storage layer to retrieve all stashes
- Handle empty case with friendly message
- Format each entry for display: index, workflow mode and node, quoted message if present, relative timestamp
- Use chrono for human-readable relative times
- Print formatted list to stdout

Format output to match SPEC examples exactly.

Validation ensures error messages are user-friendly and actionable.

### Success Criteria

- Stash command creates stash files and clears active workflow
- Stash command accepts optional message flag
- List command displays stashes in correct format
- List handles empty stash directory gracefully
- Error messages match SPEC wording
- Tests express intent clearly without comments

**Commit Point: Stash and list commands with tests**

## Step 3: Command Layer - Pop and Drop

### Goal
Implement pop and drop commands with confirmation prompts and state restoration.

### Step 3.a: Write Tests

Extend test module at `src/commands/workflow/tests/stash.rs`:

Build additional helpers:
- Helper to pop stash at specific index
- Helper to verify workflow state restored correctly
- Helper to simulate user confirmation input for drop

Test scenarios for pop command:
- Pop with no index defaults to index zero
- Pop with specific index restores correct stash
- Pop deletes stash file after restoration
- Pop reindexes remaining stashes
- Pop displays restored workflow prompt like start command
- Round-trip: stash, modify state, pop restores original exactly

Test scenarios for drop command:
- Drop with confirmation removes stash file
- Drop with cancel leaves stash unchanged
- Drop reindexes remaining stashes after deletion
- Drop displays stash info before confirming

Error cases:
- Pop with active workflow shows error with suggestion to abort or stash first
- Pop with invalid index shows error with valid range
- Pop with no stashes shows clear error
- Drop with invalid index shows error
- All error messages match SPEC wording exactly

### Step 3.b: Implement

Add pop and drop functions to `src/commands/workflow/mod.rs`:

Pop function workflow:
- Accept optional index parameter, default to zero
- Load current state and verify no active workflow, error if exists
- Call storage layer to load stash by index
- Extract workflow and workflow_state from stash
- Restore into State while preserving session metadata and global fields
- Save restored state to storage
- Delete stash file via storage layer
- Display workflow prompt matching start command output format

Drop function workflow:
- Accept optional index parameter, default to zero
- Load stash by index via storage layer
- Display stash information and confirmation prompt
- Read user input for confirmation
- If confirmed, delete stash via storage layer and show success message
- If cancelled, exit silently with no changes

Error handling provides clear messages for all validation failures. Use existing error context patterns from codebase.

Confirmation prompt uses standard input reading, format matches SPEC example.

### Success Criteria

- Pop restores workflow state exactly as it was stashed
- Pop fails appropriately when active workflow exists
- Pop handles invalid indices with clear errors
- Drop prompts for confirmation before deleting
- Drop can be cancelled without side effects
- All error messages are actionable and match SPEC
- State round-trip preserves all workflow fields correctly

**Commit Point: Pop and drop commands with tests**

## Step 4: CLI Integration

### Goal
Wire stash commands into main CLI with proper subcommand structure and help text.

### Step 4.a: Write Tests

Integration tests not required - functionality tested at command level. CLI routing is straightforward mapping.

### Step 4.b: Implement

Add Stash enum to Commands in `src/main.rs`:
- Stash subcommand with optional message flag
- List subcommand (no arguments)
- Pop subcommand with optional index parameter
- Drop subcommand with optional index parameter

Add match arm in main function to route Stash commands to appropriate functions from commands::workflow module.

Export new stash functions from `src/commands/mod.rs` alongside existing workflow functions.

Add help text for each subcommand describing purpose and basic usage. Reference SPEC examples for clarity.

### Success Criteria

- All four stash subcommands accessible via hegel CLI
- Help text clear and matches SPEC descriptions
- Commands route to correct implementation functions
- No build warnings or clippy issues

**Commit Point: CLI integration**

## Step 5: Validation and Polish

### Goal
Verify complete feature works end-to-end, ensure test coverage meets standards, confirm all SPEC requirements satisfied.

### Step 5.a: Manual Verification

Run through SPEC test scenarios manually to verify:
- Simple stash and restore workflow
- Multiple stashes with correct indexing
- List output format matches examples
- Error cases produce expected messages
- File operations leave clean state

### Step 5.b: Final Checks

Verify success criteria from SPEC:
- Run full test suite: cargo test passes
- Build succeeds: cargo build with no warnings
- Stash files created in correct location with correct schema
- Session metadata excluded from stash files
- All error cases return non-zero exit codes
- Stash indices sequential after all operations

### Success Criteria

- All SPEC success criteria verified
- Test coverage adequate for core functionality
- Manual verification confirms expected behavior
- No regressions in existing workflow commands

**Final Commit: Documentation and polish if needed**

## Commit Summary

This plan groups work into logical commits:

1. `feat(workflow): add storage layer for workflow stash` - Storage methods and tests
2. `feat(workflow): add stash and list commands` - Stash/list implementation and tests
3. `feat(workflow): add pop and drop commands` - Pop/drop implementation and tests
4. `feat(workflow): integrate stash commands into CLI` - Main.rs routing, help text, and final polish

Each commit represents a cohesive unit of functionality with passing tests.
