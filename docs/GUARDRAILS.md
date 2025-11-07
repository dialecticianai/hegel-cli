# Command Wrapping with Guardrails

Hegel can wrap dangerous commands with safety guardrails and audit logging. Commands are configured in `.hegel/guardrails.yaml` and automatically become available as `hegel <command>`.

## Configuration

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

## Usage

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
