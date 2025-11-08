# Workflow Stash Specification

Enable pausing and resuming workflows by stashing them, similar to `git stash`.

## Overview

**What it does:** Provides workflow state management for context-switching between tasks. Users can stash their current workflow (saving it to a stack), work on something else, then restore the stashed workflow later.

**Key principles:**
- **Stack-based:** Multiple stashes indexed from 0 (most recent)
- **Optional messages:** Descriptive messages via `-m` flag for context
- **Explicit operations:** No auto-stashing—all actions require user confirmation
- **Workflow isolation:** Only stash workflow-specific state, not session metadata

**Scope:** Core stash operations (stash, list, pop, drop) with minimal viable functionality. No named references, branching, or auto-stashing.

**Integration context:** Integrates with existing workflow state management in `src/storage/mod.rs` and command structure in `src/commands/workflow/mod.rs`. Stashes stored in `.hegel/stashes/` directory.

## Data Model

### Stash Entry (NEW)

Each stash is a JSON file at `.hegel/stashes/<timestamp>.json`:

```json
{
  "index": 0,
  "message": "fixing auth bug",
  "timestamp": "2025-11-07T12:34:56.789Z",
  "workflow": { ... },
  "workflow_state": {
    "current_node": "code",
    "mode": "execution",
    "history": ["spec", "plan", "code"],
    "workflow_id": "2025-11-07T10:00:00Z",
    "meta_mode": null,
    "phase_start_time": "2025-11-07T11:00:00Z",
    "is_handlebars": false
  }
}
```

**Fields:**
- `index` (number): Position in stash stack (0 = most recent)
- `message` (string, optional): User-provided description
- `timestamp` (string): ISO 8601 timestamp when stash was created
- `workflow` (object): Complete workflow YAML as JSON (from `State.workflow`)
- `workflow_state` (object): Complete workflow state (from `State.workflow_state`)

**What gets stashed:** Only workflow-specific state from `src/storage/mod.rs::State`:
- `workflow` field
- `workflow_state` field

**What does NOT get stashed:** Session-global state:
- `session_metadata` (session-specific)
- `cumulative_totals` (global metrics)
- `git_info` (cached git data)

### Modified Types

**`src/storage/mod.rs::FileStorage`** (MODIFIED):
- Add methods for stash operations (load stashes, save stash, delete stash)
- Stash directory: `.hegel/stashes/`

**`src/commands/workflow/mod.rs`** (MODIFIED):
- Add public functions: `stash_workflow`, `list_stashes`, `pop_stash`, `drop_stash`

**`src/main.rs::Commands`** (MODIFIED):
- Add `Stash` subcommand with nested subcommands

## Core Operations

### stash

**Syntax:**
```bash
hegel stash [-m "message"]
```

**Parameters:**
- `-m, --message <MSG>`: Optional descriptive message for the stash

**Behavior:**
1. Verify active workflow exists (error if none)
2. Load current `State` from `.hegel/state.json`
3. Extract `workflow` and `workflow_state` fields
4. Create timestamp (ISO 8601)
5. Build stash entry with index, message, timestamp, workflow, workflow_state
6. Write to `.hegel/stashes/<timestamp>.json`
7. Reindex all stashes (update index fields: 0 = newest)
8. Clear workflow state (preserving session_metadata, cumulative_totals, git_info)
9. Display confirmation with stash reference

**Example output:**
```
Saved working directory to stash@{0}: execution/code "fixing auth bug"
```

**Validation:**
- Error if no active workflow: "No active workflow to stash"
- Create `.hegel/stashes/` directory if missing

**Edge cases:**
- Stashing at terminal node: Allowed (user might want to revisit)
- Empty message: Allowed (message is optional)

### list

**Syntax:**
```bash
hegel stash list
```

**Behavior:**
1. Read all files from `.hegel/stashes/` directory
2. Parse JSON and sort by timestamp (newest first)
3. Reindex if necessary (ensure indices match sort order)
4. Display formatted list with index, workflow/node, message, relative time

**Example output:**
```
stash@{0}: execution/code (2 hours ago)
stash@{1}: discovery/plan "fixing auth bug" (yesterday)
stash@{2}: research/study (3 days ago)
```

**Format details:**
- Index: `stash@{N}`
- Workflow/node: `<mode>/<current_node>`
- Message: Quoted string if present, omitted if not
- Time: Relative human-readable (uses chrono humanize)

**Validation:**
- If no stashes exist: Display "No stashes found"

### pop

**Syntax:**
```bash
hegel stash pop [index]
```

**Parameters:**
- `index` (optional): Stash index to restore (defaults to 0)

