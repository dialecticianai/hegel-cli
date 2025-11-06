# Handlebars Template Engine Implementation Plan

Add Handlebars support for workflow templates while preserving existing Markdown template system.

---

## Overview

**Goal:** Enable Handlebars templates for guides and workflow prompts, allowing conditional logic and partials. Both old and new systems coexist without breaking changes.

**Scope:**
- Add Handlebars crate and create rendering module
- Support dual template engines (route by extension)
- Implement partial loading with precedence rules
- Migrate code_map templates as proof-of-concept
- Integration with workflow YAML parser

**Priorities:**
1. Non-breaking: existing templates continue working
2. TDD: tests drive interface design
3. Security: reuse path validation from existing system
4. Minimal: vanilla Handlebars with one custom helper (eq)

---

## Methodology

**TDD Approach:**
- Write tests for each major component before implementation
- Focus on interfaces: partial loading, rendering, routing
- Test security validation and error cases
- Integration test with actual workflow YAML

**What to Test:**
- Handlebars rendering with context variables
- Partial loading from guides/partials directory
- Load precedence (partials over root guides)
- Custom eq helper for conditionals
- Security validation (path traversal rejection)
- Dual-engine routing (prompt vs prompt_hbs)
- Integration with existing workflow system

**What to Skip:**
- Handlebars internals (trust the library)
- Performance optimization (premature)
- Exhaustive edge cases (focus on core behavior)
- All existing template.rs tests (don't re-run, trust they still pass)

---

## Step 1: Add Handlebars Dependency and Module Structure

### Goal
Establish foundation for Handlebars integration without breaking existing code.

### Step 1.a: Add Dependency and Create Module

Add handlebars crate to Cargo.toml dependencies section. Create new module file for Handlebars rendering at src/engine/handlebars.rs. Export the new module from src/engine/mod.rs alongside existing template module.

### Step 1.b: Write Placeholder Tests

Create test module in src/engine/handlebars.rs with placeholder tests for basic rendering functionality. Tests should verify that Handlebars can render a simple template with context variables.

### Success Criteria

- Cargo.toml includes handlebars dependency
- src/engine/handlebars.rs file exists with module structure
- src/engine/mod.rs exports handlebars module
- Placeholder tests compile (may be marked ignore initially)
- cargo build succeeds
- Existing tests still pass (no regressions)

**Commit:** feat(templates): add handlebars dependency and module structure

---

## Step 2: Implement Partial Loading System

### Goal
Enable loading Handlebars partials from guides/partials and guides directories with correct precedence.

### Step 2.a: Write Partial Loading Tests

Write tests that verify partial loading behavior. Test should check that partials load from guides/partials directory first, then fall back to guides directory. Test should verify that partial names are validated for security (reject path traversal, absolute paths, nested subdirectories). Test missing partial returns appropriate error.

### Step 2.b: Implement Partial Registration

Implement function to scan guides/partials directory for hbs files and register them as Handlebars partials. Each file stem becomes the partial name. Implement fallback to guides directory for additional partials. Reuse validate_guide_name logic from existing template.rs module for security validation. Return error if partial file cannot be read or name is invalid.

### Success Criteria

- Function registers partials from guides/partials directory
- Partials from guides/partials take precedence over guides directory
- Security validation rejects path traversal attempts
- Security validation rejects absolute paths
- Tests verify precedence behavior
- Tests verify security validation
- cargo test passes for new tests

**Commit:** feat(templates): implement Handlebars partial loading with precedence

---

## Step 3: Implement Core Handlebars Rendering

### Goal
Create public rendering function that processes Handlebars templates with context and partials.

### Step 3.a: Write Rendering Tests

Write tests for basic Handlebars rendering. Test simple variable substitution. Test partial inclusion. Test conditional logic using if/else blocks. Test eq helper for string comparison (needed for code_map). Test error cases: missing context variables, missing partials, invalid template syntax.

### Step 3.b: Implement Rendering Function

Implement public render_template_hbs function that initializes Handlebars registry, registers partials, registers custom eq helper for string equality comparisons, renders template with provided context, and returns rendered string or error. Custom eq helper should compare two values and return boolean for use in if conditions.

### Success Criteria

- render_template_hbs function accepts template string, guides directory, and context
- Function initializes Handlebars and registers partials
- Custom eq helper enables conditional logic
- Tests verify variable substitution works
- Tests verify partial inclusion works
- Tests verify conditional rendering with eq helper
- Tests verify error handling for missing partials and invalid syntax
- cargo test passes

**Commit:** feat(templates): implement Handlebars rendering with custom eq helper

---

## Step 4: Add Dual-Engine Routing

### Goal
Enable selection between old Markdown engine and new Handlebars engine based on context.

### Step 4.a: Write Routing Tests

Write tests that verify routing behavior. Test that when is_handlebars flag is false, old template engine is used. Test that when is_handlebars flag is true, new Handlebars engine is used. Test that both engines can coexist in same test run without conflicts.

### Step 4.b: Implement Router Function

Implement render_prompt function in src/engine/mod.rs that accepts template content, is_handlebars boolean flag, guides directory, and context. Function delegates to template::render_template when is_handlebars is false. Function delegates to handlebars::render_template_hbs when is_handlebars is true. Function propagates errors from both engines.

### Success Criteria

- render_prompt function exists in engine module
- Function routes to correct engine based on flag
- Both engines remain functional
- Tests verify routing behavior
- Tests verify error propagation
- cargo test passes

**Commit:** feat(templates): add dual-engine routing for Markdown and Handlebars

---

## Step 5: Extend Workflow YAML Support

### Goal
Allow workflow nodes to specify prompt_hbs field for Handlebars templates.

### Step 5.a: Write YAML Parsing Tests

Write tests for Node struct deserialization. Test that node with only prompt field deserializes correctly. Test that node with only prompt_hbs field deserializes correctly. Test that node with both prompt and prompt_hbs fields returns validation error. Test that done node with neither field is valid. Test workflow validation catches conflicting fields.

### Step 5.b: Modify Node Struct and Validation

Add prompt_hbs field to Node struct as optional String. Update Node deserialization to handle both fields. Add validation in Workflow::validate that ensures nodes do not have both prompt and prompt_hbs fields simultaneously. Update get_next_prompt function to detect which field is present and call render_prompt with appropriate is_handlebars flag.

### Success Criteria

- Node struct has prompt_hbs field
- Validation rejects nodes with both prompt and prompt_hbs
- Validation allows nodes with only prompt or only prompt_hbs
- get_next_prompt routes to correct engine based on field presence
- Tests verify YAML parsing behavior
- Tests verify validation rules
- cargo test passes

**Commit:** feat(templates): support prompt_hbs field in workflow YAML

---

## Step 6: Create Proof-of-Concept Code Map Partial

### Goal
Migrate code_map templates from two separate Markdown files to one Handlebars file with conditionals.

### Step 6.a: Write Code Map Integration Test

Write integration test that verifies code_map partial loads and renders correctly. Test hierarchical mode renders content about larger projects. Test monolithic mode renders content about small projects. Test that conditional logic selects correct branch based on code_map_style context variable. Test that new hbs partial takes precedence over old md templates.

### Step 6.b: Create Handlebars Code Map Partial

Create guides/partials directory if it does not exist. Create guides/partials/code_map.hbs file. Combine content from guides/templates/code_map_hierarchical.md and guides/templates/code_map_monolithic.md into single file. Use Handlebars if/else conditional with eq helper to select content based on code_map_style context variable. Leave old md files in place for precedence testing.

### Success Criteria

- guides/partials directory exists
- guides/partials/code_map.hbs exists with conditional logic
- Old md template files remain unchanged
- Integration test verifies hierarchical mode renders correctly
- Integration test verifies monolithic mode renders correctly
- Integration test verifies precedence (hbs over md)
- cargo test passes

**Commit:** feat(templates): migrate code_map to Handlebars with conditionals

---

## Step 7: End-to-End Integration Testing

### Goal
Verify complete system works with actual workflow YAML and guide loading.

### Step 7.a: Write Workflow Integration Test

Write integration test that loads workflow YAML with prompt_hbs field. Test verifies that workflow loads successfully. Test verifies that get_next_prompt renders using Handlebars engine. Test verifies that partials from guides/partials are accessible. Test verifies that context variables propagate correctly. Test existing workflows with prompt field still work unchanged.

### Step 7.b: Validate System Integration

Run full test suite to verify no regressions. Build release binary to verify compilation. Create simple test workflow YAML file demonstrating prompt_hbs usage. Execute workflow initialization and prompt rendering manually to verify end-to-end flow.

### Success Criteria

- Integration test verifies workflow YAML with prompt_hbs works
- Integration test verifies context propagation through system
- Full test suite passes with no regressions
- cargo build --release succeeds
- Manual verification of workflow with Handlebars prompt succeeds
- Old workflows continue functioning unchanged

**Commit:** test(templates): add end-to-end integration tests for Handlebars workflow

---

## Final Validation

After all steps complete:

- All tests pass: cargo test
- Release build succeeds: cargo build --release
- Code coverage remains above 80 percent threshold
- Both template engines coexist without conflicts
- Proof-of-concept code_map partial demonstrates value
- Documentation (SPEC, PLAN) reflects actual implementation
- No breaking changes to existing functionality

---

## Notes

**Build Protocol:**
- Use build.sh --install --skip-bump for testing between steps
- Only final commit gets version bump with build.sh --install

**Test Philosophy:**
- Tests express intent, not implementation details
- Use test helpers for expressive DSL
- Keep tests focused on interfaces and contracts
- Avoid testing Handlebars library internals

**Security:**
- Reuse existing validate_guide_name from template.rs
- No new attack vectors introduced
- Path traversal protection maintained

**Migration Strategy:**
- Old system unchanged and fully functional
- New system introduced alongside, not as replacement
- Gradual adoption possible, no forced migration
- code_map serves as reference for future migrations
