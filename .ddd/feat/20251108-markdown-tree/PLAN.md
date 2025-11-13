# Markdown Tree Implementation Plan

Build the markdown tree visualization command with TDD discipline and themed output.

---

## Overview

**Goal:** Implement `hegel md` / `hegel markdown` command that scans the current directory for markdown files, categorizes them as DDD artifacts or regular documentation, and displays them in a tree structure with line counts.

**Scope:** Pure Rust implementation with gitignore support, themed terminal output, and optional JSON mode.

**Priorities:**
1. Correct file scanning and DDD classification
2. Consistent theming using existing Theme system
3. Clean tree visualization
4. JSON output for machine consumption

**Methodology:** TDD where it drives development forward. Test core classification logic, tree rendering structure, and JSON output schema. Skip trivial helpers and formatting details.

---

## Step 1: Project Scaffolding

### Goal
Set up command structure, dependencies, and module registration.

### Step 1.a: Add Dependencies

Add required crates to Cargo.toml:
- `ignore` crate for gitignore-aware directory walking
- `walkdir` may not be needed if `ignore` provides walking
- `serde_json` for JSON output (likely already present)
- `chrono` for timestamp formatting (likely already present)

### Step 1.b: Create Command Module

Create new command module at src/commands/markdown.rs with:
- Command argument structure using clap derive macros
- Two flags: `--json` and `--no-ddd`
- Public function to execute the command
- Basic command skeleton that returns success

### Step 1.c: Register Command

Integrate with existing command infrastructure:
- Add module declaration in src/commands/mod.rs
- Export public function from mod.rs
- Add command and alias to main.rs CLI definition
- Wire up the command handler in main match statement

### Success Criteria

- Command builds without errors
- `hegel md --help` shows usage information
- Both `hegel md` and `hegel markdown` are recognized
- Flags `--json` and `--no-ddd` appear in help text

**→ Commit Point: chore(markdown): add command scaffolding and dependencies**

---

## Step 2: File Scanning and Classification

### Goal
Implement directory walking with gitignore support and DDD artifact classification.

### Step 2.a: Write Classification Tests

Create tests for DDD pattern matching:
- Test that files matching `.ddd/**/*.md` are classified as DDD
- Test that files matching `toys/**/*.md` are classified as DDD
- Test that `HANDOFF.md` at any location is classified as DDD with ephemeral flag
- Test that other markdown files are classified as regular
- Test that gitignored files are excluded (except HANDOFF.md)

### Step 2.b: Implement File Scanner

Build directory walking functionality:
- Use `ignore` crate to walk directory tree starting from current directory
- Respect .gitignore files in the tree
- Special case: always include HANDOFF.md files even if gitignored
- Collect only files with .md extension
- Handle permission errors gracefully by skipping and continuing

### Step 2.c: Implement Classification Logic

Create classification function:
- Check if path matches `.ddd/` prefix (at any depth)
- Check if path matches `toys/` prefix followed by .md file
- Check if filename is exactly `HANDOFF.md` (mark as ephemeral)
- Return classification result indicating DDD vs regular and ephemeral flag

### Step 2.d: Implement Line Counting

Create line counting utility:
- Read file contents
- Count newline characters (matching wc -l behavior)
- Handle UTF-8 errors by returning None or error marker
- Return line count as usize

### Success Criteria

- Classification tests pass
- Files are correctly categorized as DDD or regular
- HANDOFF.md files get ephemeral flag set
- Gitignore rules respected with HANDOFF.md exception
- Line counts match wc -l output for sample files
- Permission errors don't crash the scan

**→ Commit Point: feat(markdown): implement file scanning and DDD classification**

---

## Step 3: Tree Rendering with Theming

### Goal
Generate human-readable tree output with proper theming.

### Step 3.a: Write Tree Output Tests

Create tests for tree structure generation:
- Test simple flat structure with mixed DDD and regular files
- Test nested directory structures
- Test empty categories (no DDD files, or no regular files)
- Verify tree characters appear correctly (├──, └──, │)
- Verify line counts appear in correct format

Note: These tests validate structure, not exact theming output.

### Step 3.b: Build Tree Structure

Create tree building logic:
- Group files by category (DDD vs regular)
- Sort files alphabetically within each category
- Build nested directory structure preserving hierarchy
- Generate tree characters based on position (last child vs not)
- Format line counts with parentheses

### Step 3.c: Apply Theme Styling

