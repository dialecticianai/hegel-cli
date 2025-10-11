# Test Coverage Report

**Last Updated**: 2025-10-11 01:01
**Tool**: cargo-llvm-cov
**Overall Coverage**: **91.34%** lines | **89.84%** regions | **88.98%** functions

## Summary

```
TOTAL                            6887               700    89.84%         363                40    88.98%        4098               355    91.34%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/analyze.rs` | 98.15% | 97.09% | 100.00% | 🟢 Excellent |
| `commands/hook.rs` | 89.61% | 87.10% | 64.71% | 🟡 Good |
| `commands/workflow.rs` | 96.77% | 93.77% | 92.00% | 🟢 Excellent |
| `engine/mod.rs` | 99.67% | 99.43% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 95.47% | 94.58% | 96.30% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `metrics/graph.rs` | 99.20% | 98.76% | 100.00% | 🟢 Excellent |
| `metrics/hooks.rs` | 95.30% | 95.83% | 94.12% | 🟢 Excellent |
| `metrics/mod.rs` | 96.67% | 95.57% | 100.00% | 🟢 Excellent |
| `metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/evaluator.rs` | 97.53% | 97.81% | 92.31% | 🟢 Excellent |
| `rules/types.rs` | 95.14% | 91.63% | 100.00% | 🟢 Excellent |
| `storage/mod.rs` | 92.49% | 93.13% | 70.00% | 🟢 Excellent |
| `test_helpers.rs` | 90.14% | 87.47% | 90.48% | 🟢 Excellent |
| `tui/app.rs` | 83.64% | 84.52% | 90.91% | 🟡 Good |
| `tui/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `tui/ui.rs` | 69.32% | 64.29% | 78.95% | 🟠 Moderate |
| `tui/utils.rs` | 96.60% | 96.92% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/analyze.rs` - 98.15%
- `commands/workflow.rs` - 96.77%
- `engine/mod.rs` - 99.67%
- `engine/template.rs` - 95.47%
- `metrics/graph.rs` - 99.20%
- `metrics/hooks.rs` - 95.30%
- `metrics/mod.rs` - 96.67%
- `metrics/states.rs` - 100.00%
- `metrics/transcript.rs` - 100.00%
- `rules/evaluator.rs` - 97.53%
- `rules/types.rs` - 95.14%
- `storage/mod.rs` - 92.49%
- `test_helpers.rs` - 90.14%
- `tui/utils.rs` - 96.60%

### 🟡 Good (70-89% lines)
- `commands/hook.rs` - 89.61%
- `tui/app.rs` - 83.64%

### 🟠 Moderate (40-69% lines)
- `tui/ui.rs` - 69.32%

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%
- `tui/mod.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 91.34% | ✅ Met |
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
