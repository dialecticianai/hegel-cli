# src/commands/workflow/

Workflow orchestration command implementations. Handles workflow lifecycle: starting, advancing, repeating, restarting, and aborting workflows.

## Purpose

Implements the workflow command surface (start, next, prev, repeat, restart, abort, status, stash operations) by coordinating the engine, storage, and rules layers. Evaluates transitions, renders prompts with guide injection, and maintains workflow state. Supports saving and restoring workflow snapshots via stash commands.

## Structure

```
workflow/
├── mod.rs               Command handlers (start, next with force_bypass, prev, repeat, restart, abort, status, reset, stash)
│                        Stash operations: stash_workflow, list_stashes, pop_stash, drop_stash
├── claims.rs            ClaimAlias type (Next/Repeat/Restart/Custom claim transformations)
├── context.rs           WorkflowContext (loading), render_node_prompt (dual-engine routing), display_workflow_prompt
├── transitions.rs       Transition evaluation and execution with force_bypass support (Stay/IntraWorkflow/InterWorkflow/Ambiguous)
│
└── tests/               Modular test structure
    ├── mod.rs           Shared test helpers
    ├── commands.rs      Command tests (start, next, repeat, reset, status, restart)
    ├── stash.rs         Stash command tests (save, list, pop, drop workflows)
    ├── transitions.rs   Transition and state logging tests
    ├── integration.rs   End-to-end workflow tests
    ├── production.rs    Production workflow validation
    └── node_flow.rs     Node flow extraction tests
```

## Key Concepts

**Claims**: Simple string assertions that trigger transitions (e.g., "spec_complete", "needs_refactor")
**Context**: Workflow definition + current state + guide/template resolution
**Transitions**: Four outcomes: Stay (no match), IntraWorkflow (same workflow), InterWorkflow (switch workflows), Ambiguous (multiple matches)
**Prompt Rendering**: Dual-engine system - Markdown ({{GUIDE}}) or Handlebars ({{> partial}}) based on is_handlebars state flag
