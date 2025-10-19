# Using Hegel for Workflow Orchestration

**Hegel** is your orchestration layer for Dialectic-Driven Development. Use it to enforce structured workflows, wrap dangerous commands with guardrails, and perform precise AST-based code transformations.

---

## Quick Start

### Check if Hegel is Available

```bash
command -v hegel >/dev/null 2>&1 && echo "Hegel available" || echo "Hegel not installed"
```

If not available, check if it's built locally:
```bash
[ -f ./target/release/hegel ] && echo "Use ./target/release/hegel" || echo "Run: cargo build --release"
```

---

## Workflow Orchestration

### Declaring a Meta-Mode (Required First Step)

**IMPORTANT:** Before starting any workflow, declare which meta-mode pattern you're following:

```bash
# For greenfield learning projects
hegel meta learning

# For standard feature development with known patterns
hegel meta standard

# View current meta-mode
hegel meta
```

**Available meta-modes:**
- `learning` - Greenfield learning project (Research ↔ Discovery loop, starts with research)
- `standard` - Feature development with known patterns (Discovery ↔ Execution, starts with discovery)

**What happens:**
- Declares your meta-mode in `.hegel/metamode.json`
- Automatically starts the appropriate initial workflow for your meta-mode
- Sets up the workflow transition pattern

### Starting Additional Workflows

After declaring your meta-mode, transition between workflows:

```bash
hegel start discovery    # When in learning mode after research complete
hegel start execution    # When transitioning to production delivery
hegel start research     # External knowledge gathering (learning mode)
```

**What happens:**
- Loads workflow definition from `workflows/*.yaml`
- Initializes state in `.hegel/state.json`
- Prints the current phase prompt

**Available workflows:**
- `research` - External knowledge gathering (PLAN → STUDY → ASSESS → QUESTIONS)
- `discovery` - Optimized for learning density (SPEC → PLAN → CODE → LEARNINGS → README)
- `execution` - Optimized for production delivery
- `minimal` - Simplified workflow for quick iterations

**When to start workflows:**
- Beginning a new feature or project
- Transitioning between exploration and execution modes
- When the user explicitly requests DDD methodology

**When NOT to start workflows:**
- Simple, straightforward tasks
- User hasn't requested structured workflow
- Already mid-workflow (check `hegel status` first)

### Advancing Through Phases

Use these ergonomic commands to navigate the workflow:

```bash
# Happy path: advance to next phase
hegel next

# Restart workflow cycle (back to SPEC)
hegel restart

# Abandon current workflow and start fresh
hegel abort
```

**How it works:**
- `hegel next` automatically infers the completion claim for the current phase
- `hegel restart` always returns to the SPEC phase (universal across all workflows)
- `hegel abort` clears workflow state (required before starting a new workflow)
- No manual claim construction needed for common workflows

**Guardrails:**
- Cannot start a new workflow while one is active - must run `hegel abort` first
- This prevents accidentally losing workflow progress

### Checking Workflow Status

```bash
hegel status
```

Shows:
- Current mode (discovery/execution)
- Current phase/node
- Full transition history

**Use this to:**
- Orient yourself at session start
- Verify workflow state before advancing
- Check if a workflow is active

### Re-displaying Current Prompt

To see the current phase prompt again without advancing:

```bash
hegel repeat
```

Useful after an interrupt or when you need to review the current phase requirements.

### Resetting Workflow

```bash
hegel reset
```

Clears all workflow state. Use when:
- Abandoning current workflow
- Switching to a different methodology
- User explicitly requests reset

---

## Command Wrapping with Guardrails

Hegel can wrap dangerous commands with safety rules. If configured in `.hegel/guardrails.yaml`, commands are automatically protected.

### Using Wrapped Commands

```bash
# Git operations with safety checks
hegel git status
hegel git add .
hegel git commit -m "feat: add feature"
hegel git push

# Docker operations with safety checks
hegel docker ps
hegel docker stop container-name
```

**When wrapped commands are blocked:**
- Hegel prints the blocked command and reason
- Exits with code 1
- Logs the attempt to `.hegel/command_log.jsonl`
- Suggests editing `.hegel/guardrails.yaml`

**Default behavior if no guardrails.yaml:**
- Commands execute normally (passthrough)
- Still logged to `.hegel/command_log.jsonl` for audit trail

### Checking Guardrails Configuration

```bash
cat .hegel/guardrails.yaml
```

