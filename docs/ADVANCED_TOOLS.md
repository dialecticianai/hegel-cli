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

**See [MD_REVIEW.md](MD_REVIEW.md) for complete documentation on:**
- `hegel reflect` - GUI-based document review
- `hegel review` - CLI-based review management with IDE integration

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

## No-Code IDE

Launch Electron-based IDE for AI-first development:

```bash
# Launch IDE
hegel ide

# Pass Electron arguments
hegel ide -- --inspect
```

**Powered by [hegel-ide](https://github.com/dialecticianai/hegel-ide)**, an Electron-based no-code IDE. Requires `hegel-ide` repo with dependencies installed (adjacent repo or via npx when published).

**Features:**
- Integrated terminal (xterm.js)
- Workflow orchestration interface
- No code editor by design (work at orchestration level)
- Session ID passthrough via `HEGEL_SESSION_ID`

**Philosophy:** AI handles code generation. Humans orchestrate workflows. No text editor needed.

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
