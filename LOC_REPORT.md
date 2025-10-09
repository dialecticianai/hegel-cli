# Lines of Code Report

**Last Updated**: 2025-10-09 19:08
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 1,883 | 2,191 | 4,074 |
| **Comments** | 268 | - | 268 |
| **Blank Lines** | 347 | - | 347 |
| **Total Lines** | 2,498 | 2,191 | 4,689 |
| **Files** | 9 | 15 | 24 |

**Documentation Ratio**: 1.16 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                             9            347            268           1883
-------------------------------------------------------------------------------
SUM:                             9            347            268           1883
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
| `commands/hook.rs` | 117 | 62 | 55 | 47.0% | ✅ |
| `commands/mod.rs` | 6 | 6 | 0 | 0.0% | ✅ |
| `commands/workflow.rs` | 400 | 158 | 242 | 60.5% | ✅ |
| `engine/mod.rs` | 452 | 97 | 355 | 78.5% | ✅ |
| `engine/template.rs` | 351 | 87 | 264 | 75.2% | ✅ |
| `main.rs` | 98 | 5 | 93 | 94.9% | ✅ |
| `metrics/mod.rs` | 283 | 165 | 118 | 41.7% | ✅ |
| `storage/mod.rs` | 450 | 162 | 288 | 64.0% | ✅ |
| `test_helpers.rs` | 341 | 341 | 0 | 0.0% | ✅ (infra) |

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 282 |
| `CODE_MAP.md` | 76 |
| `COVERAGE_REPORT.md` | 72 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 81 |
| `LOC_REPORT.md` | 91 |
| `PLAN.md` | 436 |
| `README.md` | 190 |
| `ROADMAP.md` | 148 |

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Docs/Code Ratio | ≥0.3 | 1.16 | ✅ Excellent |
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
