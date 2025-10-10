# Lines of Code Report

**Last Updated**: 2025-10-10 16:43
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 2,594 | 2,079 | 4,673 |
| **Comments** | 383 | - | 383 |
| **Blank Lines** | 467 | - | 467 |
| **Total Lines** | 3,444 | 2,079 | 5,523 |
| **Files** | 13 | 15 | 28 |

**Documentation Ratio**: 0.80 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            13            467            383           2594
-------------------------------------------------------------------------------
SUM:                            13            467            383           2594
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze.rs` | 216 | 216 | 0 | 0.0% | ⚠️ Large |
| `commands/hook.rs` | 127 | 72 | 55 | 43.3% | ✅ |
| `commands/mod.rs` | 8 | 8 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 400 | 158 | 242 | 60.5% | ✅ |
| `engine/mod.rs` | 452 | 97 | 355 | 78.5% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 103 | 5 | 98 | 95.1% | ✅ |
| `metrics/hooks.rs` | 291 | 181 | 110 | 37.8% | ✅ |
| `metrics/mod.rs` | 394 | 215 | 179 | 45.4% | ⚠️ Large |
| `metrics/states.rs` | 67 | 33 | 34 | 50.7% | ✅ |
| `metrics/transcript.rs` | 172 | 100 | 72 | 41.9% | ✅ |
| `storage/mod.rs` | 461 | 173 | 288 | 62.5% | ✅ |
| `test_helpers.rs` | 402 | 402 | 0 | 0.0% | ✅ (infra) |

**⚠️ Warning:** 2 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 117 |
| `CODE_MAP.md` | 128 |
| `COVERAGE_REPORT.md` | 80 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 97 |
| `PLAN.md` | 436 |
| `README.md` | 203 |
| `ROADMAP.md` | 119 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.80 | ✅ Excellent |
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
