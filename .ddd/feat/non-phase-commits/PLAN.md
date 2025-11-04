# Cowboy Mode Activity Attribution Implementation Plan

## Overview

**Goal**: Implement synthetic cowboy workflow attribution to track all development activity, including work done between explicit workflows.

**Scope**: Extend archiving system to detect inter-workflow activity, create synthetic cowboy workflow archives, and display them in analysis/TUI with distinct visual styling.

**Priorities**:
1. Data integrity: synthetic archives match existing archive structure
2. Backward compatibility: old archives load without issues
3. Visual clarity: synthetic workflows clearly distinguished from explicit ones
4. Complete coverage: all inter-workflow activity captured

## Methodology

### TDD Approach
- Test archive structure before implementation
- Test detection logic with time-based scenarios
- Test visualization with mock data
- Integration tests for full workflow cycle

### What to Test
- Synthetic archive creation and structure
- Inter-workflow gap detection
- Time-based grouping (1-hour threshold)
- Archive round-trip serialization
- Display formatting with synthetic flag

### What Not to Test
- Individual timestamp parsing (stdlib tested)
- File I/O edge cases (covered by storage layer)
- Display rendering details (visual testing)

---

## Step 1: Extend Archive Schema

### Goal
Add `is_synthetic` field to `WorkflowArchive` to distinguish auto-detected from explicit workflows.

### Step 1.a: Write Tests
- Test WorkflowArchive serialization with is_synthetic flag
- Test backward compatibility: archives without is_synthetic load as is_synthetic=false
- Test round-trip: serialize with is_synthetic, deserialize preserves value

### Step 1.b: Implement
- Add `is_synthetic: bool` field to WorkflowArchive struct
- Default to false with serde annotation
- Update from_metrics to accept is_synthetic parameter
- Verify existing archives still load (backward compatible)

### Success Criteria
- WorkflowArchive includes is_synthetic field
- Existing archives load without modification
- Serialization preserves is_synthetic value
- All existing tests pass

---

## Step 2: Detect Inter-Workflow Activity Gaps

### Goal
Identify time gaps between archived workflows where development activity occurred.

### Step 2.a: Write Tests
- Test gap detection with 2 workflows and activity between
- Test no gaps when workflows cover full timeline
- Test activity before first workflow
- Test multiple gaps with different activity types
- Test 1-hour grouping threshold
- Test overlapping workflow times (error case)

### Step 2.b: Implement
- Create identify_cowboy_workflows function in new module
- Accept hooks, transcripts, commits, and existing workflow archives
- Build timeline of all workflow time ranges
- Identify activity timestamps outside workflow ranges
- Group consecutive activities within 1-hour window
- Return list of activity groups with time ranges

### Success Criteria
- Detects gaps between workflows correctly
- Groups activity by temporal proximity
- Handles edge cases (pre-workflow, no gaps, overlaps)
- Returns structured gap data with time ranges

---

## Step 3: Create Synthetic Cowboy Archives

### Goal
Generate WorkflowArchive instances for inter-workflow activity gaps.

### Step 3.a: Write Tests
- Test synthetic archive creation from activity gap
- Test mode field set to "cowboy"
- Test is_synthetic set to true
- Test single "ride" phase with aggregated metrics
- Test timestamp-based workflow_id
- Test empty activity results in no archive

### Step 3.b: Implement
- Create build_synthetic_cowboy_archive function
- Accept activity group (hooks, transcripts, commits) and time range
- Set workflow_id to first activity timestamp
- Set mode to "cowboy"
- Set is_synthetic to true
- Create single PhaseArchive named "ride" with aggregated metrics
- Aggregate tokens, bash commands, file modifications, git commits
- Build transitions array (minimal: START → ride → done)
- Compute totals for workflow level

### Success Criteria
- Synthetic archives match WorkflowArchive structure
- All metrics properly aggregated
- Mode and is_synthetic correctly set
- Archives serializable to JSON

---

## Step 4: Integrate with Archive Command

### Goal
Modify `hegel archive` to create synthetic cowboy archives when archiving workflows.

### Step 4.a: Write Tests
- Test archive command creates synthetic archives for gaps
- Test explicit workflow archived first, then synthetic
- Test multiple synthetic archives for multiple gaps
- Test no synthetic archives when no gaps exist
- Test dry-run mode doesn't create synthetic archives

