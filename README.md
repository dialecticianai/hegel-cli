<p align="center">
  <img src="hegel.png" alt="Hegel Logo" width="200">
</p>

# Hegel - Dialectic-Driven Development CLI

**Thesis. Antithesis. Synthesis.**

A command-line tool for orchestrating Dialectic-Driven Development workflows. Hegel guides you through structured development cycles with state-based workflow management.

**Designed for AI agents, ergonomic for humans.** Hegel is primarily an agent-facing CLI that provides deterministic workflow guardrails for AI-assisted development. It's also comfortable for direct human use.

**A swiss army knife for agents**—everything they need to keep themselves on track: workflow state machines, metrics tracking, safety guardrails, AST-based code search, document review, and external agent orchestration.

## Installation

```bash
cargo build --release
```

The binary will be available at `./target/release/hegel`.

## Usage

### Project Initialization

Bootstrap a new DDD project or retrofit DDD to an existing codebase:

```bash
hegel init
```

**What it does:**
- Automatically detects project type (greenfield vs retrofit)
- **Greenfield** (no non-.md files): Guides you through creating CLAUDE.md, VISION.md, and ARCHITECTURE.md
- **Retrofit** (existing code): Analyzes project structure and creates code maps in README.md files
- Walks through initialization workflow with prompts for each phase

**Configuration:**

Customize initialization behavior:

```bash
# View all settings
hegel config list

# Get specific setting
hegel config get code_map_style

# Set code map style (monolithic or hierarchical)
hegel config set code_map_style hierarchical
hegel config set code_map_style monolithic

# Toggle auto-launching reflect GUI after doc generation
hegel config set use_reflect_gui true
hegel config set use_reflect_gui false
```

Configuration is persisted to `.hegel/config.toml`.

### Declaring a Meta-Mode (Optional)

Optionally declare which meta-mode pattern you're following:

```bash
# For greenfield learning projects
hegel meta learning

# For standard feature development
hegel meta standard
```

Meta-modes enable higher-level workflow orchestration patterns but are not required for basic workflow usage.

**Available meta-modes:**
- `learning` - Greenfield learning project (Research ↔ Discovery loop, starts with research)
- `standard` - Feature development with known patterns (Discovery ↔ Execution, starts with discovery)

**View current meta-mode:**
```bash
hegel meta
```

### Starting Workflows

Start and transition between workflows:

```bash
# Start workflow at default beginning
hegel start discovery  # Starts at 'spec' phase
hegel start execution  # Starts at workflow's default start_node

# Start workflow at specific phase (useful for resuming or testing)
hegel start discovery plan     # Skip spec, start directly at plan
hegel start execution code     # Start directly at code phase
hegel start research study     # Start directly at study phase
```

**Custom start nodes:**
- Skips earlier phases in the workflow
- Useful for resuming interrupted workflows or testing specific phases
- History will only show nodes from the custom start point forward
- Validates node exists and provides helpful error with available nodes

Available workflows:
- `cowboy` - **DEFAULT** - Minimal overhead for straightforward tasks
- `init-greenfield` - Initialize new DDD project
- `init-retrofit` - Add DDD to existing project
- `research` - External knowledge gathering
- `discovery` - Optimized for learning density
- `execution` - Optimized for production delivery
- `refactor` - Focused refactoring workflow

### Listing Available Resources

Discover what workflows and guides are available:

```bash
# List all workflows (embedded + user-defined)
hegel workflows

# List all guides (embedded + user-defined)
hegel guides
```

Output shows where each resource comes from:
```
Available workflows:
  discovery (embedded, local)   # Both embedded and user-defined version exist
  execution (embedded)           # Only embedded version
  my-custom (local)              # User-defined only

Available guides:
  SPEC_WRITING.md (embedded, local)
  MY_GUIDE.md (local)
```

**See [CUSTOMIZING.md](CUSTOMIZING.md) for details on creating custom workflows and guides.**

### Advancing Through Phases

