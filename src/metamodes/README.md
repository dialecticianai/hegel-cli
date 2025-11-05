# src/metamodes/

Meta-mode orchestration. Defines higher-level development patterns (learning vs standard) that coordinate workflow transitions.

## Purpose

Meta-modes are workflow-of-workflows patterns. They define macro-level development strategies:
- **Learning**: Research ↔ Discovery loop (greenfield learning projects)
- **Standard**: Discovery ↔ Execution loop (feature development with known patterns)

Meta-modes enable automatic workflow selection based on completion conditions, supporting more sophisticated development patterns than single workflows allow.

## Structure

```
metamodes/
└── mod.rs               MetaModeDefinition (learning/standard), transition evaluation, workflow completion detection
```

## Key Concepts

**Meta-Mode**: High-level pattern coordinating multiple workflows
**Completion Detection**: Logic to determine when a workflow cycle completes (e.g., discovery workflow reaches "readme" node)
**Transition Rules**: Which workflow to start next based on current workflow completion
**Optional**: Meta-modes are opt-in via `hegel meta <name>` - workflows work independently without them
