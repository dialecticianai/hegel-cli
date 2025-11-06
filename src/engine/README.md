# src/engine/

Layer 2: Workflow state machine and template rendering. Defines workflow structure, evaluates transitions, and renders prompts with guide injection.

## Purpose

The engine is the core workflow orchestrator. It loads YAML workflow definitions, maintains workflow state, evaluates transition rules to determine the next phase, and renders prompts by injecting guides and templates.

## Structure

```
engine/
├── mod.rs               Workflow/Node/Transition structs, load_workflow, init_state, get_next_prompt, render_prompt (dual-engine router)
├── handlebars.rs        Handlebars template engine (partials, eq helper, HandlebarsContext wrapping)
└── template.rs          Markdown template rendering ({{UPPERCASE}} guides, {{templates/name}} includes, {{var}} context, recursive expansion)
```

## Key Concepts

**Workflow**: Collection of nodes with start_node and mode (discovery/execution)
**Node**: Workflow phase with prompt/prompt_hbs and list of possible transitions
**Transition**: Rule for moving between nodes based on claims (when + to)
**Dual Template Engines**: Markdown (.md) and Handlebars (.hbs) coexist - routed by prompt field and is_handlebars state
**Template Expansion**: Recursive placeholder replacement (Markdown) or Handlebars rendering with partials/conditionals

## Template Syntax

**Markdown templates (prompt field):**
- `{{GUIDE_NAME}}` - Required guide injection (errors if missing)
- `{{?optional_guide}}` - Optional guide (empty string if missing)
- `{{templates/name}}` - Include reusable template fragment
- `{{var}}` - Context variable substitution

**Handlebars templates (prompt_hbs field):**
- `{{> partial_name}}` - Include partial from guides/partials/ or guides/
- `{{context.variable}}` - Context variable access (wrapped in HandlebarsContext)
- `{{#if (eq context.var "value")}}` - Conditional rendering with custom eq helper
- `{{#each items}}` - Iteration (standard Handlebars)
