# Phase 1.1 Refactor Plan: Multi-Agent Support

## Current State Analysis

**What we have:**
- Tight coupling to Claude Code hook event structure (`src/commands/hook.rs`)
- Single parser for hooks (`src/metrics/hooks.rs`) assumes Claude Code format
- Direct JSON append to `hooks.jsonl` without normalization
- Session metadata in `state.json` assumes Claude Code transcript path
- ~2,339 lines across metrics/hooks/storage/commands

**Problems:**
1. **Hard-coded Claude Code assumptions** - `session_id`, `hook_event_name`, `tool_name` fields
2. **No graceful degradation** - Missing fields cause silent failures or errors
3. **Manual error handling** - try-catch everywhere, not functional
4. **Mixed responsibilities** - `hook.rs` does both ingestion AND normalization
5. **No adapter abstraction** - Can't support Cursor/Codex without major rewrites

## Refactor Goals

### Primary: Agent Abstraction
✅ Support multiple agents (Claude Code, Cursor, Codex) via adapters
✅ Normalize different log formats to canonical schema
✅ Auto-detect agent from environment variables
✅ Backward compatible with existing `.hegel/hooks.jsonl` files

### Secondary: Code Quality Improvements (Steal from ccusage)
✅ Functional error handling with Result types
✅ Graceful degradation with fallback detection
✅ In-source testing with `fs-fixture`
✅ Shared utilities, agent-specific parsing
✅ Better validation with schemas

## Architecture Changes

### New Structure

```
src/
├── adapters/                    # NEW
│   ├── mod.rs                  # AdapterRegistry + canonical schema
│   ├── claude_code.rs          # Claude Code adapter (extracted from hook.rs)
│   ├── cursor.rs               # Cursor adapter (future)
│   └── codex.rs                # Codex adapter (future)
├── commands/
│   ├── hook.rs                 # SIMPLIFIED - just stdin → adapter → append
│   └── ...
├── metrics/
│   ├── hooks.rs                # SIMPLIFIED - works with canonical events only
│   └── ...
└── storage/
    └── mod.rs                  # UNCHANGED - still writes hooks.jsonl
```

### Canonical Event Schema (NEW)

```rust
// src/adapters/mod.rs

/// Canonical hook event - normalized from any agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHookEvent {
    // Universal fields (all agents)
    pub timestamp: String,           // ISO 8601
    pub session_id: String,
    pub event_type: EventType,

    // Optional fields (agent-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_input: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,

    // Metadata about normalization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,     // "claude_code", "cursor", "codex"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_used: Option<bool>, // True if had to guess missing fields

    // Catch-all for agent-specific extras
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    SessionStart,
    SessionEnd,
    PreToolUse,
    PostToolUse,
    Stop,
    Other(String), // Graceful unknown event types
}
```

### Adapter Trait

```rust
// src/adapters/mod.rs

pub trait AgentAdapter {
    /// Adapter name (e.g., "claude_code", "cursor", "codex")
    fn name(&self) -> &str;

    /// Check if this adapter should handle the current environment
    /// (Checks env vars like CLAUDE_CODE_SESSION_ID, CURSOR_SESSION_ID)
    fn detect(&self) -> bool;

    /// Normalize agent-specific JSON to canonical event
    /// Returns None if event should be skipped (invalid, malformed, etc.)
    fn normalize(&self, input: serde_json::Value) -> Option<CanonicalHookEvent>;

    /// Optional: Find transcript/log files for this agent
    fn discover_logs(&self) -> Result<Vec<PathBuf>> {
        Ok(vec![])
    }
}
```

### Adapter Registry

```rust
// src/adapters/mod.rs

pub struct AdapterRegistry {
    adapters: Vec<Box<dyn AgentAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: vec![
                Box::new(ClaudeCodeAdapter::new()),
                // Box::new(CursorAdapter::new()),  // Future
                // Box::new(CodexAdapter::new()),   // Future
            ],
        }
    }

    /// Auto-detect which adapter to use based on environment
    pub fn detect(&self) -> Option<&dyn AgentAdapter> {
        self.adapters
            .iter()
            .find(|adapter| adapter.detect())
            .map(|b| b.as_ref())
    }

    /// Get adapter by name (for explicit selection)
    pub fn get(&self, name: &str) -> Option<&dyn AgentAdapter> {
        self.adapters
            .iter()
            .find(|adapter| adapter.name() == name)
            .map(|b| b.as_ref())
    }
}
```

## Migration Strategy (Phased)

### Phase 1: Extract Claude Code Adapter (Week 1)

**Goal:** Refactor existing code without changing behavior

