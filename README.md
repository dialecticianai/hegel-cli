# Hegel - Dialectic-Driven Development CLI

**Thesis. Antithesis. Synthesis.**

A command-line tool for orchestrating Dialectic-Driven Development workflows. Hegel guides you through structured development cycles with state-based workflow management.

**Designed for AI agents, ergonomic for humans.** Hegel is primarily an agent-facing CLI that provides deterministic workflow guardrails for AI-assisted development. It's also comfortable for direct human use.

## Installation

```bash
cargo build --release
```

The binary will be available at `./target/release/hegel`.

## Usage

### Starting a Workflow

Start a new workflow (discovery or execution mode):

```bash
hegel start discovery
```

Available workflows:
- `discovery` - Optimized for learning density (SPEC → PLAN → CODE → LEARNINGS → README)
- `execution` - Optimized for production delivery
- `minimal` - Simplified workflow for quick iterations

### Advancing Through Phases

Transition to the next phase by providing claims:

```bash
hegel next '{"spec_complete": true}'
```

Common claims:
- `spec_complete` - SPEC phase finished
- `plan_complete` - PLAN phase finished
- `code_complete` - Implementation finished
- `learnings_complete` - LEARNINGS documented
- `restart_cycle` - Return to SPEC phase

### Checking Status

View your current workflow position:

```bash
hegel status
```

Shows:
- Current mode (discovery/execution)
- Current node/phase
- Full history of nodes visited

### Resetting State

Clear all workflow state:

```bash
hegel reset
```

## How It Works

Hegel uses YAML-based workflow definitions to guide you through development cycles. State is stored locally in `~/.hegel/state.json`, making it a fully offline tool with no API keys or external dependencies required.

Each workflow defines:
- **Nodes** - Development phases with specific prompts
- **Transitions** - Rules for moving between phases based on claims
- **Mode** - Discovery (exploration) or Execution (delivery)

## Dialectic-Driven Development

Hegel implements the Dialectic-Driven Development methodology:

1. **Docs** - Generate or update SPEC.md and PLAN.md
2. **Tests** - Derive executable tests from specifications
3. **Implementation** - Minimal code to pass tests
4. **Learnings** - Extract insights and architectural decisions

This methodology treats artifacts as disposable fuel while preserving clarity and constraints as durable value.

### When to Use DDD Workflows

**Hegel is a general workflow orchestration tool.** The DDD-opinionated guides included in this project (SPEC_WRITING, PLAN_WRITING, etc.) are defaults, not requirements.

**Use full DDD workflows for:**
- Hard problems requiring novel solutions
- Projects needing extremely rigorous documentation
- Complex domains where mistakes are expensive
- Learning-dense exploration (discovery mode)

**Skip DDD overhead for:**
- Straightforward implementations agents can handle autonomously
- Simple CRUD applications or routine features
- Projects where the agent doesn't need structured guidance

The workflow steps and accompanying token usage are designed for problems that **need** that rigor. Many projects don't.

## Project Structure

```
hegel-cli/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── commands/        # Command implementations
│   ├── engine/          # Workflow state machine
│   └── storage/         # File-based state persistence
├── workflows/           # YAML workflow definitions
└── guides/              # Writing guides for documentation
```

## State Storage

All state is stored in `~/.hegel/state.json` with atomic writes to prevent corruption. The state file contains:
- Current workflow definition
- Current node/phase
- Navigation history
- Workflow mode
- Unique workflow ID (ISO 8601 timestamp)

### Configurable State Directory

By default, Hegel uses `~/.hegel/` for state storage. You can override this:

**Via command-line flag:**
```bash
hegel --state-dir /tmp/my-project start discovery
```

**Via environment variable:**
```bash
export HEGEL_STATE_DIR=/tmp/my-project
hegel start discovery
```

**Precedence:** CLI flag > environment variable > default (`~/.hegel/`)

**Use cases:**
- **Testing:** Isolate test runs in temporary directories
- **Multiple projects:** Run separate workflow contexts simultaneously
- **CI/CD:** Configure non-default state locations in automated environments

## Claude Code Integration

Hegel integrates with [Claude Code](https://claude.com/claude-code) to capture development activity as you work. This enables metrics collection and workflow analysis.

### Hook Events

The `hegel hook` command processes Claude Code hook events:

```bash
# Typically configured in .claude/settings.json
hegel hook PostToolUse < event.json
```

Hook events are logged to `.hegel/hooks.jsonl` with timestamps. Each workflow session is assigned a unique `workflow_id` (ISO 8601 timestamp) when you run `hegel start`, enabling correlation between workflow phases and development activity.

### Analyzing Metrics

View captured development activity and metrics:

```bash
hegel analyze
```

Shows:
- Session ID and workflow summary
- Token usage (input/output/cache metrics from transcripts)
- Activity summary (bash commands, file modifications)
- Top commands and most-edited files
- Workflow state transitions
- **Phase breakdown** - Per-phase metrics including:
  - Duration (time spent in each phase)
  - Token usage (input/output tokens per phase)
  - Activity (bash commands and file edits per phase)
  - Status (active or completed)

**Coming soon:**
- `hegel top` - Interactive TUI dashboard with live metrics

**What's tracked:**
- Tool usage (Bash, Read, Edit, Write, etc.)
- File modifications with frequency counts
- Workflow state transitions (logged to states.jsonl)
- Token usage from Claude Code transcripts
- Per-phase metrics (duration, tokens, commands, file edits)
  - Correlated via timestamps across hooks.jsonl, states.jsonl, and transcripts
  - Enables budget enforcement rules in Phase 2 (cycle detection)

**Configuration:**

See `.claude/settings.json` in this repository for an example hook configuration. Hook events are optional—Hegel works without them, but metrics features require hook data.

## Contributing

For AI agents or developers working **on** Hegel (not just using it), see **`CLAUDE.md`** for project structure, development guidelines, and contribution workflow.

## License

Server Side Public License v1 (SSPL)

## Learn More

Visit [dialectician.ai](https://dialectician.ai) for more information about Dialectic-Driven Development.
