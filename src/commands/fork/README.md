# src/commands/fork/

External agent orchestration. Detects available agent CLIs and provides adapters for delegating tasks to external agents.

## Purpose

Enables multi-agent workflows by discovering installed agent CLIs (Claude, Aider, Codex, Gemini, Cody, Amp) and providing a uniform interface for task delegation. Currently implements agent detection; execution support planned for Phase 2.2.

## Structure

```
fork/
├── mod.rs               Agent detection and display (detect_agents, display_agents, handle_fork)
├── runtime.rs           Runtime version management (Node.js/Python checking, nvm compatibility, execute_agent)
│
├── amp.rs               Amp agent adapter (build_args for Sourcegraph Amp)
├── codex.rs             Codex agent adapter (codex exec, passthrough args)
├── cody.rs              Cody agent adapter (stdin support, passthrough args)
├── gemini.rs            Gemini agent adapter (positional args, -o json support)
└── generic.rs           Generic fallback for unknown agents
```

## Agent Adapters

Each adapter provides:
- **Detection**: Check if agent binary exists in PATH
- **Version Requirements**: Node.js 20+ for Gemini/Cody/Amp
- **Argument Building**: Translate Hegel invocations to agent-specific CLI args

Supported agents: `claude`, `aider`, `copilot`, `codex`, `gemini`, `cody`, `amp`
