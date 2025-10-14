# Codex Adapter Implementation Guide

**Purpose:** Comprehensive reference for implementing Codex adapter, extracted from ccusage study.

**Source:** `vendor/ccusage/apps/codex/src/data-loader.ts` (lines 1-478)

---

## Codex Event Format Overview

### Log Structure

```
${CODEX_HOME:-~/.codex}/sessions/*.jsonl
```

Each JSONL file contains events for a single Codex session. Session ID is derived from filename.

### Event Types

Codex emits two event types we care about:

1. **`turn_context`** - Metadata about the current turn (includes model name)
2. **`event_msg` with `payload.type === "token_count"`** - Token usage data

### Sample Codex Events

```json
// turn_context - Sets current model
{
  "timestamp": "2025-09-11T18:25:30.000Z",
  "type": "turn_context",
  "payload": {
    "model": "gpt-5",
    "info": {
      "model": "gpt-5",
      "model_name": "gpt-5",
      "metadata": { "model": "gpt-5" }
    }
  }
}

// event_msg - Token usage
{
  "timestamp": "2025-09-11T18:25:40.670Z",
  "type": "event_msg",
  "payload": {
    "type": "token_count",
    "info": {
      "total_token_usage": {
        "input_tokens": 1200,
        "cached_input_tokens": 200,
        "output_tokens": 500,
        "reasoning_output_tokens": 0,
        "total_tokens": 1700
      },
      "last_token_usage": {
        "input_tokens": 1200,
        "cached_input_tokens": 200,
        "output_tokens": 500,
        "reasoning_output_tokens": 0,
        "total_tokens": 1700
      },
      "model": "gpt-5"
    }
  }
}
```

---

## Key Normalization Patterns

### 1. Model Detection Cascade

Model names live in multiple locations. Check in order:

```typescript
// Priority order (from data-loader.ts:109-158)
const model =
  payload.info?.model ??
  payload.info?.model_name ??
  payload.info?.metadata?.model ??
  payload.model ??
  payload.metadata?.model ??
  LEGACY_FALLBACK_MODEL; // "gpt-5"
```

**Rust equivalent:**
```rust
fn extract_model(payload: &serde_json::Value) -> Option<String> {
    payload.get("info")?
        .get("model")?.as_str()
        .or_else(|| payload.get("info")?.get("model_name")?.as_str())
        .or_else(|| payload.get("info")?.get("metadata")?.get("model")?.as_str())
        .or_else(|| payload.get("model")?.as_str())
        .or_else(|| payload.get("metadata")?.get("model")?.as_str())
        .map(String::from)
}
```

**Fallback strategy:**
- If no model found, use `"gpt-5"` as legacy fallback
- Set `fallback_used: true` when this happens
- Surfaces in output as `isFallbackModel: true`

### 2. Cumulative → Delta Conversion

Codex reports **cumulative totals** in `total_token_usage`. We need **per-event deltas**.

```typescript
// From data-loader.ts:229-296
let previousTotals: RawUsage | null = null;

for each event {
  const totalUsage = normalizeRawUsage(info?.total_token_usage);
  const lastUsage = normalizeRawUsage(info?.last_token_usage);

  let raw = lastUsage;
  if (raw == null && totalUsage != null) {
    // No direct delta, compute from cumulative
    raw = subtractRawUsage(totalUsage, previousTotals);
  }

  if (totalUsage != null) {
    previousTotals = totalUsage; // Track for next iteration
  }
}
```

**Rust equivalent:**
```rust
struct RawUsage {
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
    reasoning_output_tokens: u64,
    total_tokens: u64,
}

fn subtract_raw_usage(current: &RawUsage, previous: Option<&RawUsage>) -> RawUsage {
    RawUsage {
        input_tokens: current.input_tokens.saturating_sub(
            previous.map(|p| p.input_tokens).unwrap_or(0)
        ),
        cached_input_tokens: current.cached_input_tokens.saturating_sub(
            previous.map(|p| p.cached_input_tokens).unwrap_or(0)
        ),
        output_tokens: current.output_tokens.saturating_sub(
            previous.map(|p| p.output_tokens).unwrap_or(0)
        ),
        reasoning_output_tokens: current.reasoning_output_tokens.saturating_sub(
            previous.map(|p| p.reasoning_output_tokens).unwrap_or(0)
        ),
        total_tokens: current.total_tokens.saturating_sub(
            previous.map(|p| p.total_tokens).unwrap_or(0)
        ),
    }
}
```

### 3. Field Name Aliases

Codex has legacy field names:

```typescript
// From data-loader.ts:44
const cached = ensureNumber(
  record.cached_input_tokens ?? record.cache_read_input_tokens
);
```

**Rust:** Check both field names:
```rust
let cached_input_tokens = payload
    .get("cached_input_tokens")
    .or_else(|| payload.get("cache_read_input_tokens"))
    .and_then(|v| v.as_u64())
    .unwrap_or(0);
```

### 4. Total Tokens Synthesis

Legacy entries may omit `total_tokens`. Fallback:

```typescript
// From data-loader.ts:47-57
total_tokens: total > 0 ? total : input + output
```

**Important:** Don't add `reasoning_output_tokens` - it's already included in `output_tokens`.

---

## Token Field Semantics

| Field                      | Meaning                              | Billing                                          |
|----------------------------|--------------------------------------|--------------------------------------------------|
| `input_tokens`             | Prompt tokens sent                   | Priced at `input_cost_per_mtoken`                |
| `cached_input_tokens`      | Prompt tokens from cache             | Priced at `cached_input_cost_per_mtoken`         |
| `output_tokens`            | Completion tokens (includes reasoning) | Priced at `output_cost_per_mtoken`              |
| `reasoning_output_tokens`  | Optional reasoning breakdown         | **Informational only** - already in output      |
| `total_tokens`             | Sum provided by Codex                | Use verbatim, or fallback to `input + output`   |

**Critical:** `reasoning_output_tokens` is NOT charged separately. Don't double-count it.

---

## Mapping to Hegel's CanonicalHookEvent

### Codex → Canonical Event Mapping

```rust
CanonicalHookEvent {
    timestamp: event.timestamp,          // ISO 8601 string
    session_id: session_id.clone(),      // From filename
    event_type: EventType::PostToolUse,  // All Codex events are post-execution

    // Store tokens in tool_response (not tool_input)
    tool_name: Some("Codex".to_string()),
    tool_input: None,
    tool_response: Some(json!({
        "input_tokens": raw.input_tokens,
        "cached_input_tokens": raw.cached_input_tokens,
        "output_tokens": raw.output_tokens,
        "reasoning_output_tokens": raw.reasoning_output_tokens,
        "total_tokens": raw.total_tokens,
        "model": model.clone(),
    })),

    cwd: None,  // Codex doesn't track working directory
    transcript_path: Some(file_path.to_string()),

    adapter: Some("codex".to_string()),
    fallback_used: Some(model_is_fallback),

    extra: {
        "model": model.clone(),  // Flatten for easy access
    },
}
```

### Why EventType::PostToolUse?

Codex token counts represent **completed turns** (tool execution finished). They're conceptually "PostToolUse" events even though Codex doesn't use that terminology.

---

## State Machine for Processing

```rust
struct CodexSessionState {
    current_model: Option<String>,
    current_model_is_fallback: bool,
    previous_totals: Option<RawUsage>,
}

impl CodexSessionState {
    fn process_event(&mut self, event: &serde_json::Value) -> Option<CanonicalHookEvent> {
        let event_type = event.get("type")?.as_str()?;

        match event_type {
            "turn_context" => {
                // Update current model context
                if let Some(model) = extract_model(event.get("payload")?) {
                    self.current_model = Some(model);
                    self.current_model_is_fallback = false;
                }
                None // Don't emit event
            }

            "event_msg" => {
                let payload = event.get("payload")?;
                if payload.get("type")?.as_str()? != "token_count" {
                    return None;
                }

                // Extract token data
                let info = payload.get("info")?;
                let last_usage = normalize_raw_usage(info.get("last_token_usage"));
                let total_usage = normalize_raw_usage(info.get("total_token_usage"));

                // Compute delta
                let mut raw = last_usage;
                if raw.is_none() && total_usage.is_some() {
                    raw = Some(subtract_raw_usage(
                        &total_usage.unwrap(),
                        self.previous_totals.as_ref()
                    ));
                }

                // Update state
                if let Some(ref t) = total_usage {
                    self.previous_totals = Some(t.clone());
                }

                // Determine model
                let extracted_model = extract_model(payload);
                let (model, is_fallback) = if let Some(m) = extracted_model {
                    self.current_model = Some(m.clone());
                    self.current_model_is_fallback = false;
                    (m, false)
                } else if let Some(ref m) = self.current_model {
                    (m.clone(), self.current_model_is_fallback)
                } else {
                    self.current_model = Some("gpt-5".to_string());
                    self.current_model_is_fallback = true;
                    ("gpt-5".to_string(), true)
                };

                // Skip zero-token events
                let raw = raw?;
                if raw.input_tokens == 0 && raw.output_tokens == 0 {
                    return None;
                }

                Some(build_canonical_event(event, raw, model, is_fallback))
            }

            _ => None
        }
    }
}
```

---

## Environment Detection

```rust
impl AgentAdapter for CodexAdapter {
    fn detect(&self) -> bool {
        // Check CODEX_HOME env var
        if std::env::var("CODEX_HOME").is_ok() {
            return true;
        }

        // Check default Codex directory
        let home = std::env::var("HOME").ok()?;
        let codex_dir = PathBuf::from(home).join(".codex/sessions");
        codex_dir.exists()
    }
}
```

---

## File Discovery

```rust
fn discover_codex_sessions() -> Result<Vec<PathBuf>> {
    let codex_home = std::env::var("CODEX_HOME")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap();
            format!("{}/.codex", home)
        });

    let sessions_dir = PathBuf::from(codex_home).join("sessions");

    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions = vec![];
    for entry in fs::read_dir(sessions_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            sessions.push(path);
        }
    }

    Ok(sessions)
}
```

