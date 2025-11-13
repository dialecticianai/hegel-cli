# Review CLI Implementation Plan

Implement `hegel review` command for JSON-based review file management.

---

## Overview

**Goal:** Add CLI command that reads/writes reviews to `.hegel/reviews.json` using existing storage infrastructure. Command operates in two modes: write (stdin present) saves reviews, read (no stdin) displays reviews.

**Scope:** CLI integration, path handling, stdin detection, JSONL I/O. Reuses all existing types and functions from `src/storage/reviews.rs`.

**Priorities:**
1. Minimal new code - maximum reuse of existing infrastructure
2. Clear error messages for common failures
3. Consistent path handling with rest of Hegel

**Methodology:** TDD where beneficial. Focus on integration points and error cases. Core storage functions already tested in reviews.rs, so focus on CLI orchestration and path handling.

---

## Step 1: Add CLI Command Structure

### Goal
Establish command definition in main.rs and create handler scaffolding.

### Step 1.a: Write Tests
Create new integration test file `tests/review_cli.rs` (NEW). Write test that validates command exists by parsing help output. Write test that command requires file argument and errors when argument missing.

### Step 1.b: Implement
Add Review variant to Commands enum in `src/main.rs` (MODIFY) with file_path field. Create new file `src/commands/review.rs` (NEW) with handle_review function signature. Add module declaration to `src/commands/mod.rs` (MODIFY). Wire handler to main.rs dispatch. Handler initially returns Ok with no-op.

### Success Criteria
- Command appears in help output
- Command accepts file path argument
- Handler function exists and is wired to main.rs dispatch

**Commit Point:** `feat(review): add CLI command structure and handler scaffold`

---

## Step 2: Implement Write Mode

### Goal
Handle stdin input, parse JSONL, save to reviews.json, output confirmation.

### Step 2.a: Write Tests
Add tests to `tests/review_cli.rs` (MODIFY) that pipe JSONL via stdin. Test successful parse and save of valid ReviewComment JSONL. Test append behavior when reviews already exist for file. Test error cases: malformed JSON, file not found, not in Hegel project. Verify output JSON format.

### Step 2.b: Implement
Update `src/commands/review.rs` (MODIFY) handle_review function. Add stdin detection check. When stdin present, read lines and parse as ReviewComment JSON. Create HegelReviewEntry with parsed comments and current timestamp. Use compute_relative_path from `src/storage/reviews.rs` to get relative path. Call read_hegel_reviews, append new entry to Vec for file key, call write_hegel_reviews. Print single JSON line with file and comment count.

### Success Criteria
- Stdin detection works correctly
- JSONL parsing handles valid ReviewComment objects
- New entry appends to existing reviews for file
- Output JSON confirms save with correct count
- Errors clearly on malformed JSON
- Errors clearly when file not found

---

## Step 3: Implement Read Mode

### Goal
Display existing reviews as JSONL output when stdin absent.

### Step 3.a: Write Tests
Add tests to `tests/review_cli.rs` (MODIFY) for read mode. Test loading and displaying reviews for file with existing reviews in reviews.json. Test flattening comments across multiple HegelReviewEntry objects. Test empty output when file has no reviews. Test empty output when file exists but not in reviews.json.

### Step 3.b: Implement
Update `src/commands/review.rs` (MODIFY) to handle read mode when stdin absent. Compute relative path for file. Call read_hegel_reviews from `src/storage/reviews.rs`. Get Vec of HegelReviewEntry for file key from HashMap. Flatten all comments from all entries. Serialize each ReviewComment as JSON and print as JSONL.

### Success Criteria
- Outputs JSONL with one ReviewComment per line
- Empty output when no reviews exist
- All comments across all entries included
- JSON serialization matches input format

**Commit Point:** `feat(review): implement read and write modes with JSONL I/O`

---

## Step 4: Add Path Handling and Validation

### Goal
Support optional .md extension, validate file existence, handle absolute and relative paths.

### Step 4.a: Write Tests
Add tests to `tests/review_cli.rs` (MODIFY) for path handling. Test paths without .md extension resolve to .md variant. Test absolute paths work correctly. Test relative paths work correctly. Test clear error when file doesn't exist (tried both with and without .md). Test clear error when not in Hegel project.

### Step 4.b: Implement
Update `src/commands/review.rs` (MODIFY) to add path resolution at start of handle_review. Try path as-is first, then with .md appended if not found. Return error if neither exists. Use `std::fs::canonicalize` for path resolution. Call FileStorage::find_project_root_from to detect Hegel project, error if not found. Pass validated path to existing logic.

### Success Criteria
- Extension optional (SPEC and SPEC.md both work)
- Absolute paths work correctly
- Relative paths work correctly
- Clear error when file not found
- Clear error when not in Hegel project
- Path computation matches existing Hegel conventions

**Commit Point:** `feat(review): add path handling and validation`

---

## Test Scope

**Test:** CLI argument parsing, stdin detection, JSONL parsing, path resolution with optional extension, file validation, append behavior, error messages

**Skip:** Storage layer functions (already tested in reviews.rs), JSON serialization (serde handles), project root discovery (tested in storage)

---

## Notes

Extension optional behavior: try exact path first, then with .md appended. This matches user expectation and allows both styles.

Error messages should guide user: "File not found: path/to/file" or "Not in Hegel project (no .hegel/ directory found)".

Stdin detection: use atty crate or check if stdin is terminal. Standard pattern used elsewhere in CLI tools.
