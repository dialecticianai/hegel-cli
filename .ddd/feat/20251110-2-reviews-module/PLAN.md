# Reviews Module Extraction Plan

Extract reviews.json management from hegel-mirror into hegel-cli storage layer.

---

## Overview

**Goal:** Move reviews.json types, I/O, and project detection from hegel-mirror to hegel-cli for reusability.

**Scope:** Create `src/storage/reviews.rs` in hegel-cli, extract mirror's implementation, update mirror's imports.

**Priorities:**
1. Token efficiency - copy and edit existing code rather than rewrite
2. Zero functionality regression in mirror
3. Maintain test coverage across both repos

**Methodology:** Extract working code with tests, validate roundtrip compatibility, update mirror imports.

---

## Step 1: Create Reviews Module in hegel-cli

### Goal
Establish module structure and copy types from mirror.

### Step 1.a: Write Tests
Create test scaffolder helper for fake Hegel projects. Check existing test_helpers for similar utilities (workflow archives may have one). Write basic tests for project detection and path computation using the scaffolder.

### Step 1.b: Implement
Copy ReviewComment, SelectionRange, Position, HegelReviewEntry, HegelReviewsMap, and ProjectType from mirror's storage.rs to new hegel-cli src/storage/reviews.rs. Update serde derives. Add module to src/storage/mod.rs exports. Implement detect_project_type functions using existing FileStorage::find_project_root_from. Implement or locate compute_relative_path helper.

### Success Criteria
- Module compiles in hegel-cli
- Basic detection tests pass
- Path computation tests pass

**Commit Point:** `feat(storage): add reviews module with types and detection`

---

## Step 2: Add Reviews I/O Functions

### Goal
Extract read_hegel_reviews and write_hegel_reviews from mirror with tests.

### Step 2.a: Write Tests
Copy relevant tests from mirror's storage.rs integration tests. Adapt to use hegel-cli test helpers and the project scaffolder from Step 1. Test empty file handling, roundtrip serialization, multiple files, multiple entries per file. Add standalone mode error test.

### Step 2.b: Implement
Copy read_hegel_reviews and write_hegel_reviews implementations from mirror. Add standalone mode error handling (both functions should error with helpful message when used outside Hegel project). Ensure atomic writes and directory creation. Export from reviews module.

### Success Criteria
- All I/O tests pass
- Roundtrip serialization works
- Standalone mode errors correctly
- hegel-cli builds with no warnings

---

## Step 3: Update Library Exports

### Goal
Make reviews module accessible from hegel crate root.

### Step 3.a: Write Tests
No new tests - validation is that mirror can import and use the module.

### Step 3.b: Implement
Add reviews module to src/lib.rs public exports. Verify module is accessible as `hegel::storage::reviews`.

### Success Criteria
- Module exported from hegel crate
- Documentation builds correctly

**Commit Point:** Bundle Steps 2-3: `feat(storage): add reviews.json I/O and exports`

---

## Step 4: Update hegel-mirror Imports

### Goal
Switch mirror from local implementation to hegel-cli module.

### Step 4.a: Write Tests
No new tests - mirror's existing tests validate compatibility.

### Step 4.b: Implement
Update mirror's src/storage.rs to import types and functions from `hegel::storage::reviews`. Remove local implementations of ReviewComment, HegelReviewEntry, ProjectType, etc. Keep ReviewStorage and standalone .review.N file logic in mirror. Update Document model imports.

### Success Criteria
- Mirror builds successfully
- All mirror tests pass
- No functionality regression in review workflow

**Commit Point:** In mirror repo: `refactor(storage): use hegel reviews module`

---

## Step 5: Verify Cross-Repo Integration

### Goal
Ensure extraction maintains compatibility and reduces duplication.

### Step 5.a: Write Tests
Run full test suites in both repos.

### Step 5.b: Implement
Verify hegel-cli tests pass with new module. Verify mirror tests pass with updated imports. Check that total line count decreased across both repos.

### Success Criteria
- cargo test passes in hegel-cli
- cargo test passes in hegel-mirror
- No duplicate code between repos
- Reviews.json format unchanged

**Commit Point:** No commit - validation only.

---

## Test Scope

**Test:** Project detection, path computation, I/O roundtrips, error handling, integration between repos

**Skip:** Standalone .review.N logic (stays in mirror), UI behavior (GUI tests manual), exhaustive edge cases

---

## Notes

Token efficiency strategy: Copy working implementations from mirror and adapt, rather than rewrite. Tests validate we didn't break anything during extraction.

Standalone mode: ProjectType enum includes Standalone variant, but read/write functions error with "not yet implemented" message. This preserves the type structure for future work without implementing sidecar file logic now.
