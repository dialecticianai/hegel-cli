# Advanced Tools

## AST-based Code Transformation

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

## Markdown Document Review

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

## Multi-Project Dashboard

Manage multiple Hegel projects via web UI (no args) or CLI commands:

```bash
# Launch web dashboard (auto-opens browser)
hegel pm

# CLI: Discover projects and print info
hegel pm discover

# CLI: Run command across all projects
hegel pm x status
```

**Powered by [hegel-pm](https://github.com/dialecticianai/hegel-pm)**, a web-based project manager. Requires `hegel-pm` binary built and available (adjacent repo or in PATH).

**Features:**
- Auto-discover projects with `.hegel/` directories
- View workflow state and metrics across projects
- Real-time updates and statistics
- Run commands across multiple projects (`hegel pm x`)
- HTTP endpoint benchmarks

## External Agent Orchestration

Detect available agent CLIs for task delegation:

```bash
# List installed agent CLIs
hegel fork
```

**Detects:**
- `claude` - Claude Code CLI (Anthropic)
- `aider` - AI pair programming (aider.chat)
- `copilot` - GitHub Copilot CLI
- `codex` - OpenAI Codex CLI (Rust binary)
- `gemini` - Google Gemini CLI (Node 20+)
- `cody` - Sourcegraph Cody CLI (Node 20+)
- `amp` - Sourcegraph Amp agentic coding (Node 20+)

**Output:** Shows which agents are installed and their paths, plus which are not available.

**Roadmap:** Future versions will support `hegel fork --agent=<name> '<prompt>'` to delegate subtasks to external agents, enabling parallel work across multiple agents and agent comparison workflows.
