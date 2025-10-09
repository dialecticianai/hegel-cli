# Lines of Code Report

**Last Updated**: 2025-10-09 19:45
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 2,271 | 2,212 | 4,483 |
| **Comments** | 350 | - | 350 |
| **Blank Lines** | 413 | - | 413 |
| **Total Lines** | 3,034 | 2,212 | 5,246 |
| **Files** | 13 | 15 | 28 |

**Documentation Ratio**: 0.97 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                            13            413            350           2271
-------------------------------------------------------------------------------
SUM:                            13            413            350           2271
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/analyze.rs` | 146 | 146 | 0 | 0.0% | ✅ |
| `commands/hook.rs` | 127 | 72 | 55 | 43.3% | ✅ |
| `commands/mod.rs` | 8 | 8 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 400 | 158 | 242 | 60.5% | ✅ |
| `engine/mod.rs` | 452 | 97 | 355 | 78.5% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 103 | 5 | 98 | 95.1% | ✅ |
| `metrics/hooks.rs` | 291 | 181 | 110 | 37.8% | ✅ |
| `metrics/mod.rs` | 58 | 58 | 0 | 0.0% | ✅ |
| `metrics/states.rs` | 67 | 33 | 34 | 50.7% | ✅ |
| `metrics/transcript.rs` | 168 | 96 | 72 | 42.9% | ✅ |
| `storage/mod.rs` | 461 | 173 | 288 | 62.5% | ✅ |
| `test_helpers.rs` | 402 | 402 | 0 | 0.0% | ✅ (infra) |

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 282 |
| `CODE_MAP.md` | 85 |
| `COVERAGE_REPORT.md` | 80 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 81 |
| `LOC_REPORT.md` | 96 |
| `PLAN.md` | 436 |
| `README.md` | 203 |
| `ROADMAP.md` | 134 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 0.97 | ✅ Excellent |
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
