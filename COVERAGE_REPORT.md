# Test Coverage Report

**Last Updated**: 2025-10-09 17:11
**Tool**: cargo-llvm-cov
**Overall Coverage**: **93.93%** lines | **92.64%** regions | **86.15%** functions

## Summary

```
TOTAL                            3031               223    92.64%         130                18    86.15%        1531                93    93.93%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/mod.rs` | 93.41% | 92.29% | 82.86% | 🟢 Excellent |
| `engine/mod.rs` | 99.78% | 99.52% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 96.01% | 93.90% | 96.88% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `storage/mod.rs` | 94.35% | 94.74% | 75.00% | 🟢 Excellent |
| `test_helpers.rs` | 90.62% | 88.33% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/mod.rs` - 93.41%
- `engine/mod.rs` - 99.78%
- `engine/template.rs` - 96.01%
- `storage/mod.rs` - 94.35%
- `test_helpers.rs` - 90.62%

### 🟡 Good (70-89% lines)

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 93.93% | ✅ Met |
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
