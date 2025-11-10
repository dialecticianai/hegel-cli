# Reviews Module Extraction Specification

Extract reviews.json management from hegel-mirror into hegel-cli as reusable library code.

---

## Overview

**What it does:** Moves reviews.json types, I/O functions, and project detection logic from hegel-mirror to hegel-cli's storage layer, enabling code reuse and reducing duplication.

**Key principles:**
- Single source of truth for reviews.json schema and operations
- Maintain hegel-cli's local-first, file-based storage philosophy
- Enable future tools to manage reviews without duplicating code
- Break mirror's dependency inversion (mirror depends on hegel, not vice versa)

**Scope:** Extract HegelReviewEntry types, project detection, path computation, and reviews.json I/O. Keep standalone .review.N file logic in mirror (out of scope).

**Integration context:** Mirror already depends on hegel-cli and uses `FileStorage::find_project_root_from()`. This extraction consolidates reviews.json management into hegel's storage layer.

---

## Data Model

### Types to Extract (NEW in hegel-cli)

All types moved from `hegel-mirror/src/storage.rs` to `hegel-cli/src/storage/reviews.rs`:

**ReviewComment** - Individual review comment
```rust
pub struct ReviewComment {
    pub timestamp: String,              // RFC3339 timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub file: String,                   // Relative path from project root
    pub selection: SelectionRange,
    pub text: String,                   // Selected text snippet
    pub comment: String,                // User's comment
}
```

**SelectionRange / Position** - Text selection coordinates
```rust
pub struct SelectionRange {
    pub start: Position,
    pub end: Position,
}

pub struct Position {
    pub line: usize,
    pub col: usize,
}
```

**HegelReviewEntry** - Single review session
```rust
pub struct HegelReviewEntry {
    pub comments: Vec<ReviewComment>,
    pub timestamp: String,              // RFC3339 timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}
```

**HegelReviewsMap** - Project-global reviews collection
```rust
pub type HegelReviewsMap = HashMap<String, Vec<HegelReviewEntry>>;
// Key: relative file path from project root
// Value: chronological list of review sessions for that file
```

**ProjectType** - Project detection result
```rust
pub enum ProjectType {
    Hegel { root: PathBuf },    // .hegel/ detected, store root path
    Standalone,                  // No .hegel/ found (errors on read/write - not yet implemented)
}
```

### Existing Types (NO CHANGES)

**src/storage/mod.rs::FileStorage** - Already provides `find_project_root_from()` used by detection

---

## Core Operations

### 1. Project Detection

**Function:** `detect_project_type() -> ProjectType`

**Behavior:**
- Calls `FileStorage::find_project_root_from(None)` to detect .hegel/ directory
- Returns `ProjectType::Hegel { root }` if found
- Returns `ProjectType::Standalone` if not found

**Function:** `detect_project_type_from(start_path: Option<PathBuf>) -> ProjectType`

**Behavior:**
- Same as above, but accepts explicit starting path for testing

### 2. Path Computation

**Function:** `compute_relative_path(project_root: &Path, file_path: &Path) -> Result<String>`

**Behavior:**
- Takes .hegel directory path and absolute file path
- Strips .hegel parent directory prefix from file path
- Returns relative path as string (e.g., "src/SPEC.md")
- Errors if file is not within project root

**Example:**
```rust
// project_root = /Users/me/project/.hegel
// file_path = /Users/me/project/src/SPEC.md
// returns "src/SPEC.md"
```

### 3. Read Reviews

**Function:** `read_hegel_reviews(hegel_dir: &Path) -> Result<HegelReviewsMap>`

**Behavior:**
- Reads `.hegel/reviews.json`
- Returns empty HashMap if file doesn't exist or empty
- Parses JSON into HegelReviewsMap
- Errors on malformed JSON

### 4. Write Reviews

**Function:** `write_hegel_reviews(hegel_dir: &Path, reviews: &HegelReviewsMap) -> Result<()>`

**Behavior:**
- Creates `.hegel/` directory if needed
- Serializes to pretty JSON, writes atomically to `.hegel/reviews.json`

**Note for both operations:** Standalone mode not yet implemented - errors if used outside Hegel project

### 5. ReviewComment Constructor

**Function:** `ReviewComment::new(...) -> Self`

**Behavior:**
- Creates ReviewComment with current timestamp (RFC3339)
- Accepts all fields as parameters
- Used by mirror to construct comments before writing

---

## Test Scenarios

### Simple: Project Detection

**Setup:** Temp directory with `.hegel/` subdirectory

**Action:** Call `detect_project_type_from(Some(temp_dir))`

**Expected:** Returns `ProjectType::Hegel { root: hegel_dir }`

### Simple: Standalone Detection

**Setup:** Temp directory without `.hegel/`

**Action:** Call `detect_project_type_from(Some(temp_dir))`

**Expected:** Returns `ProjectType::Standalone`

### Simple: Relative Path Computation

**Setup:** Project at `/tmp/project` with file `/tmp/project/src/test.md`

**Action:** Call `compute_relative_path(&PathBuf::from("/tmp/project/.hegel"), &PathBuf::from("/tmp/project/src/test.md"))`

**Expected:** Returns `"src/test.md"`

### Complex: Read Empty Reviews

**Setup:** `.hegel/` directory exists, no `reviews.json` file

**Action:** Call `read_hegel_reviews(hegel_dir)`

**Expected:** Returns `Ok(HashMap::new())`

### Complex: Write and Read Roundtrip

**Setup:** Create HegelReviewsMap with multiple files and entries

**Action:**
1. Call `write_hegel_reviews(hegel_dir, &reviews)`
2. Call `read_hegel_reviews(hegel_dir)`

**Expected:** Deserialized map equals original map

### Complex: Multiple Reviews Per File

**Setup:** Create map with multiple HegelReviewEntry items for same file

**Action:** Write and read back

**Expected:** Entry order preserved, all reviews present

### Error: File Outside Project

**Setup:** Project at `/tmp/project`, file at `/tmp/other/file.md`

**Action:** Call `compute_relative_path()`

**Expected:** Returns error with helpful message

---

## Success Criteria

- hegel-cli builds successfully with new `src/storage/reviews.rs` module
- All tests pass in hegel-cli: `cargo test`
- hegel-mirror builds successfully with updated imports
- All tests pass in hegel-mirror: `cargo test`
- Mirror's existing reviews.json tests still pass (validate compatibility)
- No functionality regression in mirror's review workflow
- Code extraction reduces total line count across both repos

---

## Out of Scope

- Standalone .review.N file logic (ReviewStorage) - remains in mirror
- Document integration routing logic - stays in mirror
- UI/UX changes to mirror's review interface
- New features or enhancements to reviews.json format
- Migration tooling for existing review files
- CLI commands in hegel for reading/managing reviews
