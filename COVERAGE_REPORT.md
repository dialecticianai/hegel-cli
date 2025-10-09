# Test Coverage Report

**Last Updated**: 2025-10-09 19:13
**Tool**: cargo-llvm-cov
**Overall Coverage**: **91.71%** lines | **90.63%** regions | **84.71%** functions

## Summary

```
TOTAL                            3042               285    90.63%         170                26    84.71%        1616               134    91.71%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/hook.rs` | 81.69% | 81.82% | 50.00% | 🟡 Good |
| `commands/workflow.rs` | 96.49% | 93.40% | 91.67% | 🟢 Excellent |
| `engine/mod.rs` | 99.67% | 99.43% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 95.47% | 94.58% | 96.30% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `metrics/mod.rs` | 83.40% | 83.20% | 76.67% | 🟡 Good |
| `storage/mod.rs` | 93.01% | 93.72% | 73.68% | 🟢 Excellent |
| `test_helpers.rs` | 97.64% | 95.60% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/workflow.rs` - 96.49%
- `engine/mod.rs` - 99.67%
- `engine/template.rs` - 95.47%
- `storage/mod.rs` - 93.01%
- `test_helpers.rs` - 97.64%

### 🟡 Good (70-89% lines)
- `commands/hook.rs` - 81.69%
- `metrics/mod.rs` - 83.40%

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 91.71% | ✅ Met |
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
