# Test Coverage Report

**Last Updated**: 2025-10-09 19:20
**Tool**: cargo-llvm-cov
**Overall Coverage**: **85.16%** lines | **85.06%** regions | **81.82%** functions

## Summary

```
TOTAL                            3259               487    85.06%         176                32    81.82%        1752               260    85.16%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/analyze.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `commands/hook.rs` | 80.52% | 80.39% | 40.00% | 🟡 Good |
| `commands/workflow.rs` | 96.49% | 93.40% | 91.67% | 🟢 Excellent |
| `engine/mod.rs` | 99.67% | 99.43% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 95.47% | 94.58% | 96.30% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `metrics/mod.rs` | 81.99% | 82.81% | 79.31% | 🟡 Good |
| `storage/mod.rs` | 92.47% | 93.12% | 70.00% | 🟢 Excellent |
| `test_helpers.rs` | 97.64% | 95.60% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/workflow.rs` - 96.49%
- `engine/mod.rs` - 99.67%
- `engine/template.rs` - 95.47%
- `storage/mod.rs` - 92.47%
- `test_helpers.rs` - 97.64%

### 🟡 Good (70-89% lines)
- `commands/hook.rs` - 80.52%
- `metrics/mod.rs` - 81.99%

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `commands/analyze.rs` - 0.00%
- `main.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 85.16% | ✅ Met |
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