Integrate with existing Theme system:
- Apply `Theme::header()` to section headers ("DDD Documents:", "Other Markdown:")
- Apply `Theme::secondary()` to directory names and tree characters
- Apply `Theme::metric_value()` to line count numbers
- Apply `Theme::warning()` to `[ephemeral]` markers
- Leave file names unstyled (default terminal color)

### Step 3.d: Implement --no-ddd Filter

Add filtering logic:
- When flag is set, skip DDD categorization entirely
- Only display "Other Markdown" section
- Omit DDD section header and files

### Success Criteria

- Tree structure tests pass
- Output shows correct nesting with tree characters
- Theming applied using Theme methods
- `--no-ddd` flag filters out DDD artifacts
- Empty categories display appropriately
- Visual output is readable and consistent with other Hegel commands

**→ Commit Point: feat(markdown): add tree rendering with themed output**

---

## Step 4: JSON Output Mode

### Goal
Provide machine-readable JSON output with full metadata.

### Step 4.a: Write JSON Tests

Create tests for JSON output structure:
- Test that JSON output matches schema with ddd_documents and other_markdown arrays
- Test that each file entry contains path, lines, size_bytes, last_modified, ephemeral fields
- Test that timestamps are valid ISO 8601 format
- Test that `--no-ddd` omits ddd_documents array in JSON mode

### Step 4.b: Implement Metadata Collection

Extend file scanning to collect metadata:
- File size in bytes from filesystem metadata
- Last modified timestamp from filesystem metadata
- Convert timestamp to ISO 8601 string format
- Preserve line count from earlier step
- Preserve ephemeral flag from classification

### Step 4.c: Build JSON Output

Create JSON serialization:
- Define struct matching JSON schema from SPEC
- Serialize collected file metadata to JSON
- Pretty-print with 2-space indentation
- Output to stdout without any theming

### Step 4.d: Implement JSON with --no-ddd

Handle filtering in JSON mode:
- When flag is set, populate only other_markdown array
- Omit ddd_documents field entirely from JSON output

### Success Criteria

- JSON tests pass
- Output is valid JSON matching schema
- All metadata fields populated correctly
- Timestamps in ISO 8601 format
- `--no-ddd` filters correctly in JSON mode
- No ANSI color codes in JSON output

**→ Commit Point: feat(markdown): add JSON output mode with metadata**

---

## Step 5: Error Handling and Edge Cases

### Goal
Handle error conditions gracefully with appropriate user feedback.

### Step 5.a: Write Error Tests

Create tests for error scenarios:
- Test behavior when no markdown files found
- Test behavior when permission denied on directory
- Test behavior when file is unreadable
- Test behavior with invalid UTF-8 in file (line counting)

### Step 5.b: Implement Error Messages

Add user-friendly error messaging:
- Display "No markdown files found in current directory" when scan returns empty
- Use `Theme::error()` for error messages in tree mode
- Skip unreadable files and continue scanning (log warning if verbose)
- Show "?" for line count when UTF-8 decode fails

### Step 5.c: Integration Testing

Create integration tests using test_helpers:
- Set up temporary directory with sample markdown structure
- Run command and verify output
- Test both tree and JSON modes
- Test with and without `--no-ddd` flag

### Success Criteria

- Error tests pass
- Empty directory shows appropriate message
- Permission errors don't crash the command
- UTF-8 errors handled gracefully
- Integration tests verify end-to-end behavior
- Error messages use Theme methods consistently

**→ Commit Point: feat(markdown): add error handling and integration tests**

---

## Final Success Criteria

Implementation complete when:

- All unit tests pass
- All integration tests pass
- `cargo build` succeeds
- `cargo test` passes
- Command available as both `hegel md` and `hegel markdown`
- Tree output displays correctly with theming
- JSON output validates against schema
- `--no-ddd` flag works in both modes
- Gitignore respected (with HANDOFF.md exception)
- Line counts match wc -l behavior
- Error cases handled gracefully
- No regression in existing Hegel commands

---

## Out of Scope for Implementation

These are explicitly excluded from this implementation phase:

- Manual testing procedures (handled in separate test phase)
- Documentation updates to README or CLAUDE.md (handled in separate doc phase)
- Performance optimization for large repositories
- Recursive .gitignore handling across parent directories
- Custom output formatting beyond tree and JSON
- Watch mode or live updates
- Markdown content parsing or analysis
