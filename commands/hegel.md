---
description: Run Hegel CLI commands for workflow orchestration
---

# Hegel Workflow Orchestration

Execute Hegel commands to manage Dialectic-Driven Development workflows.

```bash
hegel $ARGUMENTS
```

This command provides access to all Hegel CLI functionality:

- `hegel start <workflow>` - Start a new workflow
- `hegel next` - Advance to the next workflow step
- `hegel prev` - Go back to the previous workflow step
- `hegel status` - Check current workflow state
- `hegel reset` - Reset workflow state
- `hegel analyze` - Analyze metrics and telemetry
- `hegel hook <event>` - Record hook events
- `hegel astq` - AST-aware code search

Run without arguments to see the current workflow state and available commands.
