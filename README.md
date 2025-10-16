<p align="center">
  <img src="hegel.png" alt="Hegel Logo" width="200">
</p>

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
- `research` - External knowledge gathering (PLAN → STUDY → ASSESS → QUESTIONS)
- `discovery` - Optimized for learning density (SPEC → PLAN → CODE → LEARNINGS → README)
- `execution` - Optimized for production delivery
- `minimal` - Simplified workflow for quick iterations

### Advancing Through Phases

Hegel provides ergonomic commands for common workflow transitions:

```bash
# Happy path: advance to next phase
hegel next

# Repeat current phase (e.g., after addressing feedback)
hegel repeat

# Restart workflow cycle (return to SPEC phase)
hegel restart
```

**Advanced usage:**

For custom transitions, provide explicit claims:

```bash
hegel next '{"spec_complete": true}'
hegel next '{"custom_claim": true}'
```

The `next` command automatically infers the happy-path claim (`{"{current_phase}_complete": true}`) when no claim is provided

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

Hegel uses YAML-based workflow definitions to guide you through development cycles. State is stored locally in `.hegel/state.json` (in your current working directory), making it a fully offline tool with no API keys or external dependencies required.

Each workflow defines:
- **Nodes** - Development phases with specific prompts
- **Transitions** - Rules for moving between phases based on claims
- **Mode** - Discovery (exploration) or Execution (delivery)

## Dialectic-Driven Development

Hegel implements the Dialectic-Driven Development methodology across three operational modes:

### Research Mode (External Knowledge Gathering)
1. **Plan** - Define study priorities and scope
2. **Study** - Cache external sources, synthesize into learning docs
3. **Assess** - Meta-cognitive reflection on progress
4. **Questions** - Catalogue open questions as Discovery roadmap

**Use for**: Pre-implementation knowledge building, unfamiliar domains, systematic study of external sources

### Discovery Mode (Toy Experiments)
1. **Docs** - Generate SPEC.md and PLAN.md
2. **Tests** - Derive executable tests from specifications
3. **Implementation** - Minimal code to pass tests
4. **Learnings** - Extract insights and architectural decisions

**Use for**: Validating uncertainties, prototype implementations, answering questions from Research

### Execution Mode (Production Delivery)
Similar cycle to Discovery but with production-grade rigor, comprehensive error handling, and mandatory code review phase.

**Use for**: Building production features with validated patterns from Discovery

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

All state is stored in `.hegel/state.json` (current working directory) with atomic writes to prevent corruption. The state file contains:
- Current workflow definition
- Current node/phase
- Navigation history
- Workflow mode
- Unique workflow ID (ISO 8601 timestamp)
- Session metadata (session ID, transcript path, timestamp)

### Configurable State Directory

By default, Hegel uses `.hegel/` in the current working directory for state storage. You can override this:

**Via command-line flag:**
```bash
hegel --state-dir /tmp/my-project start discovery
```

**Via environment variable:**
```bash
export HEGEL_STATE_DIR=/tmp/my-project
hegel start discovery
```

**Precedence:** CLI flag > environment variable > default (`.hegel/` in cwd)

**Use cases:**
- **Testing:** Isolate test runs in temporary directories
- **Multi-project workflows:** Override the default per-project state location
- **CI/CD:** Configure non-default state locations in automated environments

**Note:** The default behavior (`.hegel/` in current working directory) ensures state is session-local and project-specific. Each project directory gets its own workflow state, aligning with the design philosophy that sessions and workflows are coupled to the working directory where Claude Code is running.

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
- **Workflow graph** - ASCII visualization of phase transitions:
  - Node metrics (visits, tokens, duration, bash commands, file edits)
  - Cycle detection (identifies workflow loops)
  - DOT export instructions for Graphviz diagrams

### Interactive Dashboard

Launch a real-time TUI dashboard:

```bash
hegel top
```