1. **Create adapter module structure**
   ```bash
   mkdir src/adapters
   touch src/adapters/mod.rs
   touch src/adapters/claude_code.rs
   ```

2. **Define canonical schema** in `src/adapters/mod.rs`
   - Copy event fields from current `HookEvent`
   - Add `adapter` and `fallback_used` metadata
   - Define `AgentAdapter` trait

3. **Extract Claude Code normalization** to `src/adapters/claude_code.rs`
   - Move JSON parsing from `commands/hook.rs` → `ClaudeCodeAdapter::normalize()`
   - Keep same validation logic
   - Add `adapter: "claude_code"` to output

4. **Simplify `commands/hook.rs`**
   ```rust
   pub fn handle_hook(_event_name: &str, storage: &FileStorage) -> Result<()> {
       let stdin = io::stdin();
       let hook_json: String = read_line(stdin)?;

       let registry = AdapterRegistry::new();
       let adapter = registry.detect().context("No agent detected")?;

       let raw_event: serde_json::Value = serde_json::from_str(&hook_json)?;
       let canonical = adapter.normalize(raw_event).context("Failed to normalize event")?;

       // Inject timestamp if not present
       let enriched = enrich_with_timestamp(canonical);

       // Write to hooks.jsonl
       storage.append_jsonl("hooks.jsonl", &enriched)?;

       // Handle SessionStart metadata
       if matches!(enriched.event_type, EventType::SessionStart) {
           update_session_metadata(storage, &enriched)?;
       }

       Ok(())
   }
   ```

5. **Update `metrics/hooks.rs`** to use canonical schema
   - Change `HookEvent` → `CanonicalHookEvent`
   - Update field access (`hook_event_name` → `event_type`)
   - Keep same aggregation logic

6. **Test backward compatibility**
   - Existing `.hegel/hooks.jsonl` files should still parse
   - New events should include `adapter` field
   - All tests should pass

**Deliverables:**
- ✅ No behavior changes for Claude Code users
- ✅ Canonical event schema defined
- ✅ Claude Code adapter extracted and tested
- ✅ Foundation for adding more adapters

### Phase 2: Add Result Types (Week 2)

**Goal:** Functional error handling like ccusage

1. **Add dependency**
   ```toml
   # Cargo.toml
   anyhow = "1.0"  # Already have this
   ```

2. **Create Result wrapper pattern**
   ```rust
   // src/adapters/mod.rs or utils
   pub enum ParseResult<T> {
       Ok(T),
       Skip,      // Malformed, should skip
       Error(E),  // Fatal error
   }
   ```

3. **Refactor parsers**
   - Replace `match serde_json::from_str() { Ok(..) => .., Err(..) => continue }`
   - With `if ParseResult::isSkip(result) { continue; }`
   - Better than silent failures

4. **Graceful degradation**
   - Add fallback field detection (like ccusage)
   - Set `fallback_used: true` when guessing
   - Log warnings for degraded data

**Deliverables:**
- ✅ Explicit error vs skip vs success
- ✅ Fallback detection flags
- ✅ Better debugging (know WHY events were skipped)

### Phase 3: In-Source Testing (Week 2-3)

**Goal:** Better test coverage with realistic fixtures

1. **Add test dependencies**
   ```toml
   [dev-dependencies]
   fs-fixture = "0.2"  # For realistic file system testing
   ```

2. **Convert tests** to in-source style
   ```rust
   // In src/adapters/claude_code.rs
   #[cfg(test)]
   mod tests {
       use super::*;
       use fs-fixture::FileFixture;

       #[test]
       fn test_normalize_session_start() {
           let input = serde_json::json!({
               "session_id": "test",
               "hook_event_name": "SessionStart",
               "transcript_path": "/tmp/transcript.jsonl"
           });

           let adapter = ClaudeCodeAdapter::new();
           let event = adapter.normalize(input).unwrap();

           assert_eq!(event.session_id, "test");
           assert!(matches!(event.event_type, EventType::SessionStart));
       }
   }
   ```

3. **Add integration tests** for file discovery
   ```rust
   #[test]
   fn test_discover_claude_code_logs() {
       let fixture = FileFixture::new()
           .file(".claude/projects/test/session.jsonl", "...")
           .build();

       let adapter = ClaudeCodeAdapter::new();
       let logs = adapter.discover_logs().unwrap();

       assert_eq!(logs.len(), 1);
   }
   ```

**Deliverables:**
- ✅ Tests alongside code
- ✅ Realistic file system fixtures
- ✅ Better coverage of edge cases

### Phase 4: Add Cursor/Codex Adapters (Future)

**Goal:** Prove the abstraction works

