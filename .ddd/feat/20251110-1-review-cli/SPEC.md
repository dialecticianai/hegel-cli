# Review CLI Command Specification

Add `hegel review` command for JSON-based review file management.

---

## Overview

**What it does:** Provides CLI interface to read and write reviews for files in `.hegel/reviews.json`. Operates in two modes: write (stdin present) saves reviews for a file, read (no stdin) displays reviews for a file.

**Key principles:**
- Pure JSON/JSONL interface (human-friendly output deferred to future `--json` flag work)
- Reuses existing `src/storage/reviews.rs` infrastructure
- Consistent path handling with rest of Hegel
- Simple stdin-based mode detection

**Scope:** Add `hegel review <file_path>` command with stdin/stdout JSONL interface. File path required, `.md` extension optional.

**Integration context:** Uses existing `FileStorage`, `read_hegel_reviews()`, `write_hegel_reviews()`, and `compute_relative_path()` from `src/storage/reviews.rs`.

---

## Data Model

### Existing Types (NO CHANGES)

**ReviewComment** - `src/storage/reviews.rs:16`
```rust
pub struct ReviewComment {
    pub timestamp: String,              // RFC3339
    pub session_id: Option<String>,
    pub file: String,                   // Relative path from project root
    pub selection: SelectionRange,
    pub text: String,
    pub comment: String,
}
```

**HegelReviewsMap** - `src/storage/reviews.rs:79`
```rust
pub type HegelReviewsMap = HashMap<String, Vec<HegelReviewEntry>>;
// Key: relative file path
// Value: chronological list of review entries
```

**HegelReviewEntry** - `src/storage/reviews.rs:71`
```rust
pub struct HegelReviewEntry {
    pub comments: Vec<ReviewComment>,
    pub timestamp: String,
    pub session_id: Option<String>,
}
```

### CLI Interface (NEW)

**Command structure:**
```
hegel review <file_path>
```

**file_path:**
- Required argument
- Accepts absolute or relative paths
- Optional `.md` extension (e.g., `PLAN` and `PLAN.md` both work)
- Must exist on filesystem (validates before operation)

---

## Core Operations

### 1. Write Mode (stdin present)

**Syntax:** `cat reviews.jsonl | hegel review path/to/file.md`

**Input format:** JSONL with one complete ReviewComment per line

**Behavior:**
1. Validate file path exists
2. Compute relative path from project root using `compute_relative_path()`
3. Read JSONL from stdin, parse each line as `ReviewComment`
4. Create new `HegelReviewEntry` with parsed comments, current timestamp, and optional session_id
5. Load existing reviews using `read_hegel_reviews()`
6. Append new entry to `Vec<HegelReviewEntry>` for the file's relative path key
7. Write updated map using `write_hegel_reviews()`

**Output:** Single JSON line confirming save
```json
{"file": ".ddd/feat/review-cli/SPEC.md", "comments": 2}
```

**Example:**
```bash
# reviews.jsonl contains:
# {"timestamp":"2025-01-10T10:00:00Z","file":".ddd/feat/reviews-cli/SPEC.md","selection":{"start":{"line":1,"col":0},"end":{"line":1,"col":10}},"text":"# Review CLI","comment":"Good title"}
# {"timestamp":"2025-01-10T10:01:00Z","file":".ddd/feat/reviews-cli/SPEC.md","selection":{"start":{"line":5,"col":0},"end":{"line":5,"col":20}},"text":"## Overview","comment":"Needs more detail"}

cat reviews.jsonl | hegel review .ddd/feat/reviews-cli/SPEC.md
# Output: {"file":".ddd/feat/review-cli/SPEC.md","comments":2}
```

**Validation:**
- File must exist before operation
- Each JSONL line must parse as valid ReviewComment
- Error on malformed JSON with clear message
- Error if not in Hegel project (no `.hegel/` found)

### 2. Read Mode (no stdin)

**Syntax:** `hegel review path/to/file.md`

**Output format:** JSONL with one ReviewComment per line

**Behavior:**
1. Validate file path exists
2. Compute relative path from project root
3. Load reviews using `read_hegel_reviews()`
4. Get `Vec<HegelReviewEntry>` for the relative path key
5. Flatten all comments from all entries
6. Output each ReviewComment as JSONL (one per line)

