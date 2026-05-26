# Refactor Analysis: DRY & Idiomatic Cleanup

**Date**: 2026-05-26
**Trigger**: Codebase-wide sweep for DRY violations and non-idiomatic Rust
**Goal**: Maximize token efficiency through DRY/SoC; modernize idioms. No behavior changes.
**Method**: Four parallel analyses (adapters / commands / metrics+analyze / core layers), each reading full source and reporting file:line evidence.

---

## Executive Summary

Findings cluster into three actionable tiers plus structural splits:

| Tier | Theme | Est. LOC removed | Risk |
|------|-------|------------------|------|
| **T1** | Cross-cutting shared helpers (touch many files) | ~140 | Low–Med |
| **T2** | File-local DRY (collapse duplicated blocks) | ~430 | Low–Med |
| **T3** | Idiomatic sweep (mechanical) | ~30 + clarity | Very low |
| **SoC** | Split 6 oversized files (>200 impl lines) | line-neutral | Med |

Plus **dead-code removal** (~180 lines) and **incidental correctness findings** (flagged separately, out of strict refactor scope).

The single highest-leverage pattern is **JSONL parse-and-iterate**, hand-rolled ~8 times across `metrics/` and `storage/` with *inconsistent* error handling (some fail-fast, some skip-corrupt) — a DRY win that also removes behavioral drift.

---

## Tier 1 — Cross-Cutting Shared Helpers (highest leverage)

These each replace a pattern duplicated across **multiple modules**. Build once, reuse everywhere.

### T1.1 — `parse_jsonl<T>()` helper [HIGH]

The "read file → iterate lines → skip blank → `serde_json::from_str` → handle error" loop is hand-rolled with subtle, inconsistent variations:

- `metrics/transcript.rs:66-100` (with_context, fail-fast)
- `metrics/states.rs:18-32` (fail-fast)
- `metrics/hooks.rs:101-189` (skip-corrupt, canonical→legacy fallback)
- `metrics/aggregation.rs:161-221` (`?` propagates — fails on bad line)
- `metrics/aggregation.rs:284-325` (skip-corrupt — *opposite* of the above, same job)
- `metrics/mod.rs:198-215` (inline hooks scan)
- `storage/mod.rs:429-443` (`read_command_log`)
- `storage/archive/mod.rs:133-153` (`read_archives`)

The inconsistency is itself a smell: two functions aggregating tokens disagree on whether a malformed line is fatal.

**Fix:** `fn parse_jsonl<T: DeserializeOwned>(content: &str, skip_errors: bool) -> Result<Vec<T>>` (or an iterator form `jsonl_records<T>(content) -> impl Iterator<Item = Result<T>>`). The write side (`storage::append_jsonl`) is already centralized — this is its read-side counterpart.
**Savings:** ~50-60 lines + unified error semantics.

### T1.2 — Token extraction + accumulation [HIGH]

The "old `.usage` vs new `.message.usage`" extraction plus the 5-field accumulation is copy-pasted 4×:

