# Test Coverage Report

**Last Updated**: 2025-10-09 16:58
**Tool**: cargo-llvm-cov
**Overall Coverage**: **94.18%** lines | **92.95%** regions | **85.48%** functions

## Summary

```
TOTAL                            3221               227    92.95%         124                18    85.48%        1599                93    94.18%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/mod.rs` | 93.35% | 91.99% | 82.86% | 🟢 Excellent |
| `engine/mod.rs` | 99.78% | 99.52% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 96.01% | 93.90% | 96.88% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `storage/mod.rs` | 95.19% | 95.74% | 75.68% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/mod.rs` - 93.35%
- `engine/mod.rs` - 99.78%
- `engine/template.rs` - 96.01%
- `storage/mod.rs` - 95.19%

### 🟡 Good (70-89% lines)

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 94.18% | ✅ Met |
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
