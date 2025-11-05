# src/guardrails/

Command safety layer. Pre-execution guardrails for wrapped commands (git, docker) with pattern-based blocking and audit logging.

## Purpose

Provides safety guardrails for dangerous commands by evaluating patterns against command arguments before execution. Prevents destructive operations (force push, hard reset, etc.) with clear explanations. All command invocations are logged for audit trails.

## Structure

```
guardrails/
├── mod.rs               Module exports (load, evaluate)
├── parser.rs            Load guardrails.yaml, parse blocked/allowed patterns
└── types.rs             GuardRailsConfig, CommandGuardrails, RuleMatch (Allowed/Blocked/NoMatch)
```

## How It Works

1. **Configuration**: `.hegel/guardrails.yaml` defines per-command blocked patterns with reasons
2. **Evaluation**: Regex patterns matched against full command string
3. **Action**: Blocked commands exit with code 1 and display reason; allowed commands execute normally
4. **Logging**: All invocations logged to `.hegel/command_log.jsonl` with timestamp and outcome

## Example Configuration

```yaml
git:
  blocked:
    - pattern: "reset --hard"
      reason: "Destructive: permanently discards uncommitted changes"
```

Used by `commands/wrapped.rs` for `hegel git`, `hegel docker`, etc.