Example configuration:
```yaml
git:
  blocked:
    - pattern: "reset --hard"
      reason: "Destructive: permanently discards uncommitted changes"
    - pattern: "clean -fd"
      reason: "Destructive: removes untracked files/directories"
    - pattern: "push.*--force"
      reason: "Force push can overwrite remote history"
```

---

## AST-based Code Search and Transformation

Use `hegel astq` for precise, format-preserving code operations.

### Searching for Patterns

```bash
# Find all public functions
hegel astq -p 'pub fn $FUNC' src/

# Find struct definitions
hegel astq -p 'struct $NAME' src/

# Find function calls
hegel astq -p '$OBJ.$METHOD($$$)' src/
```

**Pattern syntax:**
- `$VAR` - Matches a single AST node (identifier, expression, etc.)
- `$$$` - Matches zero or more nodes (variadic)
- Patterns use tree-sitter syntax

### Transforming Code

```bash
# Replace println with log::info
hegel astq -p 'println!($X)' -r 'log::info!($X)' src/

# Add logging to functions
hegel astq -p 'fn $NAME($$$) { $$$ }' -r 'fn $NAME($$$) { log::trace!(stringify!($NAME)); $$$ }' src/
```

**Best practices:**
- Always preview changes first (astq shows diffs by default)
- Use `--apply` flag to actually write changes
- Run tests after transformations
- Commit before large refactorings

### When to Use astq vs. Manual Editing

**Use astq when:**
- Refactoring across many files
- Renaming functions/types that simple find-replace would break
- Adding consistent patterns (logging, error handling, etc.)
- Removing deprecated code patterns

**Use manual editing when:**
- Single-file changes
- Context-dependent logic changes
- astq pattern would be complex to express

---

## Document Review

Launch GUI for reviewing Markdown artifacts:

```bash
# Single file
hegel reflect SPEC.md

# Multiple files (tabs)
hegel reflect SPEC.md PLAN.md

# With custom output directory
hegel reflect SPEC.md --out-dir .reviews/
```

**What happens:**
1. Mirror GUI launches
2. User selects text → adds comment → submits
3. Review saved to `.ddd/<filename>.review.N` (JSONL format)
4. Mirror exits (ephemeral - no persistent state)

**Reading review files:**
```bash
cat .ddd/SPEC.review.1
```

Each line is JSON with:
- `timestamp` - ISO 8601
- `file` - Source file
- `selection` - Line/col range
- `text` - Selected snippet
- `comment` - User's feedback

**When to use reflect:**
- After generating SPEC.md or PLAN.md
- User wants to review long artifacts in GUI
- Collecting structured feedback on documents

**When NOT to use reflect:**
- User is happy with inline iteration
- Simple text files that don't need GUI review
- User hasn't requested review

---

## Metrics and Analysis

### Real-time Dashboard

```bash
hegel top
```

Launches interactive TUI with:
- Overview (session, tokens, activity)
- Phases (per-phase metrics)
- Events (tool usage timeline)
- Files (most-edited files)

**Keyboard shortcuts:**
- `q` - Quit
- `Tab` / `BackTab` - Switch tabs
- `↑↓` or `j`/`k` - Scroll
- `g` / `G` - Jump to top/bottom
- `r` - Reload metrics

### Static Analysis

```bash
hegel analyze
```

Prints summary of:
- Token usage (input/output/cache)
- Activity (bash commands, file edits)
- Top commands and files
- Workflow transitions
- Per-phase breakdown
- Workflow graph visualization

---

## Integration Patterns

### Starting a Session

```bash
# Check if meta-mode is declared
hegel meta

# If no meta-mode declared and user wants structure:
hegel meta standard  # or: hegel meta learning

# Check if workflow is active
hegel status

# If workflow active, check current phase:
hegel status | grep "Current node"
```

### During Development

```bash
# Before dangerous git operations:
hegel git reset --hard  # Will block if configured

# Search for patterns:
hegel astq -p 'pattern' src/

# Check metrics:
hegel top  # Interactive
hegel analyze  # One-shot summary
```

### Advancing Workflow

```bash
# Completed current phase
hegel next

# User requests to restart cycle
hegel restart
```

### Reviewing Artifacts

```bash
# Launch review UI
hegel reflect SPEC.md

# After user reviews, read feedback:
cat .ddd/SPEC.review.1 | jq -r '.comment'
```

---

## State Files Reference