**Example:**
```bash
hegel review .ddd/feat/reviews-cli/SPEC.md
# Outputs:
# {"timestamp":"2025-01-10T10:00:00Z","file":".ddd/feat/reviews-cli/SPEC.md","selection":{"start":{"line":1,"col":0},"end":{"line":1,"col":10}},"text":"# Review CLI","comment":"Good title"}
# {"timestamp":"2025-01-10T10:01:00Z","file":".ddd/feat/reviews-cli/SPEC.md","selection":{"start":{"line":5,"col":0},"end":{"line":5,"col":20}},"text":"## Overview","comment":"Needs more detail"}
```

**Edge cases:**
- No reviews for file: output nothing (empty output, exit 0)
- File exists but not in reviews.json: output nothing (empty output, exit 0)
- Error if not in Hegel project

### 3. Path Handling

**Absolute paths:** Accept as-is
**Relative paths:** Resolve against current working directory, then compute relative to project root

**Extension handling:**
- If path ends with `.md`, use as-is
- If path doesn't end with `.md`, try with `.md` appended
- If neither exists, error with clear message

**Example:**
```bash
hegel review SPEC          # Tries SPEC, then SPEC.md
hegel review SPEC.md       # Uses SPEC.md directly
hegel review /abs/path/SPEC.md  # Uses absolute path
```

---

## Test Scenarios

### Simple: Write Reviews

**Setup:** Hegel project with existing file `.ddd/SPEC.md`

**Action:**
```bash
echo '{"timestamp":"2025-01-10T10:00:00Z","file":".ddd/SPEC.md","selection":{"start":{"line":1,"col":0},"end":{"line":1,"col":5}},"text":"# Spec","comment":"test"}' | hegel review .ddd/SPEC.md
```

**Expected:**
- `.hegel/reviews.json` contains entry for `.ddd/SPEC.md`
- Entry has one comment

### Simple: Read Reviews

**Setup:** reviews.json with reviews for `.ddd/SPEC.md`

**Action:** `hegel review .ddd/SPEC.md`

**Expected:** JSONL output with all comments for that file

### Simple: Extension Optional

**Setup:** File `.ddd/PLAN.md` exists

**Action:** `hegel review .ddd/PLAN`

**Expected:** Works same as `hegel review .ddd/PLAN.md`

### Complex: Append to Existing Reviews

**Setup:** reviews.json already has one review entry for file

**Action:** Pipe new reviews via stdin

**Expected:** New entry appended to Vec for that file, old reviews preserved

### Complex: Multiple Comments in One Entry

**Setup:** JSONL with 3 ReviewComment objects

**Action:** Pipe all 3 via stdin

**Expected:** Single HegelReviewEntry created with all 3 comments

### Error: File Not Found

**Setup:** File `.ddd/MISSING.md` doesn't exist

**Action:** `hegel review .ddd/MISSING.md`

**Expected:** Error: "File not found: .ddd/MISSING.md"

### Error: Not in Hegel Project

**Setup:** Directory without `.hegel/`

**Action:** `hegel review somefile.md`

**Expected:** Error indicating no Hegel project found

### Error: Malformed JSONL

**Setup:** JSONL with invalid JSON

**Action:** Pipe malformed JSONL via stdin

**Expected:** Error with clear message about JSON parsing failure

---

## Success Criteria

- `hegel review <file>` command available in CLI
- Write mode (stdin present) saves reviews to `.hegel/reviews.json`
- Read mode (no stdin) outputs reviews as JSONL
- Extension optional (`.md` auto-appended if needed)
- Absolute and relative paths both work
- File existence validation before operation
- Errors clearly when not in Hegel project
- All tests pass: `cargo test review`
- Integration with existing reviews.rs types

---

## Out of Scope

- Human-friendly output format (deferred to future `--json` flag work)
- Review comment validation beyond JSON schema
- Session ID management or auto-detection
- GUI integration or interactive mode
- Editing or deleting existing reviews
- Filtering or querying reviews
- Standalone project support (currently errors, as per existing reviews.rs)
