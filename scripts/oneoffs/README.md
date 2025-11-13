# One-Off Scripts Archive

**Purpose**: Historical one-off scripts for refactoring, bug reproduction, and specific migrations. Preserved for reference and as examples of refactoring patterns.

**File Naming**: `YYYY-MM-DD-<script-name>` where the date is the git addition date.

---

## Scripts

### 2025-10-15-add-serial-to-cwd-tests.sh
Bash script to add `#[serial]` attribute to tests that mutate working directory using `setup_workflow_env` or `setup_production_workflows`. Prevents parallel execution race conditions.

### 2025-10-15-add-serial-to-env-tests.pl
Perl version of the above. Adds `#[serial]` to tests using regex pattern matching.

### 2025-10-15-refactor-theme-colors.pl
Robust color refactoring script using grep discovery and `hegel astq` for rewrites. Migrates `colored::Colorize` methods to `Theme` tokens with auto-fix for simple cases and manual guidance for complex patterns.

### 2025-10-15-refactor-theme-colors.sh
Bash version of theme color refactoring. Systematic migration of `.cyan()`, `.green()`, etc. to `Theme::*` methods using ast-grep patterns.

### 2025-10-29-debug-meta-mode-bug.sh
Reproduces meta-mode preservation bug. Verifies that meta-mode (e.g., "learning") persists correctly through workflow transitions instead of being reset.

### 2025-10-29-reproduce-init-workflow-bug.sh
Reproduces init-retrofit workflow transition bug where `hegel next` at vision node incorrectly jumps to execution workflow instead of advancing to architecture node.

### 20251103-fix-archive-duplication.pl
One-off script to fix workflow archive duplication bug (date format predates standardization).

### 20251110-remove-explicit-done-nodes.sh
One-off script to remove explicit done nodes from workflow YAML files (date format predates standardization).

---

## Why Keep These?

1. **Historical Reference** - Document decisions and migration patterns
2. **Learning Examples** - Show refactoring techniques (grep + astq, serial test fixes, etc.)
3. **Bug Reproduction** - Reproducible test cases for fixed bugs
4. **Pattern Library** - Reusable patterns for future similar tasks

---

## Usage

These scripts are **not intended for regular use**. They were created for specific one-time tasks and are preserved here for documentation purposes.

If you need to run one:
1. Review the script first to understand its purpose
2. Ensure the conditions it was designed for still apply
3. Use `--dry-run` if available to test first

To archive new one-off scripts, use:
```bash
../archive-oneoff.pl <script-name>
```
