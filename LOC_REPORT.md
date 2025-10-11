# Lines of Code Report

**Last Updated**: 2025-10-11 00:56
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 4,646 | 4,187 | 8,833 |
| **Comments** | 755 | - | 755 |
| **Blank Lines** | 836 | - | 836 |
| **Total Lines** | 6,237 | 4,187 | 10,424 |
| **Files** | 20 | 17 | 37 |

**Documentation Ratio**: 0.90 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            20            836            755           4646
-------------------------------------------------------------------------------
SUM:                            20            836            755           4646
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze.rs` | 404 | 250 | 154 | 38.1% | ⚠️ Large |
| `commands/hook.rs` | 230 | 110 | 120 | 52.2% | ✅ |
| `commands/mod.rs` | 8 | 8 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 439 | 170 | 269 | 61.3% | ✅ |
| `engine/mod.rs` | 452 | 97 | 355 | 78.5% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 110 | 7 | 103 | 93.6% | ✅ |
| `metrics/graph.rs` | 370 | 222 | 148 | 40.0% | ⚠️ Large |
| `metrics/hooks.rs` | 286 | 176 | 110 | 38.5% | ✅ |
| `metrics/mod.rs` | 465 | 247 | 218 | 46.9% | ⚠️ Large |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `rules/mod.rs` | 3 | 3 | 0 | 0.0% | ✅ |
| `rules/types.rs` | 202 | 45 | 157 | 77.7% | ✅ |
| `storage/mod.rs` | 472 | 184 | 288 | 61.0% | ✅ |
| `test_helpers.rs` | 750 | 494 | 256 | 34.1% | ✅ (infra) |
| `tui/app.rs` | 369 | 165 | 204 | 55.3% | ✅ |
| `tui/mod.rs` | 65 | 65 | 0 | 0.0% | ✅ |
| `tui/ui.rs` | 547 | 449 | 98 | 17.9% | ⚠️ Large |
| `tui/utils.rs` | 320 | 187 | 133 | 41.6% | ✅ |

**⚠️ Warning:** 4 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 117 |
| `CODE_MAP.md` | 143 |
| `COVERAGE_REPORT.md` | 92 |
| `DEP_REVIEW.md` | 678 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 105 |
| `PLAN.md` | 1,348 |
| `README.md` | 236 |
| `ROADMAP.md` | 95 |
| `SPEC.md` | 474 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.90 | ✅ Excellent |
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