### Step 4.b: Implement
- Modify archive_single_workflow to detect gaps after archiving
- Load all existing archives to get timeline
- Call identify_cowboy_workflows with current activity
- For each gap, call build_synthetic_cowboy_archive
- Write synthetic archives to .hegel/archive/
- Log synthetic archive creation
- Respect dry-run flag (skip writing)

### Success Criteria
- Synthetic archives created during normal archiving
- Logged output indicates synthetic creation
- Dry-run doesn't create files
- All tests pass including integration tests

---

## Step 5: Display in Analysis Output

### Goal
Include synthetic cowboy workflows in `hegel analyze` Phase Breakdown with "(synthetic)" label.

### Step 5.a: Write Tests
- Test analyze output includes synthetic phases
- Test "(synthetic)" label appears in output
- Test synthetic phases sorted chronologically
- Test metrics displayed same as explicit phases
- Test no synthetic phases when none exist

### Step 5.b: Implement
- Modify hegel analyze to load all archives (explicit + synthetic)
- Check is_synthetic flag when building Phase Breakdown
- Append "(synthetic)" to phase name if is_synthetic is true
- Sort phases chronologically across all workflows
- Display same metrics format for synthetic phases

### Success Criteria
- Synthetic workflows appear in analyze output
- Clearly labeled with "(synthetic)"
- Metrics displayed correctly
- Chronological ordering maintained

---

## Step 6: DOT Export with Visual Distinction

### Goal
Render synthetic cowboy workflows as diamond nodes with dashed borders in DOT export.

### Step 6.a: Write Tests
- Test DOT export includes synthetic workflows
- Test synthetic nodes have diamond shape
- Test synthetic nodes have dashed border style
- Test explicit workflows have rounded box shape
- Test node labels include "(synthetic)" indicator

### Step 6.b: Implement
- Modify export_dot to check is_synthetic flag
- For synthetic workflows: shape=diamond, style=dashed
- For explicit workflows: shape=box, style=rounded
- Include "(synthetic)" in node label for synthetic workflows
- Preserve all metrics in node labels

### Success Criteria
- DOT output syntactically valid
- Synthetic nodes visually distinct (diamond + dashed)
- Explicit nodes use standard styling
- Labels correctly formatted

---

## Step 7: TUI Dashboard Integration

### Goal
Display synthetic phases in `hegel top` with special icon and "(synthetic)" label.

### Step 7.a: Write Tests
- Test TUI phases tab includes synthetic phases
- Test diamond icon (◆) appears for synthetic
- Test "(synthetic)" label in phase header
- Test same metrics displayed as explicit phases
- Test interaction (scroll, select) works same as explicit

### Step 7.b: Implement
- Modify TUI phases tab to load all archives
- Check is_synthetic flag when rendering phase list
- Add diamond icon (◆) prefix for synthetic phases
- Append "(synthetic)" to phase name display
- Apply distinct color/styling for synthetic phases
- Maintain same interaction handlers

### Success Criteria
- Synthetic phases appear in TUI
- Visually distinguished with icon and label
- All interactions work correctly
- Real-time updates include synthetic phases

---

## Step 8: Archive Repair Implementation

### Goal
Implement `hegel analyze --fix-archives` to backfill synthetic cowboy workflows for historical data.

### Step 8.a: Write Tests
- Test fix-archives creates missing synthetic archives
- Test existing archives not modified
- Test idempotent (safe to run multiple times)
- Test dry-run shows what would be created
- Test reports number of archives created
- Test handles missing historical data gracefully

### Step 8.b: Implement
- Add --fix-archives flag to analyze command
- Load all existing workflow archives
- Scan historical hooks.jsonl, transcript files, git log
- Call identify_cowboy_workflows with historical data
- Create synthetic archives for detected gaps
- Skip if synthetic archive already exists (timestamp collision)
- Report created archives to user
- Support --dry-run mode

### Success Criteria
- Historical synthetic archives created correctly
- Existing archives never modified
- Idempotent operation (safe reruns)
- Clear reporting of created archives
- Dry-run shows preview without changes

---

## Step 9: Integration Testing

### Goal
End-to-end testing of complete synthetic cowboy workflow lifecycle.

### Step 9.a: Write Tests
- Test complete workflow: activity → archive → analyze → display
- Test mixed timeline: explicit → synthetic → explicit workflows
- Test archive repair on historical data
- Test visualization in all formats (analyze, DOT, TUI)
- Test error handling across all components

