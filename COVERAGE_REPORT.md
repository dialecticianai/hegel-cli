# Test Coverage Report

**Last Updated**: 2025-10-09 04:33
**Tool**: cargo-llvm-cov
**Overall Coverage**: **92.95%** lines | **91.51%** regions | **85.44%** functions

## Summary

```
TOTAL                            2555               217    91.51%         103                15    85.44%        1319                93    92.95%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/mod.rs` | 87.81% | 87.21% | 77.78% | 🟡 Good |
| `engine/mod.rs` | 99.77% | 99.52% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 96.01% | 93.90% | 96.88% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `storage/mod.rs` | 95.97% | 96.33% | 75.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `engine/mod.rs` - 99.77%
- `engine/template.rs` - 96.01%
- `storage/mod.rs` - 95.97%

### 🟡 Good (70-89% lines)
- `commands/mod.rs` - 87.81%

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 92.95% | ✅ Met |
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