1. **Copy patterns from ccusage**
   - Use their Codex data loader as reference
   - Understand Cursor log format
   - Implement `normalize()` for each

2. **Add environment detection**
   ```rust
   impl AgentAdapter for CursorAdapter {
       fn detect(&self) -> bool {
           std::env::var("CURSOR_SESSION_ID").is_ok()
       }
   }
   ```

3. **Test with real logs** from each agent

**Deliverables:**
- ✅ Multi-agent support working
- ✅ Auto-detection via env vars
- ✅ All adapters output canonical schema

## Improvements Beyond Abstraction

### 1. Better Validation (Use Schemas)

Currently: Manual field access with `and_then()`
```rust
// Bad: Fragile
let command = tool_input.get("command").and_then(|v| v.as_str());
```

Better: Define schemas with serde
```rust
#[derive(Deserialize)]
struct BashToolInput {
    command: String,
    #[serde(default)]
    description: Option<String>,
}

let input: BashToolInput = serde_json::from_value(tool_input)?;
```

### 2. Reduce Duplication

Currently: `process_hook_event` + `parse_hooks_file` have overlapping logic

Better: Extract shared normalization
```rust
// src/adapters/claude_code.rs has normalize()
// Both hook.rs and metrics/hooks.rs call it
```

### 3. Environment-Based Discovery

Currently: Hard-coded `~/.claude/projects`

Better: Like ccusage
```rust
impl ClaudeCodeAdapter {
    fn get_project_dirs(&self) -> Vec<PathBuf> {
        let paths = if let Ok(custom) = env::var("CLAUDE_CONFIG_DIR") {
            vec![PathBuf::from(custom)]
        } else {
            vec![
                PathBuf::from("~/.config/claude/projects"),
                PathBuf::from("~/.claude/projects"),
            ]
        };

        paths.into_iter()
            .filter(|p| p.exists())
            .collect()
    }
}
```

### 4. File Locking Helper

Currently: Duplicated in `storage::append_jsonl` and `hook::process_hook_event`

Better: Extract to utility
```rust
// src/utils/fs.rs
pub fn append_jsonl_locked<P>(path: P, value: &serde_json::Value) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.lock_exclusive()?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    file.flush()?;
    Ok(())
}
```

## Testing Strategy

### Unit Tests (In-Source)
- [x] Canonical event schema serialization
- [x] Claude Code adapter normalization
- [x] Fallback detection
- [x] Event type parsing

### Integration Tests
- [x] End-to-end: stdin JSON → adapter → hooks.jsonl
- [x] Backward compatibility with existing hooks.jsonl
- [x] Session metadata updates on SessionStart
- [x] Multiple adapters in registry

### Regression Tests
- [x] All existing metrics tests still pass
- [x] Workflow rules still work with canonical events
- [x] TUI dashboard displays canonical events

## Success Criteria

✅ **Backward compatible** - Existing `.hegel/hooks.jsonl` files still work
✅ **Multi-agent ready** - Can add Cursor/Codex without refactoring
✅ **Better tested** - In-source tests with realistic fixtures
✅ **More resilient** - Graceful degradation with fallback flags
✅ **Cleaner code** - Single responsibility, shared utilities
✅ **Same behavior** - No user-facing changes for Claude Code users

## Timeline

- **Week 1**: Phase 1 (Extract Claude Code adapter)
- **Week 2**: Phase 2 (Result types) + Phase 3 (Testing)
- **Week 3**: Phase 4 (Add 1 more adapter to prove it works)

## Risks & Mitigations

**Risk:** Breaking existing hooks.jsonl files
**Mitigation:** Keep backward compat parsing, add `adapter` field as optional

**Risk:** Adapter abstraction too complex
**Mitigation:** Start simple (just normalize), add features incrementally

**Risk:** Performance degradation
**Mitigation:** Benchmark before/after, use same file I/O patterns

## Open Questions

1. Should we migrate existing `hooks.jsonl` files to canonical format?
   - **Recommendation:** No, read both formats (old + new)

2. Should adapters also handle transcript parsing?
   - **Recommendation:** Yes, each adapter knows its transcript format

3. How to handle agent-specific fields (like Codex `reasoning_output_tokens`)?
   - **Recommendation:** Use `extra` HashMap for adapter-specific data

## Next Steps

1. Review this plan with team/user
2. Create tracking issue for Phase 1.1
3. Start with Phase 1: Extract Claude Code adapter
4. Iterate based on feedback

---

**Key Insight from ccusage:** Don't build a generic adapter system. Build **agent-specific normalizers** that all output the **same canonical schema**. The complexity is in parsing varied inputs, not in the output format.
