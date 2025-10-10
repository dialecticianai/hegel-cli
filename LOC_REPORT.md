# Lines of Code Report

**Last Updated**: 2025-10-10 17:02
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 2,871 | 2,089 | 4,960 |
| **Comments** | 449 | - | 449 |
| **Blank Lines** | 522 | - | 522 |
| **Total Lines** | 3,842 | 2,089 | 5,931 |
| **Files** | 13 | 15 | 28 |

**Documentation Ratio**: 0.73 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            13            522            449           2871
-------------------------------------------------------------------------------
SUM:                            13            522            449           2871
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze.rs` | 371 | 217 | 154 | 41.5% | ⚠️ Large |
| `commands/hook.rs` | 127 | 72 | 55 | 43.3% | ✅ |
| `commands/mod.rs` | 8 | 8 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 400 | 158 | 242 | 60.5% | ✅ |
| `engine/mod.rs` | 452 | 97 | 355 | 78.5% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 103 | 5 | 98 | 95.1% | ✅ |
| `metrics/hooks.rs` | 291 | 181 | 110 | 37.8% | ✅ |
| `metrics/mod.rs` | 394 | 215 | 179 | 45.4% | ⚠️ Large |
| `metrics/states.rs` | 137 | 33 | 104 | 75.9% | ✅ |
| `metrics/transcript.rs` | 257 | 100 | 157 | 61.1% | ✅ |
| `storage/mod.rs` | 461 | 173 | 288 | 62.5% | ✅ |
| `test_helpers.rs` | 490 | 490 | 0 | 0.0% | ✅ (infra) |

**⚠️ Warning:** 2 file(s) over 200 impl lines - consider splitting for maintainability

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 117 |
| `CODE_MAP.md` | 129 |
| `COVERAGE_REPORT.md` | 80 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 84 |
| `LOC_REPORT.md` | 99 |
| `PLAN.md` | 436 |
| `README.md` | 210 |
| `ROADMAP.md` | 119 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.73 | ✅ Excellent |
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