### Step 9.b: Implement
- Create integration test with realistic workflow timeline
- Simulate inter-workflow activity (commits, edits, tokens)
- Run archive command
- Verify synthetic archives created
- Run analyze and verify output
- Run export-dot and verify graph
- Test TUI rendering with test data

### Success Criteria
- All integration tests pass
- Complete lifecycle works end-to-end
- Error handling robust across components
- Performance acceptable for typical workloads

---

## Step 10: Documentation and Polish

### Goal
Update user-facing documentation and ensure feature completeness.

### Step 10.a: Write Tests
- Test all help text updated
- Test examples in docs work correctly
- Test error messages clear and actionable

### Step 10.b: Implement
- Update HEGEL_CLAUDE.md with cowboy attribution details
- Update command help text for analyze --fix-archives
- Ensure error messages guide users to solutions
- Add inline code comments for complex logic

### Success Criteria
- Documentation complete and accurate
- Help text comprehensive
- Error messages actionable
- Code well-commented

---

## Error Handling Strategy

### Archive Creation
- Invalid timestamps: skip activity, log warning
- Missing data sources: work with available data
- Disk write failures: abort gracefully, preserve existing archives

### Gap Detection
- Overlapping workflows: log error, prefer explicit over synthetic
- Malformed event data: skip event, continue processing
- Empty activity gaps: don't create synthetic archives

### Display
- Corrupted synthetic archives: skip, show warning in output
- Missing metrics: display "-" for unavailable fields
- Load failures: continue with partial data, warn user

### Archive Repair
- Historical data missing: report what's unavailable
- Partial success: report both successes and failures
- Dry-run errors: show what would fail without changes

---

## Performance Considerations

### Expected Load
- Typical: 5-20 workflows, 10-100 commits, 100-1000 hook events
- Large: 100+ workflows, 1000+ commits, 10000+ events

### Optimization Strategy
- Cache timeline computations during gap detection
- Stream event parsing (don't load all in memory)
- Lazy load archives (only when needed)
- Index archives by timestamp for fast lookup

### Acceptable Thresholds
- Archive creation: <1s for typical project
- Gap detection: <2s for large project
- Display rendering: <100ms for analyze output

---

## Security Considerations

### Input Validation
- Timestamp format validation for all events
- Workflow ID format validation
- Path validation for archive files
- JSON schema validation for archives

### Data Integrity
- Atomic archive writes (temp file + rename)
- Never modify existing archives
- Verify is_synthetic flag on load
- Validate archive structure before use

### Privacy
- No filtering of git commit messages
- User emails stored as-is in archives
- Review artifacts (*.review.*) properly gitignored

---

## Testing Strategy Summary

### Unit Tests (Per Step)
- Archive schema extensions
- Gap detection logic
- Synthetic archive creation
- Display formatting

### Integration Tests (Step 9)
- Full archive → analyze → display cycle
- Mixed explicit/synthetic timelines
- Archive repair workflow
- Error scenarios

### Test Coverage Goals
- Core functionality: 100% coverage
- Error paths: 90% coverage
- Edge cases: 80% coverage
- Visual rendering: manual verification

---

## Rollout Plan

### Phase 1: Core Implementation (Steps 1-4)
- Archive schema + gap detection + synthetic creation + archive command
- Deploy behind feature flag initially
- Monitor for archive corruption issues

### Phase 2: Display Integration (Steps 5-7)
- Analysis output + DOT export + TUI
- Enable by default after validation
- Gather user feedback on clarity

### Phase 3: Archive Repair (Step 8)
- Implement fix-archives command
- Test on production data
- Document repair process

### Phase 4: Polish (Steps 9-10)
- Integration testing
- Documentation
- Performance tuning

---

## Success Metrics

### Functional
- All synthetic archives valid WorkflowArchive instances
- 100% of inter-workflow activity captured
- Visual distinction clear in all output modes
- Archive repair idempotent and safe

### Quality
- Test coverage ≥85% overall
- No regressions in existing archive functionality
- Performance within acceptable thresholds
- Documentation complete and accurate

### User Experience
- Synthetic workflows clearly labeled
- Easy to distinguish from explicit workflows
- Analysis output remains readable
- Error messages helpful and actionable
