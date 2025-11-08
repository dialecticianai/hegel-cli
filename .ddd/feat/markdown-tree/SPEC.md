# Markdown Tree Visualization Specification

Display repository markdown files in a tree structure, categorizing DDD artifacts separately.

---

## Overview

**What it does:** Scans the current directory tree for markdown files, categorizes them as DDD artifacts or regular documentation, and renders them in a tree visualization with line counts.

**Key principles:**
- DDD artifacts are clearly separated from regular markdown
- Gitignore is respected, with special handling for ephemeral files
- Two output modes: human-readable tree and machine-readable JSON
- Pure Rust implementation (no external `tree` dependency)

**Scope:** Command-line utility for markdown file discovery and visualization

**Integration context:** New command in `src/commands/` module, integrates with existing command structure (`src/commands/mod.rs`, `src/main.rs`), uses existing theme system (`src/theme.rs`)

---

## Data Model

### DDD Artifact Classification

**DDD Documents** (rendered in separate section):
- Any file matching `.ddd/**/*.md` (at any depth)
- Any file matching `toys/**/*.md` (at any depth)
- `HANDOFF.md` at any location (special ephemeral marker)

**Other Markdown** (regular documentation):
- All other `*.md` files not matching DDD patterns

### JSON Output Schema

**NEW Type:** `MarkdownTree` (location: `src/commands/markdown.rs` or similar)

```json
{
  "ddd_documents": [
    {
      "path": ".ddd/SPEC.md",
      "lines": 142,
      "size_bytes": 5847,
      "last_modified": "2025-11-08T12:34:56Z",
      "ephemeral": false
    },
    {
      "path": "HANDOFF.md",
      "lines": 23,
      "size_bytes": 892,
      "last_modified": "2025-11-08T13:45:12Z",
      "ephemeral": true
    }
  ],
  "other_markdown": [
    {
      "path": "README.md",
      "lines": 234,
      "size_bytes": 12456,
      "last_modified": "2025-11-07T09:22:34Z",
      "ephemeral": false
    }
  ]
}
```

**Field purposes:**
- `path`: Relative path from current working directory
- `lines`: Line count (equivalent to `wc -l`)
- `size_bytes`: File size in bytes
- `last_modified`: ISO 8601 timestamp of last modification
- `ephemeral`: Boolean flag (true only for HANDOFF.md files)

---

## Core Operations

### Command Syntax

```bash
hegel md [OPTIONS]
hegel markdown [OPTIONS]  # Alias
```

### Parameters

**Options:**
- `--json`: Output as JSON instead of tree format (optional)
- `--no-ddd`: Exclude DDD artifacts from output (optional)

### Examples

**Simple usage (default tree output):**
```bash
hegel md
```

**JSON output:**
```bash
hegel md --json
```

**Exclude DDD artifacts:**
```bash
hegel md --no-ddd
```

**JSON without DDD artifacts:**
```bash
hegel md --json --no-ddd
```

### Behavior

**Scanning:**
1. Start from current working directory
2. Walk directory tree recursively
3. Collect all `*.md` files
4. Respect `.gitignore` rules with ONE exception: always include `HANDOFF.md` files
5. Categorize files as DDD or Other based on path patterns

**Tree Rendering (default mode):**
1. Sort files alphabetically within each category
2. Group by directory structure
3. Show line counts for each file
4. Mark HANDOFF.md with `[ephemeral]` indicator
5. Use tree-style indentation with box-drawing characters

**JSON Rendering (`--json` mode):**
1. Output structured JSON with two arrays: `ddd_documents` and `other_markdown`
2. Include full metadata for each file
3. Pretty-print JSON with 2-space indentation

**DDD Exclusion (`--no-ddd` flag):**
1. Skip all DDD categorization
2. Only show "Other Markdown" files
3. In JSON mode: omit `ddd_documents` array entirely

### Validation

**Error cases:**
- No markdown files found: Display message "No markdown files found in current directory"
- Permission denied on directory: Skip directory, continue scanning
- Unreadable file: Skip file, continue scanning
- Invalid UTF-8 in file (for line counting): Report error, show "?" for line count

---

## Theming and Visual Styling

**MODIFIED:** Use existing `src/theme.rs::Theme` struct for consistent styling

**Tree output color scheme:**

