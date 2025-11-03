# Test Helpers

Reusable test utilities and fixtures for Hegel tests. Split from a 579-line monolithic `test_helpers.rs` into focused modules for better organization and discoverability.

## Philosophy

Test helpers are **infrastructure** - they compress boilerplate and provide context locality for related utilities. When test patterns become repetitive (>3 uses), extract them here.

## Module Organization

### `mod.rs` - Module Root
Re-exports all public items from submodules for backwards compatibility. Tests can use `use crate::test_helpers::*;` to access everything.

### `storage.rs` - Storage and State Helpers
**Purpose**: Setup test storage, create test state, and assert state equality.

**Exports**:
- `test_storage()` - Create temporary test storage
- `test_workflow_state()` - Create test workflow state with defaults
- `assert_state_eq()` - Deep equality assertion for State objects

**Unused**:
- None currently

### `jsonl.rs` - JSONL File Utilities
**Purpose**: Create and read JSONL test files (hooks, transcripts, states).

**Exports**:
- `read_jsonl_all()` - Read all JSONL records from file
- `create_hooks_file()` - Create test hooks.jsonl with sample data
- `create_transcript_file()` - Create test transcript JSONL with sample data
- `create_states_file()` - Create test states.jsonl with sample data
- `count_jsonl_lines()` - Count lines in JSONL file ⚠️ UNUSED

**Unused**:
- ⚠️ `count_jsonl_lines()` - Reserved for future metrics validation tests

### `workflow.rs` - Workflow Builders and Setup
**Purpose**: Build test workflows, setup workflow environments, create test claims.

**Exports**:
- `WorkflowBuilder` - Fluent builder for test workflows
- `setup_workflow_env()` - Setup temp dir + storage with discovery workflow ⚠️ UNUSED
- `setup_meta_mode_workflows()` - Setup learning + discovery workflows ⚠️ UNUSED
- `setup_production_workflows()` - Setup execution workflow ⚠️ UNUSED
- `claim()` - Create claim HashSet for transitions ⚠️ UNUSED
- `TEST_WORKFLOW_YAML` - Minimal workflow YAML constant ⚠️ UNUSED

**Unused**:
- ⚠️ `TEST_WORKFLOW_YAML` - Simple workflow definition for tests (reserved for engine tests)
- ⚠️ `claim()` - Create claim sets for workflow transitions (reserved for transition tests)
- ⚠️ `setup_workflow_env()` - Discovery workflow setup (reserved for command integration tests)
- ⚠️ `setup_meta_mode_workflows()` - Meta-mode workflow setup (reserved for meta-mode tests)
- ⚠️ `setup_production_workflows()` - Execution workflow setup (reserved for production workflow tests)

### `metrics.rs` - Metrics Builders
**Purpose**: Build test metrics data structures for hooks, transcripts, and unified metrics.

**Exports**:
- `UnifiedMetricsBuilder` - Fluent builder for creating test UnifiedMetrics ⚠️ UNUSED
- `test_storage_with_files()` - Create storage with sample JSONL files ⚠️ UNUSED
- `test_unified_metrics()` - Create sample UnifiedMetrics for testing ⚠️ UNUSED

**Unused**:
- ⚠️ `UnifiedMetricsBuilder` - Reserved for comprehensive metrics analysis tests
- ⚠️ `test_storage_with_files()` - Reserved for metrics integration tests
- ⚠️ `test_unified_metrics()` - Reserved for metrics rendering/analysis tests

### `tui.rs` - TUI Test Utilities
**Purpose**: Terminal testing utilities for ratatui-based TUI components.

**Exports**:
- `SMALL_TERM`, `MEDIUM_TERM`, `LARGE_TERM` - Terminal size constants ⚠️ UNUSED
- `test_terminal()` - Create TestBackend terminal ⚠️ UNUSED
- `buffer_to_string()` - Convert terminal buffer to string ⚠️ UNUSED
- `render_to_string()` - Render widget to string ⚠️ UNUSED

**Unused**:
- ⚠️ All exports - Reserved for future TUI testing (`hegel top` dashboard tests)

### `fixtures.rs` - Test Fixture Loader
**Purpose**: Load test fixture files for adapter tests.

**Exports**:
- `load_fixture()` - Load fixture file from `tests/fixtures/`

**Unused**:
- None currently

## Usage Guidelines

**DO**:
- Extract test patterns when used >3 times
- Add comprehensive doc comments with examples
- Keep helpers focused and composable
- Use `WorkflowBuilder` for complex workflow construction

**DON'T**:
- Add one-off helpers (inline them in tests)
- Create helpers before they're needed (YAGNI)
- Mix test logic with helper logic (helpers are pure setup)

## Unused Items Policy

Items marked ⚠️ UNUSED are **intentionally kept** as infrastructure for planned tests:
- TUI testing (for `hegel top` dashboard)
- Comprehensive metrics analysis tests
- Meta-mode transition tests
- Production workflow validation tests

These warnings are **silenced via `#[allow(dead_code)]`** after being documented here. When you use an unused helper, remove it from the "Unused" section above and remove the `#[allow(dead_code)]` attribute.

## Future Work

As test coverage expands, these helpers will be used for:
1. **TUI testing** - Testing dashboard rendering and interactions
2. **Metrics validation** - Verifying metrics collection accuracy
3. **Workflow transitions** - Testing meta-mode orchestration
4. **Integration tests** - End-to-end workflow execution tests
