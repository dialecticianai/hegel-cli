# Lines of Code Report

**Last Updated**: 2025-10-24 12:47
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 11,512 | 6,226 | 17,738 |
| **Comments** | 1,574 | - | 1,574 |
| **Blank Lines** | 2,128 | - | 2,128 |
| **Total Lines** | 15,214 | 6,226 | 21,440 |
| **Files** | 66 | 30 | 96 |

**Documentation Ratio**: 0.54 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            66           2128           1574          11512
-------------------------------------------------------------------------------
SUM:                            66           2128           1574          11512
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `adapters/claude_code.rs` | 258 | 129 | 129 | 50.0% | ✅ |
| `adapters/codex.rs` | 466 | 285 | 181 | 38.8% | ✅ (infra) |
| `adapters/cursor.rs` | 307 | 157 | 150 | 48.9% | ✅ |
| `adapters/mod.rs` | 223 | 125 | 98 | 43.9% | ✅ |
| `commands/analyze/mod.rs` | 182 | 28 | 154 | 84.6% | ✅ |
| `commands/analyze/sections.rs` | 267 | 267 | 0 | 0.0% | ✅ (infra) |
| `commands/archive.rs` | 302 | 212 | 90 | 29.8% | ⚠️ Large |
| `commands/astq.rs` | 113 | 88 | 25 | 22.1% | ✅ |
| `commands/config.rs` | 134 | 57 | 77 | 57.5% | ✅ |
| `commands/fork.rs` | 154 | 115 | 39 | 25.3% | ✅ |
| `commands/git.rs` | 48 | 0 | 48 | 100.0% | ✅ |
| `commands/hook.rs` | 236 | 108 | 128 | 54.2% | ✅ |
| `commands/hooks_setup.rs` | 235 | 118 | 117 | 49.8% | ✅ |
| `commands/init.rs` | 167 | 72 | 95 | 56.9% | ✅ |
| `commands/meta.rs` | 247 | 163 | 84 | 34.0% | ✅ |
| `commands/mod.rs` | 30 | 30 | 0 | 0.0% | ✅ |
| `commands/reflect.rs` | 132 | 83 | 49 | 37.1% | ✅ |
| `commands/workflow/claims.rs` | 28 | 28 | 0 | 0.0% | ✅ |
| `commands/workflow/context.rs` | 76 | 76 | 0 | 0.0% | ✅ |
| `commands/workflow/mod.rs` | 397 | 16 | 381 | 96.0% | ✅ |
| `commands/workflow/tests/commands.rs` | 372 | 372 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/tests/integration.rs` | 65 | 65 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/mod.rs` | 69 | 69 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/node_flow.rs` | 142 | 142 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/production.rs` | 67 | 67 | 0 | 0.0% | ✅ |
| `commands/workflow/tests/transitions.rs` | 346 | 346 | 0 | 0.0% | ✅ (infra) |
| `commands/workflow/transitions.rs` | 333 | 333 | 0 | 0.0% | ✅ (infra) |
| `commands/wrapped.rs` | 131 | 68 | 63 | 48.1% | ✅ |
| `config.rs` | 147 | 102 | 45 | 30.6% | ✅ |
| `embedded.rs` | 90 | 90 | 0 | 0.0% | ✅ |
| `engine/mod.rs` | 819 | 169 | 650 | 79.4% | ✅ |
| `engine/template.rs` | 676 | 162 | 514 | 76.0% | ✅ |
| `guardrails/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `guardrails/parser.rs` | 71 | 23 | 48 | 67.6% | ✅ |
| `guardrails/types.rs` | 170 | 97 | 73 | 42.9% | ✅ |
| `main.rs` | 280 | 13 | 267 | 95.4% | ✅ |
| `metamodes/mod.rs` | 204 | 99 | 105 | 51.5% | ✅ |
| `metrics/aggregation.rs` | 203 | 144 | 59 | 29.1% | ✅ |
| `metrics/graph.rs` | 372 | 224 | 148 | 39.8% | ✅ (infra) |
| `metrics/hooks.rs` | 330 | 220 | 110 | 33.3% | ✅ (infra) |
| `metrics/mod.rs` | 605 | 168 | 437 | 72.2% | ✅ |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 998 | 121 | 877 | 87.9% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 299 | 74 | 225 | 75.3% | ✅ |
| `storage/archive.rs` | 535 | 306 | 229 | 42.8% | ⚠️ Large |
| `storage/mod.rs` | 700 | 313 | 387 | 55.3% | ✅ (infra) |
| `test_helpers/fixtures.rs` | 29 | 29 | 0 | 0.0% | ✅ |
| `test_helpers/jsonl.rs` | 124 | 124 | 0 | 0.0% | ✅ |
| `test_helpers/metrics.rs` | 272 | 272 | 0 | 0.0% | ✅ (infra) |
| `test_helpers/mod.rs` | 20 | 10 | 10 | 50.0% | ✅ |
| `test_helpers/storage.rs` | 66 | 66 | 0 | 0.0% | ✅ |
| `test_helpers/tui.rs` | 73 | 73 | 0 | 0.0% | ✅ |
| `test_helpers/workflow.rs` | 296 | 296 | 0 | 0.0% | ✅ (infra) |
| `theme.rs` | 96 | 66 | 30 | 31.2% | ✅ |
| `tui/app.rs` | 402 | 165 | 237 | 59.0% | ✅ |
| `tui/mod.rs` | 84 | 67 | 17 | 20.2% | ✅ |
| `tui/tabs/events.rs` | 132 | 113 | 19 | 14.4% | ✅ |
| `tui/tabs/files.rs` | 90 | 72 | 18 | 20.0% | ✅ |
| `tui/tabs/mod.rs` | 9 | 9 | 0 | 0.0% | ✅ |
| `tui/tabs/overview.rs` | 88 | 74 | 14 | 15.9% | ✅ |
| `tui/tabs/phases.rs` | 143 | 111 | 32 | 22.4% | ✅ |
| `tui/ui.rs` | 319 | 160 | 159 | 49.8% | ✅ |
| `tui/utils.rs` | 364 | 211 | 153 | 42.0% | ✅ (infra) |

**⚠️ Warning:** 2 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `.ddd/feat/log_retention/PLAN.md` | 535 |
| `.ddd/feat/log_retention/SPEC.md` | 366 |
| `CLAUDE.md` | 149 |
| `CODE_MAP.md` | 210 |
| `commands/hegel.md` | 23 |
| `COVERAGE_REPORT.md` | 166 |
| `DEP_REVIEW.md` | 678 |
| `docs/astq_patterns/README.md` | 127 |
| `guides/ARCHITECTURE_WRITING.md` | 258 |
| `guides/CLAUDE_CUSTOMIZATION.md` | 312 |
| `guides/CODE_MAP_WRITING.md` | 108 |
| `guides/HANDOFF_WRITING.md` | 207 |
| `guides/KICKOFF_WRITING.md` | 96 |
| `guides/KNOWLEDGE_CAPTURE.md` | 345 |
| `guides/LEARNINGS_WRITING.md` | 96 |
| `guides/PLAN_WRITING.md` | 149 |
| `guides/QUESTION_TRACKING.md` | 362 |
| `guides/README_WRITING.md` | 142 |
| `guides/SPEC_WRITING.md` | 115 |
| `guides/STUDY_PLANNING.md` | 209 |
| `guides/templates/code_map_hierarchical.md` | 48 |
| `guides/templates/code_map_monolithic.md` | 40 |
| `guides/templates/mirror_workflow.md` | 11 |
| `guides/VISION_WRITING.md` | 176 |
| `HEGEL_CLAUDE.md` | 267 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 167 |
| `README.md` | 556 |
| `ROADMAP.md` | 183 |
| `TESTING.md` | 41 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.54 | ✅ Excellent |
| README exists | Yes | ✅ | Met |
| Architecture docs | Yes | ❌ | Optional |

---

## How to Update This Report

```bash
# Regenerate LOC report
./scripts/generate-loc-report.sh
```

---

*This report is auto-generated from `cloc` and `wc` output.*
*Updated automatically by pre-commit hook when source files change.*