- `metrics/transcript.rs:85-95`
- `metrics/aggregation.rs:189-201`
- `metrics/aggregation.rs:312-323`
- `metrics/cowboy.rs:44-58` — **inverted precedence** (`.message…or(usage)` vs the others' `usage.or_else(.message)`); a latent inconsistency.

**Fix:**
```rust
impl TranscriptEvent {
    pub fn usage(&self) -> Option<&TokenUsage> {
        self.usage.as_ref()
            .or_else(|| self.message.as_ref().and_then(|m| m.usage.as_ref()))
    }
}
impl TokenMetrics {
    pub fn accumulate(&mut self, u: &TokenUsage) {
        self.total_input_tokens += u.input_tokens;
        self.total_output_tokens += u.output_tokens;
        self.total_cache_creation_tokens += u.cache_creation_input_tokens.unwrap_or(0);
        self.total_cache_read_tokens += u.cache_read_input_tokens.unwrap_or(0);
        self.assistant_turns += 1;
    }
}
```
All four sites collapse to `if let Some(u) = event.usage() { metrics.accumulate(u); }`. Also removes `.usage.clone()` allocations (returns a reference).
**Savings:** ~35 lines + fixes the precedence inconsistency.

Related: per-phase/per-archive 5-field summing is also hand-written 4× (`metrics/mod.rs:234-242, 263-269, 333-337`; `analyze/totals.rs:17-21`; `storage/archive/aggregation.rs:67-72`). Add `impl AddAssign for TokenMetrics` so these become `unified.token_metrics += phase.token_metrics;`. **+~25 lines.**

### T1.3 — `Node::prompt_text()` method [HIGH]

`if !node.prompt_hbs.is_empty() { &node.prompt_hbs } else { &node.prompt }` appears **7×**:

- `commands/workflow/mod.rs:270-274, 369-373, 509-513`
- `commands/workflow/transitions.rs:561-565`
- `engine/mod.rs` (~244, 309-313, 317-321)

This is also a SoC leak — the command layer shouldn't know prompt-field precedence.
**Fix:** `impl Node { fn prompt_text(&self) -> &str }` in the engine layer. Callers become `node.prompt_text()`.
**Savings:** ~18 lines + removes layering leak.

### T1.4 — `atomic_write_json<T: Serialize>()` [MED]

`to_string_pretty` → `fs::write(temp)` → `fs::rename` is duplicated in `storage/mod.rs:322-336` (`save`) and `storage/archive/mod.rs:104-117` (`write_archive`). Meanwhile `save_stash` (`storage/mod.rs:479-482`) and `reviews::write_hegel_reviews` (`138-139`) use *non-atomic* `fs::write` — a consistency/correctness gap.
**Fix:** Shared `fn atomic_write_json<T: Serialize>(path, value) -> Result<()>`; route all four through it.
**Savings:** ~20 lines + correctness for the two non-atomic writers.

### T1.5 — Shared `resolve_file_path` / `is_markdown_file` [MED]

- `resolve_file_path` is byte-identical in `commands/review.rs:48-65` and `commands/reflect.rs:184-201`.
- `is_markdown_file` is duplicated in `commands/init.rs:94-96` and `commands/markdown.rs:139-141`.
**Fix:** A small `commands/util.rs` shared module.
**Savings:** ~21 lines + test dedup.

---

## Tier 2 — File-Local DRY

### T2.1 — `ddd.rs`: collapse 4 scan loops + merge identical artifact types [HIGH]

`parse_ddd_structure_in` (`ddd.rs:311-450`) has four near-identical scan blocks (feat 311-346, refactor 348-380, report 382-410, toys 412-450). The refactor and report blocks are nearly byte-identical — only the enum variant and subdir name differ.

Worse, `RefactorArtifact` and `ReportArtifact` are **structurally identical** structs (date/index/name) with identical `file_name`/`file_path`/`IndexableArtifact` impls (`ddd.rs:59-91, 489-517`) differing only by `"refactor"` vs `"report"`.

**Fix:**
1. Generic `scan_dir(subdir, is_dir, parse_fn, fix_msg)` helper → refactor/report collapse to two one-line calls.
2. Merge the two structs into `SingleFileArtifact { kind, date, index, name }`.
3. Extract `fn dated_name(date, index, name, suffix) -> String` (the `{date}[-{idx}]-{name}` pattern, also in `FeatArtifact::dir_name`).
**Savings:** ~120-150 lines (biggest single-file win).

### T2.2 — `commands/new.rs`: collapse 3 create fns + use ddd.rs methods [HIGH]

`create_feat`/`create_refactor`/`create_report` (`new.rs:32-154`) share the same skeleton (validate → today → scan → dup-check → index → build name → print). They also **re-implement** ddd.rs's `dir_name`/`file_name`/`dir_path` format strings by hand (`new.rs:57-61` duplicates `FeatArtifact::dir_name`), so the formats can drift. `determine_index` (156-181) is just `determine_index_for_artifact` (183-211) with one match arm. The `unwrap_or_else(|_| DddScanResult{…})` fallback is repeated 5× (40-43, 84-87, 126-129, 160-163, 187-190).

**Fix:** Once T2.1 exposes artifact constructors, build the artifact and call `.dir_path()`. Merge the two index fns. Add `DddScanResult: Default` → `.unwrap_or_default()`.
**Savings:** ~80 lines. (Depends on T2.1.)

### T2.3 — `adapters/`: `detect_via`, `Default`, extraction helpers [HIGH]

Three adapters (claude_code/codex/cursor) duplicate the same shapes:
- **`detect()`** env-var-then-`$HOME`-path check: `claude_code.rs:31-50`, `codex.rs:149-164`, `cursor.rs:57-72`. → `fn detect_via(env_vars: &[&str], home_subpaths: &[&str]) -> bool` in `mod.rs`. ~30 lines.
- **`CanonicalHookEvent` construction**: full 11-field literal (`claude_code.rs:110-125`, `codex.rs:259-278`, `cursor.rs:142-154`). → `#[derive(Default)]` + `..Default::default()`. ~12 lines + future-proofs schema growth.
- **Field extraction** `obj.get(k).and_then(|v| v.as_str()).map(String::from)` (15+ sites) + inconsistent missing-field errors (`.context(…)` vs `ok_or_else(|| anyhow!(…))`). → promote codex's `try_extract_field` to shared `opt_str`/`req_str`. ~15 lines + uniform errors.
- **`new()` without `Default`** on all three (clippy `new_without_default`). → `#[derive(Default)]`.
**Savings:** ~80-100 lines.

### T2.4 — `commands/workflow`: `State::with_workflow` + `workflow_path` [MED-HIGH]

- **`State` reconstruction** (copy `session_metadata`/`cumulative_totals`/`git_info`, change only `workflow`) hand-written 6×: `meta.rs:47-60`, `workflow/mod.rs:100-105, 193-199, 342-347, 474-479`, `transitions.rs:454-459, 523-528`. transitions.rs even does a redundant double `storage.load()`. → `State::with_workflow(self, ws) -> State`. ~30 lines + removes a "forgot to preserve a field" footgun.
- **Workflow YAML path + load** built 5×: `workflow/mod.rs:36, 253, 306, 500`, `context.rs:29` (plus `format!("{}/{}.yaml")` variants). → `FileStorage::workflow_path(mode)` + reuse existing `load_workflow_context`. ~25 lines.
- **Stash-message quoting** `match &msg { Some(m) => format!(r#" "{}""#, m), None => String::new() }` 4× (`workflow/mod.rs:395-398, 419-422, 486-489, 533-536`). → `fmt_stash_message`. ~12 lines.
**Savings:** ~67 lines.

### T2.5 — `rules/evaluator.rs`: generic window-filter [MED]

`evaluate_repeated_file_edit` (`96-186`) and `evaluate_repeated_command` (`300-390`) are structurally identical: same phase-start parse, regex compile, timestamp-window filter, and "last 5 events" builder — differing only by source collection and message strings. Phase-lookup-by-name is also repeated 3× (`62-70, 198-206, 251-259`).
**Fix:** Generic `count_in_window<T>(items, phase_start, window, regex, get_ts, get_text)` + `current_phase_metrics(ctx)` helper.
**Savings:** ~60 lines.

### T2.6 — `storage/mod.rs`: stash directory iteration [MED]

`list_stashes` (500-531), `delete_stash` (554-581), `reindex_stashes` (584-618) all do `join("stashes")` → guard → `read_dir` → `.json` filter → parse `StashEntry`. The extension check is verbatim 3×. `load_stash` calls `list_stashes()` twice.
**Fix:** `fn load_all_stashes(&self) -> Result<Vec<(PathBuf, StashEntry)>>`.
**Savings:** ~35 lines.

### T2.7 — `metrics`/`analyze` misc DRY [MED]

- **Top-N frequency rendering** near-identical in `analyze/sections.rs:132-153` & `197-213`; `metrics/hooks.rs:80-95`. → `top_n_by_count` + generic `frequency`. ~30 lines.
- **Synthetic-cowboy builder** duplicated: `metrics/cowboy.rs:27-115` & `analyze/gap_detection.rs:271-365` (state-transitions vec byte-identical). → shared `cowboy_transitions(...)`. ~25 lines.
- **Archive-path build + remove** copy-pasted 4× in `gap_detection.rs:154-159, 221-227, 242-248, 292-297`; activity check computed twice. → `remove_archive_file` + single-pass gaps. ~30 lines.
- **Phase-label-with-tokens** duplicated across ASCII/DOT renderers in `metrics/graph.rs:149-163` & `226-240`. → shared `phase_label`. ~15 lines.
**Savings:** ~100 lines.

---

## Tier 3 — Idiomatic Sweep (mechanical, very low risk)

| Pattern | Idiomatic form | Sites |
|---------|----------------|-------|
| `.map_or(false, \|x\| …)` | `.is_some_and(\|x\| …)` | `aggregation.rs:43,54,121,122,147,179,203,209,215,223,254,256,267,327,335` + storage |
| `.or_insert_with(Vec::new)` | `.or_default()` | `review.rs:112,228`, `archive.rs:209`, `graph.rs:46`, `storage/archive/aggregation.rs:17`, `fix_state.rs:143` |
| `.args(&["…"])` | `.args(["…"])` (IntoIterator) | `ddd.rs:463`, `fix_ddd.rs:11,39,160` |
| manual `if-let`/`match … None => return` | `or_else` / `let-else` / `then_some` | `codex.rs:211-229, 276` |
| `match rule { … => "name" }` string table (duplicated in `engine/mod.rs:277-283` **and** `evaluator.rs:14-20`) | `impl RuleConfig { fn type_name(&self) -> &'static str }` (serde tags already define names) | removes a 2-place footgun |
| inline `use chrono::Utc;` in fn bodies | one module-level import | `workflow/mod.rs:27,294`, `transitions.rs:424` |
| manual `for … { count += 1 }` | `.filter(…).count()` | `new.rs:166-173, 193-203` (inconsistent with `fix_ddd.rs:85-92`) |
| `String::from_utf8_lossy(…).trim().to_string()` | `fn cmd_stdout_trimmed(output) -> String` | `fork/mod.rs:101`, `fork/runtime.rs:60`, `fix_ddd.rs:24`, `external_bin.rs:46-47` |
| dead empty `if` / vestigial alias | delete | `evaluator.rs:313-316`; `storage/mod.rs:454-458` (`let workflow_state = workflow;`) |
| `assert_eq!(x, true)` | `assert!(x)` | `cowboy.rs:191` |

**Savings:** ~30 lines + broad clarity/clippy cleanliness.

---

## Dead Code Removal (~180 lines)

- **`aggregate_tokens_for_phase`** (`metrics/aggregation.rs:132-233`) — `#[allow(dead_code)]` with "TODO: still needed?"; fully subsumed by `aggregate_tokens_for_range`. ~100 lines.
- **`parse_transcript_file`** (`metrics/transcript.rs:64-101`) — `#[allow(dead_code)]`, only used by its own tests.
- **`/tmp/hegel_repair_debug.log` double-writes** (`analyze/repair.rs:191-320`) — writes the same messages to a hardcoded temp file *and* `eprintln!`; ~80-100 lines of debug scaffolding with a stray side effect. ✅ Approved for removal (debug-only side effect).

---

## SoC — Split Oversized Files (>200 impl lines; line-neutral)

The project's own rule: split files >200 impl lines. Current violators:

| File | Lines | Proposed split |
|------|-------|----------------|
| `commands/markdown.rs` | 806 | `markdown/{scan, tree, render, json, mod}.rs` (mixes IO/model/render/serialization; `render_tree_child` alone ~127 lines) |
| `ddd.rs` | 868 | `ddd/{types, parse, scan, index}.rs` (git subprocess in a parsing module is misplaced) |
| `commands/workflow/mod.rs` | 739 | extract `stash.rs` (~160) + `listing.rs` (~190); mod.rs → ~390 |
| `storage/mod.rs` | 687 | extract `stash.rs` + `git.rs` (uses `git2`, distinct from rest) |
| `commands/workflow/transitions.rs` | 606 | extract `archiving.rs` (cowboy detection + archive + cleanup are persistence concerns, not transition eval); `execute_transition` is a 188-line fn — split per-arm + shared `print_transition` |
| `metrics/aggregation.rs` | 408→~250 | after dead-code removal; separate token parsing from `DebugConfig`/`eprintln!` formatting |

Also: `engine/mod.rs::get_next_prompt` (197-325) mixes transition eval + inline rule filtering with 5 mid-fn `use` imports → extract `select_prompt` (see T1.3), `filter_rules`, `evaluate_node_rules`.

---

## Incidental Correctness Findings (OUT OF REFACTOR SCOPE — surface to user)

Discovered during analysis; *not* DRY/idiomatic. **Decision: left entirely separate** — do NOT fold into this refactor, even opportunistically. Tracked here for awareness / a future fix pass:

1. **`rules/evaluator.rs:169`** — `…unwrap_or(&"unknown".to_string())[11..19]`: `"unknown"` is 7 chars, so the fallback slice is **out of bounds → panic**. (Also lines 280, 375 slice `[11..19]` unchecked.)
2. **`commands/status.rs:83`** — `if state.workflow.is_none() || state.workflow.is_none()` — condition OR'd with itself; almost certainly a copy-paste bug.
3. **`metrics/cowboy.rs`** — inverted token-usage precedence vs the other 3 sites (resolved incidentally by T1.2).
4. **`*.unwrap()` on `state_dir.parent()`** repeated (`mod.rs:188,350`; `gap_detection.rs:64,308,321`) — panics if no parent. A `project_root()` helper returning `Result` would fix.

---

## Implementation Order

1. **T1.1–T1.2** (jsonl + token helpers) — highest leverage, unblocks metrics simplification, fixes a precedence inconsistency en route.
2. **T3** idiomatic sweep + dead-code removal — mechanical, low-risk, shrinks files before splitting.
3. **T1.3–T1.5** (prompt_text, atomic_write, shared util) — small surface, broad reach.
4. **T2.3** adapters — self-contained.
5. **T2.1 → T2.2** (ddd.rs then new.rs) — ordered: new.rs consumes ddd.rs constructors.
6. **T2.4–T2.7** remaining file-local DRY.
7. **SoC splits** last — operate on the now-smaller files.

---

## Success Criteria

- All existing tests pass without modification (re-exports preserve public APIs).
- No functionality changes (incidental bugs handled separately, if at all).
- `LOC_REPORT.md`: no refactored module file >200 impl lines.
- Token overhead reduced when reading any single concern.
- Aggregate: **~600 lines removed** (T1 ~140 + T2 ~430 + T3 ~30) + ~180 dead-code lines, plus 6 files brought into the 200-line rule.
