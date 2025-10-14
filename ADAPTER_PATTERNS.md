# Adapter Patterns from ccusage

**Source**: Vendored from https://github.com/ryoppippi/ccusage (actively maintained, mature codebase)

## Architecture Overview

ccusage supports both Claude Code and OpenAI Codex using a **shared normalization pattern** rather than traditional adapter classes. Key insight: they normalize different log formats into a canonical event schema.

## Key Patterns to Steal

### 1. **Data Loader Per Agent** (Not Shared Adapters)

Instead of a single `AgentAdapter` interface, they use **separate data loaders** that both normalize to the same output schema:

```
apps/ccusage/src/data-loader.ts    # Claude Code specific
apps/codex/src/data-loader.ts      # Codex specific
```

**Why this works:**
- Each agent has wildly different log formats (JSONL structure, field names, etc.)
- Shared: Output schema (`TokenUsageEvent`, cost calculation)
- Different: Parsing logic, file discovery, environment variables

### 2. **Canonical Event Schema**

Both loaders normalize to a common shape:

```typescript
type TokenUsageEvent = {
    sessionId: string;
    timestamp: string;  // ISO 8601
    model: string;
    inputTokens: number;
    cachedInputTokens: number;
    outputTokens: number;
    reasoningOutputTokens: number;  // Codex-specific, 0 for Claude
    totalTokens: number;
    isFallbackModel?: boolean;  // When model can't be detected
}
```

**Normalization responsibilities:**
- Parse agent-specific JSONL format
- Extract model name from various nested locations
- Calculate deltas from cumulative totals (Codex) or use direct values (Claude)
- Handle missing/legacy fields gracefully

### 3. **Environment-Based Discovery**

**Claude Code:**
```typescript
// Checks multiple locations automatically
const paths = [
    process.env.CLAUDE_CONFIG_DIR,  // Override
    '~/.config/claude/projects/',   // New default
    '~/.claude/projects/'           // Legacy
];
```

**Codex:**
```typescript
const codexHome = process.env.CODEX_HOME ?? '~/.codex';
const sessionsDir = path.join(codexHome, 'sessions/');
```

**Pattern:** Each adapter knows where to find its logs via env vars with sensible defaults.

### 4. **Graceful Degradation**

**Model Detection Cascade (Codex example):**
```typescript
// Try multiple locations in order
const model =
    payload.info.model ??
    payload.info.model_name ??
    payload.info.metadata?.model ??
    payload.model ??
    payload.metadata?.model ??
    LEGACY_FALLBACK_MODEL;  // 'gpt-5' with isFallbackModel flag
```

**Benefits:**
- Works with old logs that lack metadata
- Flags uncertain data with `isFallbackModel: true`
- Shows degraded data to user rather than failing silently

### 5. **Shared Utilities, Not Shared Parsers**

**What's shared:**
```
packages/terminal/       # Table rendering
calculate-cost.ts       # Cost calculation from tokens
_pricing-fetcher.ts     # LiteLLM pricing database
logger.ts               # Logging utilities
```

**What's separate:**
```
data-loader.ts          # Agent-specific parsing logic
_types.ts               # Agent-specific schemas
_consts.ts              # Agent-specific paths/globs
```

**Pattern:** Share the presentation layer, not the data extraction layer.

### 6. **Result Type for Error Handling**

Uses `@praha/byethrow` Result type instead of try-catch:

```typescript
const fileContentResult = await Result.try({
    try: readFile(file, 'utf8'),
    catch: error => error,
});

if (Result.isFailure(fileContentResult)) {
    logger.debug('Failed to read file', fileContentResult.error);
    continue;  // Skip bad files, don't crash
}
```

**Benefits:**
- Explicit error handling
- No thrown exceptions
- Easy to skip malformed files

### 7. **In-Source Testing**

Tests live alongside implementation:

```typescript
if (import.meta.vitest != null) {
    describe('loadTokenUsageEvents', () => {
        it('parses token_count events', async () => {
            await using fixture = await createFixture({
                sessions: {
                    'project-1.jsonl': [/* mock data */]
                }
            });
            // Test using real file system fixtures
        });
    });
}
```

**Pattern:** Use `fs-fixture` for realistic file system testing, not mocks.

## Anti-Patterns to Avoid

❌ **Single adapter interface** - Too much variance between agents
❌ **Shared parser** - Log formats are too different
❌ **Throwing errors** - Skip bad files, log warnings
❌ **External test files** - Harder to maintain, increases context

## Recommended Hegel Architecture

```
src/adapters/
├── mod.rs              # Registry + canonical event schema
├── claude_code.rs      # Claude Code adapter
├── cursor.rs           # Cursor adapter (future)
└── codex.rs            # Codex adapter (future)
```

**Each adapter:**
1. Defines its own file discovery logic (`~/.claude`, `~/.codex`, etc.)
2. Parses agent-specific JSONL format
3. Normalizes to canonical `HookEvent` schema
4. Handles missing fields gracefully with fallbacks
5. Returns `Vec<HookEvent>` for further processing

**Shared:**
- Event schema in `adapters/mod.rs`
- Cost calculation (if we integrate LiteLLM pricing)
- Metrics aggregation (already in `src/metrics/`)
- Table rendering (already in `src/tui/`)

## Migration Path for Hegel

**Current state:**
- Tight coupling to Claude Code hook event structure
- Assumes specific JSONL format from Claude Code

**Phase 1.1 implementation:**
1. Extract current hook parsing into `adapters/claude_code.rs`
2. Define canonical `HookEvent` schema in `adapters/mod.rs`
3. Add `AdapterRegistry::detect()` that checks env vars
4. Keep everything else the same (workflows, commands, metrics)

**Later:**
5. Add `adapters/cursor.rs` using same pattern
6. Add `adapters/codex.rs` using ccusage's Codex loader as reference

## Key Takeaway

**Don't build a generic adapter system.** Build **agent-specific normalizers** that output a **canonical event schema**. The variance is in the input, not the output.
