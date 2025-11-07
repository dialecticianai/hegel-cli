# Handlebars Template Engine Specification

Add Handlebars templating as a parallel system alongside existing Markdown template engine, enabling conditional logic, loops, and partials for workflow prompts and guides.

---

## Overview

**What it does:** Introduces Handlebars template support for `.hbs` files while preserving existing `.md` template functionality. Both engines coexist, routing based on file extension and YAML field discrimination.

**Key principles:**
- **Non-breaking**: Existing `.md` templates continue working unchanged
- **Gradual migration**: Migrate templates one-by-one, starting with proof-of-concept
- **Standard syntax**: Use vanilla Handlebars (no custom extensions)
- **File precedence**: `.hbs` partials take priority over `.md` guides for same base name

**Scope:**
- Add Handlebars engine for `.hbs` files
- Support `prompt_hbs:` field in workflow YAML
- Create `guides/partials/` directory for Handlebars templates
- Migrate code_map templates as proof-of-concept (2 `.md` files → 1 `.hbs` with conditionals)

**Integration context:**
- `src/engine/template.rs` - Keep existing (handles `.md` files)
- `src/engine/handlebars.rs` - NEW (handles `.hbs` files)
- `src/engine/mod.rs` - Export both engines, route based on context
- Workflow YAML parser needs to handle both `prompt:` and `prompt_hbs:` fields

---

## Data Model

### Modified: `src/engine/mod.rs`

Add public routing function for engine selection:

```rust
/// Render template using appropriate engine based on extension
/// - .md files → custom template engine (existing)
/// - .hbs files → Handlebars engine (new)
pub fn render_prompt(
    template_content: &str,
    is_handlebars: bool,
    guides_dir: &Path,
    context: &HashMap<String, String>,
) -> Result<String>
```

**Behavior:**
- If `is_handlebars == false`: delegate to existing `template::render_template()`
- If `is_handlebars == true`: delegate to new `handlebars::render_template_hbs()`

### New: `src/engine/handlebars.rs`

```rust
use handlebars::Handlebars;
use std::path::Path;
use std::collections::HashMap;
use anyhow::Result;

/// Render Handlebars template with guide/partial loading
pub fn render_template_hbs(
    template: &str,
    guides_dir: &Path,
    context: &HashMap<String, String>,
) -> Result<String>
```

**Responsibilities:**
- Initialize Handlebars registry
- Register partials from `guides/partials/*.hbs` (precedence) and `guides/*.hbs` (fallback)
- Register helpers (if needed - start with none, vanilla Handlebars only)
- Validate guide names for security (no path traversal)
- Render template with context

**Partial loading order:**
1. Check `guides/partials/{name}.hbs` first
2. Fallback to `guides/{name}.hbs`
3. Check embedded files (TBD: mirror existing embedded guide system)
4. Error if not found

**Security:**
- Reuse existing `validate_guide_name()` logic from `template.rs`
- Reject path traversal (`..`), absolute paths, nested subdirectories
- Allow only: `partials/{name}` where `{name}` is alphanumeric + underscores

### Modified: `src/engine/mod.rs::Node`

Already has `prompt: String` field. No struct change needed - YAML parser handles routing.

### Modified: Workflow YAML schema

Support dual prompt fields (exactly one must be present):

```yaml
nodes:
  spec:
    prompt: "Old style {{GUIDE}} template"
    transitions: [...]

  plan:
    prompt_hbs: "New style {{> guides/GUIDE}} template"
    transitions: [...]
```

**Validation:**
- Error if both `prompt` and `prompt_hbs` present in same node
- Error if neither present (except `done` node)

### New: `guides/partials/code_map.hbs`

Single Handlebars template replacing two Markdown files:

```handlebars
{{#if (eq code_map_style "hierarchical")}}
## Structure for Larger Projects (>50 files)
...hierarchical content...
{{else}}
## Structure for Small Projects (<50 files)
...monolithic content...
{{/if}}
```

