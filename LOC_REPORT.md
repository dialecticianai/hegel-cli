# Lines of Code Report

**Last Updated**: 2025-10-09 17:11
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | 1,799 | 2,087 | 3,886 |
| **Comments** | 258 | - | 258 |
| **Blank Lines** | 409 | - | 409 |
| **Total Lines** | 2,466 | 2,087 | 4,553 |
| **Files** | 6 | 14 | 20 |

**Documentation Ratio**: 1.16 lines of docs per line of code

---

## Rust Code Breakdown

```
Language                     files          blank        comment           code
-------------------------------------------------------------------------------
Rust                             6            409            258           1799
-------------------------------------------------------------------------------
SUM:                             6            409            258           1799
-------------------------------------------------------------------------------
```

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % |
|------|-------------|------------|------------|--------|
| `commands/mod.rs` | 673 | 216 | 457 | 67.9% |
| `engine/mod.rs` | 594 | 97 | 497 | 83.7% |
| `engine/template.rs` | 425 | 87 | 338 | 79.5% |
| `main.rs` | 97 | 4 | 93 | 95.9% |
| `storage/mod.rs` | 547 | 154 | 393 | 71.8% |
| `test_helpers.rs` | 130 | 130 | 0 | 0.0% |

---

## Documentation Files

| File | Lines |
|------|-------|
| `CLAUDE.md` | 275 |
| `COVERAGE_REPORT.md` | 68 |
| `guides/CODE_MAP_WRITING.md` | 95 |
| `guides/HANDOFF_WRITING.md` | 142 |
| `guides/KICKOFF_WRITING.md` | 92 |
| `guides/LEARNINGS_WRITING.md` | 92 |
| `guides/PLAN_WRITING.md` | 145 |
| `guides/README_WRITING.md` | 138 |
| `guides/SPEC_WRITING.md` | 111 |
| `LEXICON.md` | 66 |
| `LOC_REPORT.md` | 89 |
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
