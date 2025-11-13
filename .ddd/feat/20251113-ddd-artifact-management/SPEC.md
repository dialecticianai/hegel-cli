# DDD Artifact Management Specification

Formalize conventions for .ddd/ artifacts with validation, creation, and repair tooling.

---

## Overview

**What it does:** Provides structured management of DDD artifacts (feat/refactor/report) with naming convention enforcement, improved visualization, programmatic creation, and automated repair of malformed artifacts.

**Key principles:**
- Naming convention: `YYYYMMDD[-N]-name` (date-based with optional index for same-day artifacts)
- Feat artifacts are directories containing SPEC.md and PLAN.md (both optional)
- Refactor/report artifacts are single markdown files
- Validation warnings guide users to repair tools
- Agent-friendly: non-interactive, stdout-based creation, dry-run safety

**Scope:** Four components working together:
1. `src/ddd.rs` - Core types and validation logic
2. Enhanced `hegel md --ddd` - Improved display with validation warnings
3. `hegel new` - Programmatic artifact creation
4. `hegel doctor` - Automated repair of malformed artifacts

**Integration context:**
- Builds on existing `src/commands/markdown.rs` (markdown tree visualization)
- Used by `hegel md --ddd` for enhanced display
- Integrates with git for date discovery in `hegel doctor`

---

## Data Model

### Core Types (src/ddd.rs)

**NEW: FeatArtifact**
```rust
struct FeatArtifact {
    date: String,          // "20251104" (YYYYMMDD format)
    index: Option<usize>,  // Some(1) for "20251104-1-name", None for "20251104-name"
    name: String,          // "non_phase_commits"
    spec_exists: bool,     // true if SPEC.md exists
    plan_exists: bool,     // true if PLAN.md exists
}

impl FeatArtifact {
    fn dir_name(&self) -> String;  // Returns "20251104-1-non_phase_commits"
    fn dir_path(&self) -> PathBuf; // Returns .ddd/feat/20251104-1-non_phase_commits
}
```

**NEW: RefactorArtifact**
```rust
struct RefactorArtifact {
    date: String,  // "20251104"
    name: String,  // "large_files"
}

impl RefactorArtifact {
    fn file_name(&self) -> String;  // Returns "20251104-large_files.md"
    fn file_path(&self) -> PathBuf; // Returns .ddd/refactor/20251104-large_files.md
}
```

**NEW: ReportArtifact**
```rust
struct ReportArtifact {
    date: String,  // "20251010"
    name: String,  // "tui_dep_review"
}

impl ReportArtifact {
    fn file_name(&self) -> String;  // Returns "20251010-tui_dep_review.md"
    fn file_path(&self) -> PathBuf; // Returns .ddd/report/20251010-tui_dep_review.md
}
```

**NEW: DddArtifact (enum)**
```rust
enum DddArtifact {
    Feat(FeatArtifact),
    Refactor(RefactorArtifact),
    Report(ReportArtifact),
}
```

**NEW: ValidationIssue**
```rust
struct ValidationIssue {
    path: PathBuf,           // Artifact path
    issue_type: IssueType,   // MissingDate, InvalidFormat, MissingIndex
    suggested_fix: String,   // Human-readable fix description
}

enum IssueType {
    MissingDate,    // No date prefix (e.g., "my-feature")
    InvalidFormat,  // Wrong format (e.g., "2025-11-04-name" with dashes)
    MissingIndex,   // Multiple same-day feats without index
}
```

**NEW: DddScanResult**
```rust
struct DddScanResult {
    artifacts: Vec<DddArtifact>,
    issues: Vec<ValidationIssue>,
}
```

---

## Core Operations

### 1. Scanning and Validation

**Function:** `scan_ddd_artifacts() -> Result<DddScanResult>`

**Behavior:**
- Scans `.ddd/feat/`, `.ddd/refactor/`, `.ddd/report/` directories
- Parses directory/file names to extract date, index, and name
- Validates naming convention: `YYYYMMDD[-N]-name`
- For feat artifacts: checks existence of SPEC.md and PLAN.md (not required, just tracked)
- Detects issues: missing dates, invalid formats, missing indices
- Returns parsed artifacts and validation issues