**Note:** Requires `eq` helper for string comparison. Add minimal custom helper:

```rust
handlebars.register_helper("eq", Box::new(eq_helper));

fn eq_helper(/* ... */) -> HelperResult {
    // Compare two values for equality
}
```

---

## Core Operations

### Load Handlebars Partials

**Syntax:** `handlebars::register_partials(registry, guides_dir)`

**Parameters:**
- `registry: &mut Handlebars` - Handlebars instance
- `guides_dir: &Path` - Base directory for guides

**Behavior:**
1. Scan `guides/partials/*.hbs` files
2. Register each as partial with name = stem (e.g., `code_map.hbs` → `"code_map"`)
3. Scan `guides/*.hbs` files (if any) as fallback
4. Validate all names for security

**Example:**
```rust
let mut hbs = Handlebars::new();
register_partials(&mut hbs, Path::new("guides"))?;
```

### Render Handlebars Template

**Syntax:** `handlebars::render_template_hbs(template, guides_dir, context)`

**Parameters:**
- `template: &str` - Template content (from YAML `prompt_hbs` field)
- `guides_dir: &Path` - Base directory for partials
- `context: &HashMap<String, String>` - Context variables

**Behavior:**
1. Initialize Handlebars registry
2. Register partials from guides directories
3. Register custom helpers (`eq` for conditionals)
4. Render template with context
5. Return rendered string

**Example:**
```rust
let context = HashMap::from([
    ("code_map_style", "hierarchical"),
]);
let result = render_template_hbs(
    "{{> partials/code_map}}",
    Path::new("guides"),
    &context
)?;
```

### Workflow YAML Parsing (Modified)

**Current behavior:** Parse `prompt:` field → render with old engine

**New behavior:**
- If `prompt:` present → render with `template::render_template()`
- If `prompt_hbs:` present → render with `handlebars::render_template_hbs()`
- Error if both present

**Location:** `src/engine/mod.rs::get_next_prompt()` or YAML deserialization

---

## Test Scenarios

### Simple: Handlebars Basic Rendering

**Setup:** Single `.hbs` file in `guides/partials/test.hbs`:
```handlebars
Hello {{name}}!
```

**Test:**
```rust
let context = HashMap::from([("name", "World")]);
let result = render_template_hbs("{{> partials/test}}", guides_dir, &context)?;
assert_eq!(result, "Hello World!");
```

### Simple: Conditional Rendering

**Setup:** `guides/partials/greeting.hbs`:
```handlebars
{{#if formal}}
Good day, {{name}}.
{{else}}
Hey {{name}}!
{{/if}}
```

**Test:**
```rust
let context = HashMap::from([("formal", "true"), ("name", "Alice")]);
let result = render_template_hbs("{{> partials/greeting}}", guides_dir, &context)?;
assert!(result.contains("Good day, Alice."));
```

### Complex: Code Map Migration

**Setup:**
- Old: `guides/templates/code_map_{hierarchical,monolithic}.md` (2 files)
- New: `guides/partials/code_map.hbs` (1 file with `{{#if}}`)

**Test:**
```rust
// Hierarchical mode
let ctx = HashMap::from([("code_map_style", "hierarchical")]);
let result = render_template_hbs("{{> partials/code_map}}", guides_dir, &ctx)?;
assert!(result.contains("Larger Projects"));

// Monolithic mode
let ctx = HashMap::from([("code_map_style", "monolithic")]);
let result = render_template_hbs("{{> partials/code_map}}", guides_dir, &ctx)?;
assert!(result.contains("Small Projects"));
```

### Complex: Precedence Testing (HBS over MD)

**Setup:**
- `guides/partials/code_map.hbs` - Contains "HANDLEBARS VERSION"
- `guides/templates/code_map_hierarchical.md` - Contains "MARKDOWN VERSION"