**Features:**
- **4 interactive tabs**: Overview, Phases, Events, Files
- **Live updates**: Auto-reloads when `.hegel/*.jsonl` files change
- **Scrolling**: Arrow keys, vim bindings (j/k), jump to top/bottom (g/G)
- **Navigation**: Tab/BackTab to switch tabs
- **Colorful UI**: Emoji icons, syntax highlighting, status indicators

**Keyboard shortcuts:**
- `q` - Quit
- `Tab` / `BackTab` - Navigate tabs
- `↑↓` / `j`/`k` - Scroll
- `g` / `G` - Jump to top/bottom
- `r` - Reload metrics manually

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

## Command Wrapping with Guardrails

Hegel can wrap dangerous commands with safety guardrails and audit logging. Commands are configured in `.hegel/guardrails.yaml` and automatically become available as `hegel <command>`.

### Configuration

Create `.hegel/guardrails.yaml`:

```yaml
git:
  blocked:
    - pattern: "clean -fd"
      reason: "Destructive: removes untracked files/directories"
    - pattern: "reset --hard"
      reason: "Destructive: permanently discards uncommitted changes"
    - pattern: "commit.*--no-verify"
      reason: "Bypasses pre-commit hooks"
    - pattern: "push.*--force"
      reason: "Force push can overwrite remote history"

docker:
  blocked:
    - pattern: "rm -f"
      reason: "Force remove containers blocked"
    - pattern: "system prune -a"
      reason: "Destructive: removes all unused containers, networks, images"
```

### Usage

```bash
# Run git through Hegel's guardrails
hegel git status           # ✓ Allowed
hegel git reset --hard     # ✗ Blocked with reason

# Run docker through guardrails
hegel docker ps            # ✓ Allowed
hegel docker rm -f my-container  # ✗ Blocked

# All commands are logged to .hegel/command_log.jsonl
cat .hegel/command_log.jsonl
```

**Features:**
- **Regex-based blocking** - Pattern match against command arguments
- **Audit logging** - All invocations logged with timestamp, success/failure, and block reason
- **No interactive prompts** - Hard blocks only (agents can't handle prompts)
- **Extensible** - Add any command to `guardrails.yaml` (currently supports: `git`, `docker`)

**When blocked**, Hegel exits with code 1 and prints:
- The blocked command
- The reason from guardrails.yaml
- Path to edit rules

## Advanced Tools

### AST-based Code Transformation

Search and rewrite code using AST patterns (wraps `ast-grep`):

```bash
# Search for patterns
hegel astq -p 'pub fn $FUNC' src/

# Replace code patterns
hegel astq -p 'println!($X)' -r 'log::info!($X)' src/

# Show help
hegel astq --help
```

**Powered by [ast-grep](https://github.com/ast-grep/ast-grep)**, a fast AST-based search and rewrite tool. First run automatically builds from vendor.

### Markdown Document Review

Launch ephemeral GUI for reviewing Markdown artifacts:

```bash
# Single file review
hegel reflect SPEC.md

# Multiple files
hegel reflect SPEC.md PLAN.md

# With output directory
hegel reflect SPEC.md --out-dir .reviews/

# Headless mode (testing)
hegel reflect SPEC.md --headless
```

**Powered by [mirror](https://github.com/dialecticianai/hegel-mirror)**, a zero-friction Markdown review UI. Requires `mirror` binary built and available (adjacent repo or in PATH).

**Review workflow:**
- Select text → comment → submit
- Comments saved to `.ddd/<filename>.review.N`
- Auto-exit on submit (like `git commit`)
- Session ID passthrough via `HEGEL_SESSION_ID`

## Contributing

For AI agents or developers working **on** Hegel (not just using it), see **`CLAUDE.md`** for project structure, development guidelines, and contribution workflow.

## License

Server Side Public License v1 (SSPL)

## Learn More

Visit [dialectician.ai](https://dialectician.ai) for more information about Dialectic-Driven Development.
