# Test Coverage Report

**Last Updated**: 2025-10-08 23:48
**Tool**: cargo-llvm-cov
**Overall Coverage**: **15.99%** lines | **18.28%** regions | **19.35%** functions

## Summary

```
TOTAL                             547               447    18.28%          31                25    19.35%         269               226    15.99%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `engine/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `engine/template.rs` | 86.00% | 81.30% | 85.71% | 🟡 Good |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `storage/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)

### 🟡 Good (70-89% lines)
- `engine/template.rs` - 86.00%

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `commands/mod.rs` - 0.00%
- `engine/mod.rs` - 0.00%
- `main.rs` - 0.00%
- `storage/mod.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 15.99% | ⏳ In Progress |
| Critical Paths | ≥95% | Check modules above | Policy |
| New Modules | ≥80% | - | Policy |

## How to Update This Report

```bash
# Regenerate coverage report
./scripts/generate-coverage-report.sh
```

## Quick Commands

```bash
# Run tests with coverage
cargo llvm-cov --html      # Detailed HTML
cargo llvm-cov --summary-only  # Terminal summary

# Update this markdown report
./scripts/generate-coverage-report.sh
```

---

*This report is auto-generated from `cargo llvm-cov` output.*
