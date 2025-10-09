# Test Coverage Report

**Last Updated**: 2025-10-08 23:54
**Tool**: cargo-llvm-cov
**Overall Coverage**: **86.30%** lines | **83.71%** regions | **79.17%** functions

## Summary

```
TOTAL                            1842               300    83.71%          72                15    79.17%         993               136    86.30%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `engine/mod.rs` | 99.77% | 99.52% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `storage/mod.rs` | 95.58% | 96.33% | 72.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `engine/mod.rs` - 99.77%
- `engine/template.rs` - 100.00%
- `storage/mod.rs` - 95.58%

### 🟡 Good (70-89% lines)

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `commands/mod.rs` - 0.00%
- `main.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 86.30% | ✅ Met |
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
