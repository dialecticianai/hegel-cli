# src/adapters/

Multi-agent support layer. Normalizes hook events from different agent CLIs (Claude Code, Cursor, Codex) to a canonical format for unified metrics collection.

## Purpose

Different agent CLIs emit hook events in different formats. Adapters provide a uniform interface through the `AgentAdapter` trait, enabling Hegel to work with multiple agents without code changes.

## Structure

```
adapters/
├── mod.rs              AgentAdapter trait, CanonicalHookEvent schema, AdapterRegistry, transcript discovery exports
├── claude_code.rs      Claude Code adapter (env detection, event normalization, transcript file discovery)
├── cursor.rs           Cursor adapter (future multi-agent support)
└── codex.rs            Codex adapter (future multi-agent support)
```

## How It Works

1. **Detection**: Each adapter implements environment detection (e.g., checking for Claude-specific env vars)
2. **Normalization**: Raw JSON hook events are parsed and converted to `CanonicalHookEvent`
3. **Registration**: `AdapterRegistry` automatically selects the correct adapter based on environment

This allows `hegel hook` to accept events from any supported agent and write them to `.hegel/hooks.jsonl` in a consistent format.