**Behavior:**
1. Verify no active workflow exists (error if exists)
2. Load stash at specified index from `.hegel/stashes/`
3. Restore `workflow` and `workflow_state` to `State`
4. Preserve existing `session_metadata`, `cumulative_totals`, `git_info`
5. Save restored state to `.hegel/state.json`
6. Delete stash file
7. Reindex remaining stashes
8. Display restored workflow prompt (like `hegel start`)

**Example output:**
```
Restored stash@{1}: discovery/plan "fixing auth bug"

Mode: discovery
Current node: plan

Prompt:
<prompt content>
```

**Validation:**
- Error if active workflow exists: "Cannot restore stash: active workflow at '<mode>/<node>'. Run 'hegel abort' or 'hegel stash' first."
- Error if index doesn't exist: "Stash index {N} not found. Available stashes: 0-{max}"
- Error if no stashes: "No stashes to restore"

**Edge cases:**
- Popping creates gaps in indices: Automatically reindex remaining stashes

### drop

**Syntax:**
```bash
hegel stash drop [index]
```

**Parameters:**
- `index` (optional): Stash index to delete (defaults to 0)

**Behavior:**
1. Load stash at specified index
2. Display confirmation prompt with stash details
3. Wait for user confirmation (y/n)
4. Delete stash file if confirmed
5. Reindex remaining stashes
6. Display confirmation message

**Example output:**
```
Drop stash@{1}: discovery/plan "fixing auth bug"? (y/n): y
Dropped stash@{1}
```

**Validation:**
- Error if index doesn't exist: "Stash index {N} not found. Available stashes: 0-{max}"
- Error if no stashes: "No stashes to drop"
- Abort if user cancels (n): No changes, exit silently

**Edge cases:**
- Dropping creates gaps: Automatically reindex

## Test Scenarios

### Simple Cases

**Stash and restore:**
1. Start workflow: `hegel start discovery`
2. Advance to plan: `hegel next`
3. Stash workflow: `hegel stash -m "exploring idea"`
4. Verify state cleared: `hegel status` shows no workflow
5. Restore stash: `hegel stash pop`
6. Verify workflow restored at plan node with full history

**List empty stashes:**
1. Fresh project with no stashes
2. Run `hegel stash list`
3. Expect: "No stashes found"

### Complex Cases

**Multiple stashes:**
1. Start discovery workflow, advance to plan
2. Stash: `hegel stash -m "idea A"`
3. Start execution workflow, advance to code
4. Stash: `hegel stash -m "idea B"`
5. Start research workflow
6. List stashes: See 2 stashes with correct indices
7. Pop specific: `hegel stash pop 1` restores "idea A"
8. Verify index 0 becomes old index 0 ("idea B")

**Drop middle stash:**
1. Create 3 stashes (indices 0, 1, 2)
2. Drop index 1: `hegel stash drop 1`
3. List stashes: Verify remaining are reindexed to 0, 1

### Error Cases

**Stash without active workflow:**
1. Fresh state, no workflow active
2. Run `hegel stash`
3. Expect error: "No active workflow to stash"

**Pop with active workflow:**
1. Start workflow: `hegel start discovery`
2. Create stash in separate state (manual setup)
3. Try to pop: `hegel stash pop`
4. Expect error: "Cannot restore stash: active workflow at 'discovery/spec'. Run 'hegel abort' or 'hegel stash' first."

**Pop invalid index:**
1. Create 2 stashes (indices 0, 1)
2. Try: `hegel stash pop 5`
3. Expect error: "Stash index 5 not found. Available stashes: 0-1"

**Drop with confirmation cancel:**
1. Create stash
2. Run: `hegel stash drop 0`
3. Type 'n' at prompt
4. Verify stash still exists

## Success Criteria

Agent-verifiable:

- `cargo test` passes all tests for stash functionality
- `cargo build` succeeds with no warnings
- `hegel stash` creates `.hegel/stashes/<timestamp>.json` file
- `hegel stash list` displays stashes in reverse chronological order
- `hegel stash pop` restores workflow and deletes stash file
- `hegel stash pop` with active workflow exits with error code 1
- `hegel stash drop` with user confirmation removes stash file
- Stash indices are sequential starting from 0 after any operation
- Stashed state round-trips correctly (save → restore preserves all fields)
- Session metadata not included in stash files
- All error cases return non-zero exit codes
- Error messages match specified formats

## Out of Scope

Deferred to future iterations:

- Named stash references (e.g., `hegel stash save my-feature`)
- Stash branching or tags
- Auto-stashing on workflow transitions
- `hegel stash show` to preview stash contents
- `hegel stash apply` (restore without deleting)
- Stash diffs
- Stash cleanup/prune commands
- Interactive stash selection