---

## Graceful Degradation Patterns

### 1. Skip Malformed Lines

```rust
for line in content.lines() {
    let event: serde_json::Value = match serde_json::from_str(line) {
        Ok(e) => e,
        Err(_) => continue, // Skip silently
    };
    // Process event...
}
```

### 2. Flag Fallbacks

```rust
if model_is_fallback {
    event.fallback_used = Some(true);
    event.extra.insert("warning".to_string(), json!(
        "Model metadata missing, using fallback 'gpt-5'"
    ));
}
```

### 3. Zero-Token Events

```rust
if delta.input_tokens == 0
   && delta.cached_input_tokens == 0
   && delta.output_tokens == 0 {
    continue; // Skip empty events
}
```

---

## Testing Strategy

### Test Cases from ccusage

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_token_count_with_model() {
        // Basic event with model metadata
        let input = json!({
            "timestamp": "2025-09-11T18:25:40.670Z",
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "info": {
                    "last_token_usage": {
                        "input_tokens": 1200,
                        "cached_input_tokens": 200,
                        "output_tokens": 500,
                        "total_tokens": 1700
                    },
                    "model": "gpt-5"
                }
            }
        });

        let adapter = CodexAdapter::new();
        let event = adapter.normalize(input).unwrap().unwrap();

        assert_eq!(event.session_id, "test-session");
        assert_eq!(event.tool_name, Some("Codex".to_string()));
        assert_eq!(event.adapter, Some("codex".to_string()));
    }

    #[test]
    fn test_cumulative_to_delta_conversion() {
        // Simulates two events with cumulative totals
        let events = vec![
            json!({
                "timestamp": "2025-09-11T18:25:40Z",
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "total_token_usage": {
                            "input_tokens": 1200,
                            "output_tokens": 500,
                            "total_tokens": 1700
                        }
                    }
                }
            }),
            json!({
                "timestamp": "2025-09-11T18:26:00Z",
                "type": "event_msg",
                "payload": {
                    "type": "token_count",
                    "info": {
                        "total_token_usage": {
                            "input_tokens": 2000,  // +800
                            "output_tokens": 800,  // +300
                            "total_tokens": 2800   // +1100
                        }
                    }
                }
            }),
        ];

        // First event: 1200 input, 500 output
        // Second event: 800 input (2000-1200), 300 output (800-500)
    }

    #[test]
    fn test_legacy_fallback_model() {
        let input = json!({
            "timestamp": "2025-09-15T13:00:00Z",
            "type": "event_msg",
            "payload": {
                "type": "token_count",
                "info": {
                    "total_token_usage": {
                        "input_tokens": 5000,
                        "output_tokens": 1000,
                        "total_tokens": 6000
                    }
                }
                // NO model field
            }
        });

        let adapter = CodexAdapter::new();
        let event = adapter.normalize(input).unwrap().unwrap();

        assert_eq!(event.fallback_used, Some(true));
        assert_eq!(
            event.extra.get("model").unwrap().as_str(),
            Some("gpt-5")
        );
    }
}
```

---

## Key Differences: Codex vs Claude Code

| Aspect             | Codex                          | Claude Code                          |
|--------------------|--------------------------------|--------------------------------------|
| **Event format**   | `turn_context` + `event_msg`   | `SessionStart`, `PostToolUse`, etc.  |
| **Token data**     | Cumulative (needs delta calc)  | Already per-event deltas             |
| **Model location** | Multiple nested locations      | Top-level or in info                 |
| **File structure** | `~/.codex/sessions/*.jsonl`    | `~/.claude/projects/*/*.jsonl`       |
| **Session ID**     | From filename                  | From `session_id` field              |
| **Reasoning**      | Separate field, info-only      | Not present                          |

---

## Implementation Checklist

- [ ] Create `src/adapters/codex.rs`
- [ ] Implement `CodexAdapter` struct with state
- [ ] Implement `AgentAdapter` trait:
  - [ ] `name()` → `"codex"`
  - [ ] `detect()` → Check `CODEX_HOME` or `~/.codex`
  - [ ] `normalize()` → State machine with cumulative tracking
- [ ] Add model extraction cascade (9 locations)
- [ ] Add cumulative → delta conversion
- [ ] Handle field aliases (`cached_input_tokens` vs `cache_read_input_tokens`)
- [ ] Synthesize `total_tokens` when missing
- [ ] Skip zero-token events
- [ ] Flag fallback models with `fallback_used`
- [ ] Add tests for:
  - [ ] Basic token count parsing
  - [ ] Cumulative → delta conversion
  - [ ] Model cascade (with/without metadata)
  - [ ] Legacy fallback
  - [ ] Turn context updates
- [ ] Register in `AdapterRegistry`

---

## References

- **Source:** `vendor/ccusage/apps/codex/src/data-loader.ts`
- **Types:** `vendor/ccusage/apps/codex/src/_types.ts`
- **Docs:** `vendor/ccusage/apps/codex/CLAUDE.md`

**Generated:** 2025-10-14 (from ccusage v2.x study)