**Validation rules:**
- Date must be 8 digits (YYYYMMDD format)
- Index (if present) must be numeric: `-1-`, `-2-`, etc.
- Name must be non-empty, lowercase with hyphens
- Feat: directory name, refactor/report: file name without `.md` extension

**Example:**
- Valid: `20251104-1-non_phase_commits`, `20251104-large_files.md`
- Invalid: `2025-11-04-name` (wrong date format), `my-feature` (no date), `20251104-name-` (trailing dash)

---

### 2. Enhanced Display (hegel md --ddd)

**Command:** `hegel md --ddd`

**Behavior:**
- Calls `scan_ddd_artifacts()` to get artifacts and issues
- Displays feat artifacts in consolidated format:
  - `20251104-1-non_phase_commits/  SPEC ✓  PLAN ✓`
  - `20251104-2-done_node_refactor/  SPEC ✓  PLAN ✗`
- Shows `⚠️` indicator next to malformed artifacts
- Displays footer message if issues found: `⚠️ Run hegel doctor to fix malformed artifacts`

**Example output:**
```
DDD Documents:
└── .ddd/
    ├── feat/
    │   ├── 20251104-1-non_phase_commits/  SPEC ✓  PLAN ✓
    │   ├── 20251104-2-done_node_refactor/  SPEC ✓  PLAN ✓
    │   ├── my-feature/ ⚠️
    │   └── 20251110-1-review-cli/  SPEC ✓  PLAN ✗
    ├── refactor/
    │   ├── 20251104-large_files.md
    │   └── bad-name.md ⚠️
    └── report/
        └── 20251010-tui_dep_review.md

⚠️ Run hegel doctor to fix malformed artifacts
```

---

### 3. Artifact Creation (hegel new)

**Command:** `hegel new feat <name>`

**Syntax:**
- `hegel new feat <name>` - Create feat directory
- `hegel new refactor <name>` - Output refactor file path
- `hegel new report <name>` - Output report file path

**Parameters:**
- `name` (required): Artifact name (lowercase-with-hyphens)

**Behavior:**

**For feat:**
1. Determine today's date (YYYYMMDD)
2. Check if any feat exists for today
3. If yes: auto-assign next index (`-1-`, `-2-`, etc.)
4. Create directory: `.ddd/feat/YYYYMMDD[-N]-name/`
5. Output to stdout:
   ```
   Created feature directory: .ddd/feat/20251113-my-feature/
   Write your planning documents to: .ddd/feat/20251113-my-feature/
   ```

**For refactor/report:**
1. Determine today's date (YYYYMMDD)
2. Construct file path: `.ddd/refactor/YYYYMMDD-name.md` or `.ddd/report/YYYYMMDD-name.md`
3. Output path to stdout:
   ```
   Write your document to: .ddd/refactor/20251113-my-refactor.md
   ```
4. **Nothing created** - agent creates the file

**Validation:**
- Error if artifact with same name exists for today
- Error if name is empty or invalid format

**Examples:**
```bash
$ hegel new feat my-feature
Created feature directory: .ddd/feat/20251113-my-feature/
Write your planning documents to: .ddd/feat/20251113-my-feature/

$ hegel new feat another-feature  # second feat today
Created feature directory: .ddd/feat/20251113-1-another-feature/
Write your planning documents to: .ddd/feat/20251113-1-another-feature/

$ hegel new refactor my-refactor
Write your document to: .ddd/refactor/20251113-my-refactor.md

$ hegel new feat my-feature  # duplicate
Error: Artifact already exists: .ddd/feat/20251113-my-feature
```

---

### 4. Automated Repair (hegel doctor)

**Command:** `hegel doctor`