**Test:**
```rust
// When both .hbs partial and .md template exist, .hbs wins
let result = render_template_hbs("{{> partials/code_map}}", guides_dir, &context)?;
assert!(result.contains("HANDLEBARS VERSION"));
assert!(!result.contains("MARKDOWN VERSION"));
```

**Purpose:** Validates that the new system takes precedence, allowing gradual migration without breaking existing references.

### Complex: Dual Engine Coexistence

**Setup:**
- Node A uses `prompt:` (old .md system)
- Node B uses `prompt_hbs:` (new .hbs system)

**Test:**
```rust
// Old engine still works
let old_prompt = "{{SPEC_WRITING}}";
let result = render_template(old_prompt, guides_dir, &context)?;
assert!(result.contains("# SPEC Writing Guide"));

// New engine works
let new_prompt = "{{> partials/test}}";
let result = render_template_hbs(new_prompt, guides_dir, &context)?;
assert!(result.contains("Hello"));
```

### Error: Path Traversal Rejected

**Test:**
```rust
let result = render_template_hbs("{{> ../../../etc/passwd}}", guides_dir, &context);
assert!(result.is_err());
assert!(result.unwrap_err().to_string().contains("Path traversal"));
```

### Error: Both prompt and prompt_hbs

**Test:**
```yaml
nodes:
  invalid:
    prompt: "Old"
    prompt_hbs: "New"
```

Should error during workflow validation: "Node 'invalid' cannot have both prompt and prompt_hbs fields"

---

## Success Criteria

Tests pass:
- `cargo test` - All existing tests continue passing (no regressions)
- New tests in `src/engine/handlebars.rs::tests` module pass:
  - Basic rendering with partials
  - Conditional logic (`{{#if}}`)
  - String equality helper (`{{#if (eq x "value")}}`)
  - Partial load precedence (`partials/*.hbs` > `guides/*.hbs`)
  - Security validation (path traversal rejection)
- Integration test: Load workflow with `prompt_hbs:` field, render successfully

Build succeeds:
- `cargo build --release` - No compilation errors
- `handlebars` dependency added to Cargo.toml

Code map migration complete:
- `guides/partials/code_map.hbs` exists with conditional logic
- Old files (`guides/templates/code_map_{hierarchical,monolithic}.md`) remain for precedence testing
- Test verifies that `.hbs` partial loads instead of `.md` template when both exist
- `CODE_MAP_WRITING.md` reference updated to use new partial (or keep old for now)

Proof-of-concept demo:
- Create test workflow node with `prompt_hbs: "{{> partials/code_map}}"`
- Render with context `code_map_style: hierarchical`
- Verify correct conditional branch loads

---

## Out of Scope

**Deferred to future work:**
- Migrating all existing `.md` guides to `.hbs` (Phase 1.0 = proof-of-concept only)
- Custom Handlebars helpers beyond `eq` (loops, advanced logic - add as needed)
- Embedded `.hbs` files (start with filesystem only, mirror later if needed)
- Deprecating old `.md` engine (keep indefinitely for backward compatibility)
- Performance optimization (both engines acceptable for current scale)
- Handlebars template precompilation (premature optimization)

**Explicitly not doing:**
- Removing or modifying existing `src/engine/template.rs`
- Changing existing workflow YAML files (except proof-of-concept demo)
- Migrating guides beyond code_map templates
- Adding custom Handlebars syntax extensions (vanilla only)

---

## Migration Path (Post-Phase 1.0)

Once proof-of-concept validates the approach:

1. **Phase 1.1:** Migrate remaining `guides/templates/*.md` to `guides/partials/*.hbs`
2. **Phase 1.2:** Migrate main guides (`SPEC_WRITING.md` → `SPEC_WRITING.hbs`) where conditionals add value
3. **Phase 1.3:** Update workflow YAML files to use `prompt_hbs:` for migrated guides
4. **Phase 1.4:** (Optional) Deprecate old engine if all guides migrated

**Decision point:** Only migrate guides where Handlebars adds value (conditionals, loops). Simple guides can stay `.md` forever - no need for perfection.
