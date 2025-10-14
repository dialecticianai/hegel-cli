# Lines of Code Report

**Last Updated**: 2025-10-14 02:50
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 7,183 | 4,972 | 12,155 |
| **Comments** | 946 | - | 946 |
| **Blank Lines** | 1,280 | - | 1,280 |
| **Total Lines** | 9,409 | 4,972 | 14,381 |
| **Files** | 36 | 40 | 76 |

**Documentation Ratio**: 0.69 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            36           1280            946           7183
-------------------------------------------------------------------------------
SUM:                            36           1280            946           7183
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze/mod.rs` | 182 | 28 | 154 | 84.6% | ✅ |
| `commands/analyze/sections.rs` | 253 | 253 | 0 | 0.0% | ⚠️ Large |
| `commands/astq.rs` | 45 | 45 | 0 | 0.0% | ✅ |
| `commands/git.rs` | 10 | 10 | 0 | 0.0% | ✅ |
| `commands/hook.rs` | 230 | 110 | 120 | 52.2% | ✅ |
| `commands/mod.rs` | 16 | 16 | 0 | 0.0% | ✅ |
| `commands/reflect.rs` | 82 | 82 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 595 | 212 | 383 | 64.4% | ⚠️ Large |
| `commands/wrapped.rs` | 128 | 64 | 64 | 50.0% | ✅ |
| `engine/mod.rs` | 722 | 130 | 592 | 82.0% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `guardrails/mod.rs` | 5 | 5 | 0 | 0.0% | ✅ |
| `guardrails/parser.rs` | 71 | 23 | 48 | 67.6% | ✅ |
| `guardrails/types.rs` | 171 | 98 | 73 | 42.7% | ✅ |
| `main.rs` | 165 | 8 | 157 | 95.2% | ✅ |
| `metrics/aggregation.rs` | 204 | 144 | 60 | 29.4% | ✅ |
| `metrics/graph.rs` | 370 | 222 | 148 | 40.0% | ⚠️ Large |
| `metrics/hooks.rs` | 286 | 176 | 110 | 38.5% | ✅ |
| `metrics/mod.rs` | 333 | 115 | 218 | 65.5% | ✅ |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/evaluator.rs` | 1,491 | 115 | 1,376 | 92.3% | ✅ |
| `rules/interrupt.rs` | 175 | 32 | 143 | 81.7% | ✅ |
| `rules/mod.rs` | 7 | 7 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 297 | 72 | 225 | 75.8% | ✅ |
| `storage/mod.rs` | 522 | 224 | 298 | 57.1% | ⚠️ Large |
| `test_helpers.rs` | 786 | 530 | 256 | 32.6% | ✅ (infra) |
| `tui/app.rs` | 402 | 165 | 237 | 59.0% | ✅ |
| `tui/mod.rs` | 66 | 66 | 0 | 0.0% | ✅ |
| `tui/tabs/events.rs` | 100 | 81 | 19 | 19.0% | ✅ |
| `tui/tabs/files.rs` | 81 | 63 | 18 | 22.2% | ✅ |
| `tui/tabs/mod.rs` | 9 | 9 | 0 | 0.0% | ✅ |
| `tui/tabs/overview.rs` | 88 | 74 | 14 | 15.9% | ✅ |
| `tui/tabs/phases.rs` | 135 | 103 | 32 | 23.7% | ✅ |
| `tui/ui.rs` | 317 | 158 | 159 | 50.2% | ✅ |
| `tui/utils.rs` | 320 | 187 | 133 | 41.6% | ✅ |

**⚠️ Warning:** 4 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 136 |
| `CODE_MAP.md` | 159 |
| `COVERAGE_REPORT.md` | 120 |
| `DEP_REVIEW.md` | 678 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 207 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 142 |
| `README.md` | 335 |
| `ROADMAP.md` | 251 |
| `TESTING.md` | 41 |
| `vendor/ast-grep/.github/CONTRIBUTING.md` | 8 |
| `vendor/ast-grep/.github/copilot-instructions.md` | 135 |
| `vendor/ast-grep/.github/ISSUE_TEMPLATE/feature_request.md` | 20 |
| `vendor/ast-grep/CHANGELOG.md` | 1,700 |
| `vendor/ast-grep/crates/napi/npm/darwin-arm64/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/darwin-x64/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/linux-arm64-gnu/README.md` | 2 |
| `vendor/ast-grep/crates/napi/npm/linux-arm64-musl/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/linux-x64-gnu/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/linux-x64-musl/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/win32-arm64-msvc/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/win32-ia32-msvc/README.md` | 3 |
| `vendor/ast-grep/crates/napi/npm/win32-x64-msvc/README.md` | 3 |
| `vendor/ast-grep/crates/napi/README.md` | 34 |
| `vendor/ast-grep/crates/pyo3/README.md` | 73 |
| `vendor/ast-grep/npm/platforms/darwin-arm64/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/darwin-x64/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/linux-arm64-gnu/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/linux-x64-gnu/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/win32-arm64-msvc/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/win32-ia32-msvc/README.md` | 3 |
| `vendor/ast-grep/npm/platforms/win32-x64-msvc/README.md` | 3 |
| `vendor/ast-grep/npm/README.md` | 11 |
| `vendor/ast-grep/README.md` | 118 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.69 | ✅ Excellent |
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
