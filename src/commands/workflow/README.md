# src/commands/workflow/

Workflow orchestration command implementations. Handles workflow lifecycle: starting, advancing, repeating, restarting, and aborting workflows.

## Purpose

Implements the workflow command surface (start, next, prev, repeat, restart, abort, status) by coordinating the engine, storage, and rules layers. Evaluates transitions, renders prompts with guide injection, and maintains workflow state.

## Structure

```
workflow/
├── mod.rs               Command handlers (start, next, prev, repeat, restart, abort, status, reset)
├── claims.rs            ClaimAlias type (Next/Repeat/Restart/Custom claim transformations)
├── context.rs           WorkflowContext (loading, prompt rendering with guide injection)
├── transitions.rs       Transition evaluation and execution (Stay/IntraWorkflow/InterWorkflow/Ambiguous)
│
└── tests/               Modular test structure
    ├── mod.rs           Shared test helpers
    ├── commands.rs      Command tests (start, next, repeat, reset, status, restart)
    ├── transitions.rs   Transition and state logging tests
    ├── integration.rs   End-to-end workflow tests
    ├── production.rs    Production workflow validation
    └── node_flow.rs     Node flow extraction tests
```

## Key Concepts

**Claims**: Simple string assertions that trigger transitions (e.g., "spec_complete", "needs_refactor")
**Context**: Workflow definition + current state + guide/template resolution
**Transitions**: Four outcomes: Stay (no match), IntraWorkflow (same workflow), InterWorkflow (switch workflows), Ambiguous (multiple matches)
**Prompt Rendering**: Template expansion with {{GUIDE}} injection and {{?optional}} support