**Syntax:**
- `hegel doctor` - Dry-run mode (show fixes, don't apply)
- `hegel doctor --apply` - Apply fixes

**Behavior:**

**Dry-run mode (default):**
1. Scan `.ddd/` for malformed artifacts
2. For each issue, use `git log --follow` to find artifact's addition date
3. Display planned fixes:
   ```
   Found 3 issues:

   1. .ddd/feat/my-feature/ (missing date)
      → Rename to: .ddd/feat/20251107-my-feature/
      (added on 2025-11-07)

   2. .ddd/feat/another-feature/ (missing index)
      → Rename to: .ddd/feat/20251107-1-another-feature/
      (same date as 20251107-my-feature)

   3. .ddd/refactor/bad-name.md (missing date)
      → Rename to: .ddd/refactor/20251108-bad-name.md
      (added on 2025-11-08)

   Run 'hegel doctor --apply' to fix
   ```

**Apply mode (`--apply`):**
1. Perform same detection as dry-run
2. Execute fixes using `git mv` for tracked files, `mv` for untracked
3. Display results:
   ```
   Fixed 3 issues:

   ✓ .ddd/feat/my-feature/ → .ddd/feat/20251107-my-feature/
   ✓ .ddd/feat/another-feature/ → .ddd/feat/20251107-1-another-feature/
   ✓ .ddd/refactor/bad-name.md → .ddd/refactor/20251108-bad-name.md
   ```

**Fixes applied:**
- **Missing date**: Use `git log --follow --format=%ad --date=short <path> | tail -1` to find addition date
- **Missing index**: Detect same-day conflicts, add `-1-`, `-2-` indices in order found

**Edge cases:**
- If git log returns no date (untracked file): warn and skip
- If date conflicts after fix: add index to resolve
- Non-interactive: always safe to run in CI/automation

---

## Test Scenarios

### Simple: Artifact Creation

**Setup:** Empty `.ddd/` directory

**Operation:**
```bash
hegel new feat my-first-feature
hegel new refactor my-refactor
```

**Verification:**
- `.ddd/feat/20251113-my-first-feature/` directory exists
- stdout shows correct paths
- No files created inside directories

---

### Complex: Index Auto-Assignment

**Setup:** `.ddd/feat/20251113-existing-feature/` already exists

**Operation:**
```bash
hegel new feat another-feature
hegel new feat third-feature
```

**Verification:**
- First call creates `.ddd/feat/20251113-1-another-feature/`
- Second call creates `.ddd/feat/20251113-2-third-feature/`
- Existing feature remains `20251113-existing-feature` (no index needed)

---

### Complex: Doctor Repairs Missing Dates

**Setup:**
- `.ddd/feat/my-feature/` (added 2025-11-07 per git)
- `.ddd/feat/another-feature/` (added 2025-11-07 per git)
- `.ddd/refactor/bad-name.md` (added 2025-11-08 per git)

**Operation:**
```bash
hegel doctor --apply
```

**Verification:**
- `.ddd/feat/my-feature/` → `.ddd/feat/20251107-my-feature/`
- `.ddd/feat/another-feature/` → `.ddd/feat/20251107-1-another-feature/`
- `.ddd/refactor/bad-name.md` → `.ddd/refactor/20251108-bad-name.md`
- All renames use `git mv` for tracked files

---

### Error: Duplicate Name

**Setup:** `.ddd/feat/20251113-my-feature/` exists

**Operation:**
```bash
hegel new feat my-feature
```

**Verification:**
- Command exits with error
- stderr: `Error: Artifact already exists: .ddd/feat/20251113-my-feature`
- No duplicate directory created

---

## Success Criteria

- Tests pass: `cargo test ddd::` (all ddd module tests)
- Build succeeds: `cargo build --release`
- `hegel new feat test-feature` creates directory and outputs path
- `hegel new refactor test-refactor` outputs file path without creating file
- `hegel md --ddd` shows consolidated SPEC/PLAN indicators
- `hegel md --ddd` shows ⚠️ for malformed artifacts
- `hegel doctor` detects missing dates and missing indices
- `hegel doctor --apply` renames artifacts using git dates
- All operations work with git-tracked and untracked files
- Commands are non-interactive (agent-friendly)

---

## Out of Scope

**Deferred to future iterations:**
- SPEC.md/PLAN.md stub generation (users create manually)
- Migration from pre-existing naming conventions (manual git operations)
- Validation of SPEC.md/PLAN.md content structure
- Integration with workflow state machine
- TUI interface for doctor repairs
- Archiving completed artifacts