| Element | Theme Method | Example |
|---------|-------------|---------|
| Section headers ("DDD Documents:", "Other Markdown:") | `Theme::header()` | Bold cyan |
| Directory names | `Theme::secondary()` | Bright black (dimmed) |
| File names | Default (no color) | Plain text |
| Line counts | `Theme::metric_value()` | Cyan |
| `[ephemeral]` marker | `Theme::warning()` | Yellow |
| Tree characters (├──, └──, │) | `Theme::secondary()` | Bright black (dimmed) |
| Error messages | `Theme::error()` | Red |

**Example styled output:**
```
DDD Documents:                          <- Theme::header()
├── .ddd/                               <- Theme::secondary() for directory
│   ├── SPEC.md (142 lines)             <- default for filename, Theme::metric_value() for count
│   └── PLAN.md (89 lines)
└── HANDOFF.md (23 lines) [ephemeral]   <- Theme::warning() for [ephemeral]

Other Markdown:                         <- Theme::header()
└── README.md (234 lines)
```

**JSON output:**
- No theming (plain JSON to stdout)
- Machine-readable format only

---

## Test Scenarios

### Simple: Single directory with mixed markdown

**Setup:**
```
test-repo/
├── README.md (10 lines)
├── HANDOFF.md (5 lines)
└── .ddd/
    └── SPEC.md (20 lines)
```

**Command:** `hegel md`

**Expected output** (with theming applied):
```
DDD Documents:
├── .ddd/
│   └── SPEC.md (20 lines)
└── HANDOFF.md (5 lines) [ephemeral]

Other Markdown:
└── README.md (10 lines)
```

Note: In actual terminal output:
- "DDD Documents:", "Other Markdown:" → bold cyan (`Theme::header()`)
- ".ddd/", tree characters → dimmed (`Theme::secondary()`)
- "(20 lines)", "(5 lines)", "(10 lines)" → cyan (`Theme::metric_value()`)
- "[ephemeral]" → yellow (`Theme::warning()`)

### Complex: Nested structure with multiple features

**Setup:**
```
hegel-cli/
├── README.md
├── ROADMAP.md
├── HANDOFF.md
├── .ddd/
│   ├── SPEC.md
│   ├── PLAN.md
│   └── feat/
│       ├── workflow-stash/
│       │   ├── SPEC.md
│       │   └── PLAN.md
│       └── markdown-tree/
│           └── SPEC.md
├── toys/
│   └── toy1_example/
│       ├── README.md
│       └── notes.md
└── guides/
    ├── SPEC_WRITING.md
    └── PLAN_WRITING.md
```

**Command:** `hegel md --json`

**Expected:** Valid JSON with both categories populated, metadata for all files

**Command:** `hegel md --no-ddd`

**Expected:** Only `guides/` and root-level markdown (excluding HANDOFF.md and .ddd/)

### Error: Gitignored files respected

**Setup:**
```
test-repo/
├── .gitignore (contains "secret.md")
├── README.md
├── HANDOFF.md (in .gitignore)
└── secret.md
```

**Command:** `hegel md`

**Expected:** Shows README.md and HANDOFF.md, but NOT secret.md

### Error: No markdown files

**Setup:** Empty directory or directory with no `.md` files

**Command:** `hegel md`

**Expected:** "No markdown files found in current directory"

---

## Success Criteria

- `cargo test` passes (new tests for markdown command)
- `cargo build` succeeds
- Command available as `hegel md` and `hegel markdown`
- Default output shows tree-style visualization with line counts
- Tree output uses `Theme` methods for consistent styling:
  - Section headers use `Theme::header()`
  - Directories use `Theme::secondary()`
  - Line counts use `Theme::metric_value()`
  - `[ephemeral]` marker uses `Theme::warning()`
  - Tree characters use `Theme::secondary()`
- JSON output matches schema structure
- `--no-ddd` flag correctly filters DDD artifacts
- Gitignore rules respected (except HANDOFF.md)
- HANDOFF.md shown with `[ephemeral]` marker in tree mode
- Line counting matches `wc -l` behavior
- Scanning starts from current working directory
- Command handles permission errors gracefully (skip and continue)

---

## Out of Scope

- Recursive .gitignore handling across all parent directories (just use closest .gitignore)
- Markdown content analysis or parsing
- Syntax highlighting or preview
- Watch mode or live updates
- Filtering by date, size, or other metadata
- Custom categorization rules beyond DDD patterns
- Output to file (stdout only for MVP)
- Color theming customization
- Performance optimization for massive repositories (>10k files)