Hegel provides ergonomic commands for common workflow transitions:

```bash
# Happy path: advance to next phase
hegel next

# Go back to previous phase (undo accidental advancement)
hegel prev

# Repeat current phase (e.g., after addressing feedback)
hegel repeat

# Restart workflow cycle (return to SPEC phase)
hegel restart

# Abandon current workflow and start fresh
hegel abort
```

**Guardrails:**
- Cannot start a new workflow while one is active - must run `hegel abort` first
- This prevents accidentally losing workflow progress

**Advanced usage:**

For custom transitions, provide explicit claims. Claims are simple strings that trigger specific workflow transitions:

```bash
hegel next spec_complete        # Explicit claim
hegel next needs_refactor        # Custom transition (e.g., in execution workflow)
```

The `next` command automatically infers the happy-path claim (`{current_phase}_complete`) when no claim is provided.

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

**Start with cowboy mode (default):** Minimal overhead with just LEXICON guidance - use for most straightforward tasks.

**Escalate to full DDD workflows when:**
- Hard problems requiring novel solutions
- Projects needing extremely rigorous documentation
- Complex domains where mistakes are expensive
- Learning-dense exploration (discovery mode)

**Workflow selection:**
```bash
hegel start cowboy      # Default: minimal overhead
hegel start discovery   # Toy experiments with learning focus
hegel start execution   # Production-grade rigor
hegel start research    # External knowledge gathering
```

The full DDD workflow steps and accompanying token usage are designed for problems that **need** that rigor. Most tasks don't.

## Project Structure

```
hegel-cli/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── commands/        # Command implementations
│   ├── engine/          # Workflow state machine (dual template engines)
│   │   ├── mod.rs       # Workflow definitions and routing
│   │   ├── handlebars.rs  # Handlebars template engine
│   │   └── template.rs  # Markdown template engine (legacy)
│   └── storage/         # File-based state persistence
├── workflows/           # YAML workflow definitions
└── guides/              # Writing guides and templates
    ├── *.md             # Markdown guides
    ├── partials/        # Handlebars partials (.hbs files)
    └── templates/       # Markdown template fragments
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

**Default output** (brief summary):
- Session ID
- Token totals (input/output/cache)
- Activity counts (commands, files, commits)
- Workflow/phase counts
- Recent transitions

**Progressive disclosure** via flags:
```bash
# Show specific sections
hegel analyze --activity           # Session, tokens, commands, files
hegel analyze --workflow-transitions  # State transition history
hegel analyze --phase-breakdown      # Per-phase metrics
hegel analyze --workflow-graph       # ASCII visualization

# Show all sections (old default behavior)
hegel analyze --full

# Combine sections
hegel analyze --brief --phase-breakdown  # Brief + phases

# Export workflow graph to DOT format
hegel analyze --export-dot > workflow.dot
dot -Tpng workflow.dot -o workflow.png
```

**Section details:**

- **Brief** (default): Cross-section summary of key metrics
- **Activity** (`--activity`): Session, tokens, top bash commands, top file modifications
- **Workflow Transitions** (`--workflow-transitions`): Complete state transition history
- **Phase Breakdown** (`--phase-breakdown`): Per-phase metrics (duration, tokens, activity, git commits)
- **Workflow Graph** (`--workflow-graph`): ASCII visualization with node metrics and cycle detection

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

**See [GUARDRAILS.md](GUARDRAILS.md) for configuring safety guardrails around dangerous commands.**

## Advanced Tools

**See [ADVANCED_TOOLS.md](ADVANCED_TOOLS.md) for AST-based code transformation, document review, multi-project dashboard, and external agent orchestration.**

## Contributing

For AI agents or developers working **on** Hegel (not just using it), see **`CLAUDE.md`** for project structure, development guidelines, and contribution workflow.

## License

Server Side Public License v1 (SSPL)

## Learn More

Visit [dialectician.ai](https://dialectician.ai) for more information about Dialectic-Driven Development.