### .hegel/state.json
Current workflow state:
- Workflow definition
- Current node/phase
- History
- Session metadata

### .hegel/hooks.jsonl
Claude Code hook events (one per line):
- Tool usage (Bash, Read, Edit, etc.)
- File modifications
- Timestamps

### .hegel/states.jsonl
Workflow state transitions (one per line):
- From/to nodes
- Mode
- Workflow ID
- Timestamp

### .hegel/command_log.jsonl
Command invocations (one per line):
- Command name
- Arguments
- Success/failure
- Blocked reason (if applicable)
- Timestamp

### .hegel/guardrails.yaml
Safety rules for wrapped commands:
- Blocked patterns (regex)
- Reasons for blocks
- Per-command configuration

---

## Common Patterns

### Pattern: Start Session with Workflow

```bash
# Check status first
if ! hegel status 2>/dev/null | grep -q "Current node"; then
    echo "Starting discovery workflow..."
    hegel start discovery
else
    echo "Workflow already active:"
    hegel status
fi
```

### Pattern: Safe Git Operations

```bash
# Always use wrapped git in workflows
hegel git add .
hegel git commit -m "feat: implement feature"
hegel git push
```

### Pattern: AST-based Refactoring

```bash
# Search first to preview
hegel astq -p 'old_pattern' src/

# Transform with preview (default behavior shows diff)
hegel astq -p 'old_pattern' -r 'new_pattern' src/

# Apply changes
hegel astq -p 'old_pattern' -r 'new_pattern' --apply src/

# Verify with tests
cargo test
```

### Pattern: Document Review Loop

```bash
# Generate artifact
echo "Writing SPEC.md based on user requirements..."
# ... create SPEC.md ...

# Request review
echo "Please review SPEC.md in the GUI:"
hegel reflect SPEC.md

# Read feedback
if [ -f .ddd/SPEC.review.1 ]; then
    echo "Review received. Processing feedback..."
    cat .ddd/SPEC.review.1 | jq -r '.comment'
fi
```

---

## Error Handling

### Command Not Found

If `hegel` command fails:
1. Check if built: `ls -l ./target/release/hegel`
2. Use local binary: `./target/release/hegel status`
3. Build if missing: `cargo build --release`

### Workflow Not Loaded

Error: "No workflow loaded"

**Solution:**
```bash
hegel start discovery  # or execution
```

### Guardrail Violation

Error: "⛔ Command blocked by guardrails"

**Solutions:**
1. Review reason: Check the printed error message
2. Edit rules: Modify `.hegel/guardrails.yaml` if block is incorrect
3. Use alternative command: Find non-destructive alternative

### No Transition Available

Error: "Stayed at current node" when expecting to advance

**Solution:**
- Check current phase: `hegel status`
- Verify you're not at a terminal node (e.g., "done")
- Use `hegel restart` to return to SPEC if you want to restart the cycle
- For custom workflows with special transitions, the user may need to teach you about custom claims

---

## Best Practices

### DO:
- ✅ Check `hegel status` before starting workflows
- ✅ Use `hegel git` for all git operations in structured workflows
- ✅ Preview `astq` transformations before applying
- ✅ Read review files after `hegel reflect`
- ✅ Use `hegel top` to monitor session metrics

### DON'T:
- ❌ Start workflow if user hasn't requested structure
- ❌ Skip guardrails with raw `git` commands
- ❌ Use `astq --apply` without previewing changes
- ❌ Ignore workflow prompts (they contain phase-specific guidance)
- ❌ Reset workflow without user confirmation

---

## Quick Reference

```bash
# Meta-mode (required first step)
hegel meta <mode>               # Declare meta-mode (learning/standard)
hegel meta                      # View current meta-mode

# Workflow
hegel start <workflow>          # Start workflow
hegel next                      # Advance to next phase
hegel restart                   # Restart cycle (back to SPEC)
hegel status                    # Check state
hegel repeat                    # Re-show prompt
hegel reset                     # Clear state

# Commands
hegel git <args>                # Safe git wrapper
hegel docker <args>             # Safe docker wrapper

# Code
hegel astq -p 'pattern' <path>  # Search code
hegel astq -p 'old' -r 'new'    # Transform code

# Review
hegel reflect <files>           # Launch review GUI

# Metrics
hegel top                       # Interactive dashboard
hegel analyze                   # Static summary
```

---

**Remember:** Hegel is orchestration, not control. Use it when structure helps, skip it when it doesn't. The user always knows best.
