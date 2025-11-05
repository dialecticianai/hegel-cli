# Analyze Summary Default - Implementation Plan

Implement brief cross-section summary as default output for `hegel analyze`, with progressive disclosure via flags.

---

## Overview

**Goal**: Change default `hegel analyze` behavior from showing all sections (3000+ lines) to showing a brief cross-section summary (20-30 lines), while preserving all detail via explicit flags.

**Scope**:
- Add `--brief` and `--full` flags
- Rename `--summary` to `--activity`
- Implement new brief renderer
- Update default logic

**Priorities**:
1. Maintain backward compatibility via explicit flags
2. Keep brief output scannable (≤50 lines)
3. Preserve all existing detail levels

**Methodology**: TDD where it validates behavior changes. Focus on:
- Default behavior produces brief output
- Flag combinations work correctly
- Renamed flag maintains same output
- No tests needed for internal rendering details

---

## Step 1: Rename `--summary` to `--activity`

### Goal
Rename the existing summary flag to activity to make room for the new brief summary concept. This is purely a refactoring step with no behavior change.

### Step 1.a: Update Tests
Update existing tests that reference the summary flag to use activity instead. Verify tests still pass with renamed field, confirming no behavior change.

### Step 1.b: Implement Rename
Rename the field in three locations:
- AnalyzeOptions struct field from summary to activity
- CLI argument definition in main.rs from --summary to --activity
- Section display logic variable from show_summary to show_activity

Run existing tests to confirm rename is purely mechanical.

### Success Criteria
- All tests pass with no behavior change
- CLI help text shows --activity instead of --summary
- Running analyze with --activity produces identical output to old --summary

---

## Step 2: Add `--brief` flag and renderer

### Goal
Implement the new brief cross-section summary that aggregates top-level metrics from all sections.

### Step 2.a: Write Tests
Add test for brief rendering that verifies:
- Brief output includes session info
- Brief output includes token totals
- Brief output includes activity counts
- Brief output includes workflow/phase counts
- Output length is reasonable (not thousands of lines)

Test uses sample metrics data and verifies brief renderer extracts and formats key aggregates.

### Step 2.b: Implement Brief Renderer
Create render_brief function in sections.rs that:
- Accepts UnifiedMetrics as input
- Extracts top-level aggregates (session ID, token totals, command/file/commit counts, transition counts, phase counts)
- Formats output concisely with labeled sections
- Includes recent activity highlights if available

Add brief field to AnalyzeOptions struct and CLI args. Wire up brief flag to call render_brief when enabled.

### Success Criteria
- Tests pass validating brief output format and content
- Running analyze --brief shows cross-section summary
- Brief output is under fifty lines for typical projects
- All key metrics represented in brief view

---

## Step 3: Add `--full` flag

### Goal
Provide explicit flag to request all sections in detail, replacing the current implicit default behavior.

### Step 3.a: Write Tests
Add test verifying that --full flag enables all section flags:
- Activity section shown
- Workflow transitions section shown
- Phase breakdown section shown
- Workflow graph section shown

Test compares --full output to manually enabling all individual flags, confirming they produce identical results.

### Step 3.b: Implement Full Flag
Add full field to AnalyzeOptions struct and CLI args. Update section display logic so that when full is true, all section booleans (show_activity, show_workflow_transitions, show_phase_breakdown, show_workflow_graph) become true.

### Success Criteria
- Tests pass validating --full shows all sections
- Running analyze --full produces same output as old default (no flags)
- Full flag does not conflict with individual section flags

---

## Step 4: Update default behavior

### Goal
Change the default when no flags are provided from showing all sections to showing only brief summary.

### Step 4.a: Write Tests
Add test for default behavior (no flags) that verifies:
- Brief summary is shown
- Detailed sections are not shown
- Output is concise

Update existing test for default behavior to expect brief output instead of all sections.

### Step 4.b: Implement Default Logic
Update section display logic in analyze/mod.rs. When no section flags are provided, set show_brief to true and all other section flags to false. The logic should be:
- If no flags: show brief only
- If --full: show all sections
- If individual flags: show only those sections
- If --brief plus others: show brief plus requested sections

### Success Criteria
- Tests pass validating new default behavior
- Running analyze with no flags shows only brief summary
- Default output is under fifty lines
- Old default behavior available via --full flag

---

## Step 5: Verify flag combinations

### Goal
Ensure all flag combinations work as specified, particularly that brief can be combined with other sections.

### Step 5.a: Write Tests
Add tests for key flag combinations:
- brief plus activity shows both
- brief plus workflow-transitions shows both
- full overrides individual flags (or is additive)
- Individual flags without brief show only requested sections

### Step 5.b: Validate Logic
Review section display logic to ensure flag combinations behave correctly. The brief flag should be independently controlled and can appear alongside any other sections. The full flag should enable all detailed sections.

### Success Criteria
- All flag combination tests pass
- Users can request brief plus any subset of detailed sections
- Flag behavior is intuitive and documented in help text

---

## Commit Discipline

After each step:
- Run full test suite to verify changes
- Commit with conventional format: `feat(analyze): complete Step N - description`
- Ensure working tree is clean before next step

Final commit should include updates to any integration tests affected by the default behavior change.

---

## Out of Scope

- Changes to individual section rendering logic (only adding brief renderer)
- Performance optimization of metric collection or rendering
- Changes to archive repair or other analyze subcommands
- Output format changes beyond adding brief section
