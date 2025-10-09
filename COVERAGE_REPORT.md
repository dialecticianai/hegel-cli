# Test Coverage Report

**Last Updated**: 2025-10-09 00:03
**Tool**: cargo-llvm-cov
**Overall Coverage**: **95.41%** lines | **93.33%** regions | **88.00%** functions

## Summary

```
TOTAL                            2505               167    93.33%         100                12    88.00%        1285                59    95.41%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/mod.rs` | 95.58% | 92.84% | 91.30% | 🟢 Excellent |
| `engine/mod.rs` | 99.77% | 99.52% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 96.01% | 93.90% | 96.88% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `storage/mod.rs` | 95.58% | 96.33% | 72.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/mod.rs` - 95.58%
- `engine/mod.rs` - 99.77%
- `engine/template.rs` - 96.01%
- `storage/mod.rs` - 95.58%

### 🟡 Good (70-89% lines)

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 95.41% | ✅ Met |
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
