# State Storage

All state is stored in `.hegel/state.json` (current working directory) with atomic writes to prevent corruption. The state file contains:
- Current workflow definition
- Current node/phase
- Navigation history
- Workflow mode
- Unique workflow ID (ISO 8601 timestamp)
- Session metadata (session ID, transcript path, timestamp)

## Configurable State Directory

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
