# Scripts Directory

**Purpose**: Reusable utilities for Hegel CLI development, testing, and metrics analysis.

**One-off scripts** (refactoring, bug reproduction) are archived in [`oneoffs/`](oneoffs/) with git addition dates.

---

## Build & Release

### build.sh
Build hegel in release mode with optional version bumping and installation. Supports `--install` flag to install to `~/.cargo/bin` and `--skip-bump` to skip version increment (use when testing).

### post-build.sh
Post-build hook that installs release binary to `~/.cargo/bin` after successful release build. Called automatically by `build.sh --install`.

---

## Code Quality & Metrics

### generate-coverage-report.sh
Generates `COVERAGE_REPORT.md` from `cargo-llvm-cov` output with git-diff-friendly formatting. Auto-run by pre-commit hook.

### generate-loc-report.sh
Generates `LOC_REPORT.md` tracking Rust source code and markdown documentation with large file warnings. Auto-run by pre-commit hook.

### test-stability.sh
Runs test suite N times (default 20) to detect flaky tests. Aggregates failure statistics and identifies specific unstable tests.

---

## Development & Analysis

### commits-per-day.py
Counts commits per day from git history with optional ASCII bar graph visualization (`--bars` flag). Useful for tracking development velocity.

---

## Script Management

### archive-oneoff.pl
Archives one-off scripts to `scripts/oneoffs/` directory with git addition date prefix. Supports `--dry-run` mode and uses `git mv` for tracked files.

---

## Metrics & Token Attribution

### analyze/analyze-hook-schema.sh
Analyzes Claude Code hook event schema from `.hegel/hooks.jsonl`. Shows available fields, event types, and data structure for metrics development.

### analyze/analyze-token-attribution.py
Aggregates token attribution analysis from `hegel analyze --debug --json`. Identifies zero-token phases, suspicious patterns, and potential attribution bugs. Reads JSON from stdin.

### analyze/analyze-transcripts.sh
Analyzes Claude Code transcript files for token usage. Extracts event types, usage data structure, and validates token field availability.

### check-hook-fields.sh
Checks for specific fields in Claude Code hook events (token usage, timestamps, session IDs, tool names). Quick field availability verification.

### check-transcript-tokens.sh
Validates that Claude Code transcript files contain token usage data at `message.usage` path. Returns event count and sample usage data.

### scan-gap-transcripts.pl
Scans Claude Code transcripts in a time range and aggregates token metrics (input, output, cache creation/read). Outputs JSON to stdout for Rust parsing.

### summarize-findings.sh
Summarizes findings from hook event analysis. Documents data sources, available fields, and recommendations for metrics implementation.

---

## Debugging Tools

### debug/debug-analyze.sh
Debugs why `hegel analyze` shows "No token data found". Validates transcript paths, token field locations, and identifies root causes.

### debug/debug-phase-count.sh
Debugs phase count anomalies in `.hegel` directories. Compares current state, archived workflows, and metrics output to identify discrepancies.

---

## One-Off Scripts

One-off scripts (refactoring, bug reproduction, specific migrations) have been archived to `oneoffs/` directory with git addition dates.

See [`oneoffs/README.md`](oneoffs/README.md) for archived script descriptions and historical reference.

To archive new one-off scripts:
```bash
./archive-oneoff.pl <script-name>
```

---

## Usage Examples

```bash
# Build and install (with version bump)
./scripts/build.sh --install

# Build and install without bumping version (testing)
./scripts/build.sh --install --skip-bump

# Check for flaky tests (30 iterations)
./scripts/test-stability.sh 30

# Analyze token attribution for a time range
hegel analyze --debug 2025-11-01T00:00:00Z..2025-11-03T00:00:00Z --json | \
  ./scripts/analyze/analyze-token-attribution.py

# Scan transcripts for tokens in specific time window
./scripts/scan-gap-transcripts.pl 2025-11-03T03:13:34Z 2025-11-04T22:11:56Z

# View commit velocity with graph
./scripts/commits-per-day.py --bars

# Archive a one-off script (with dry-run)
./scripts/archive-oneoff.pl --dry-run refactor-theme-colors.sh
./scripts/archive-oneoff.pl refactor-theme-colors.sh  # Actually perform the move

# Debug phase count issues
./scripts/debug/debug-phase-count.sh ~/Code/aecrim/.hegel

# Validate hook event schema
./scripts/analyze/analyze-hook-schema.sh
```

---

## Directory Structure

```
scripts/
├── README.md              # This file
├── analyze/               # Metrics & token attribution
│   ├── analyze-hook-schema.sh
│   ├── analyze-token-attribution.py
│   └── analyze-transcripts.sh
├── debug/                 # Debugging utilities
│   ├── debug-analyze.sh
│   └── debug-phase-count.sh
├── oneoffs/               # Archived one-off scripts
│   ├── README.md
│   └── YYYY-MM-DD-*.{sh,pl,py}
├── build.sh               # Build + version + install
├── post-build.sh          # Install helper
├── generate-*.sh          # Report generation (auto-run)
├── test-stability.sh      # Flaky test detection
├── check-*.sh             # Field validation
├── scan-gap-transcripts.pl # Transcript scanning
├── summarize-findings.sh  # Analysis summary
├── commits-per-day.py     # Git analysis
└── archive-oneoff.pl      # Script management
```

---

**Note**: One-off scripts are archived in `oneoffs/` directory with git addition dates. See `oneoffs/README.md` for details.
