# Test Coverage Report

**Last Updated**: 2025-10-11 05:02
**Tool**: cargo-llvm-cov
**Overall Coverage**: **95.51%** lines | **94.30%** regions | **91.76%** functions

## Summary

```
TOTAL                               8831               503    94.30%         449                37    91.76%        5432               244    95.51%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/analyze/mod.rs` | 100.00% | 99.52% | 100.00% | 🟢 Excellent |
| `commands/analyze/sections.rs` | 97.46% | 95.75% | 100.00% | 🟢 Excellent |
| `commands/hook.rs` | 95.45% | 92.83% | 70.59% | 🟢 Excellent |
| `commands/workflow.rs` | 89.88% | 88.17% | 80.56% | 🟡 Good |
| `engine/mod.rs` | 99.80% | 99.41% | 96.77% | 🟢 Excellent |
| `engine/template.rs` | 95.47% | 94.58% | 96.30% | 🟢 Excellent |
| `main.rs` | 97.14% | 88.89% | 100.00% | 🟢 Excellent |
| `metrics/aggregation.rs` | 97.18% | 95.50% | 100.00% | 🟢 Excellent |
| `metrics/graph.rs` | 99.20% | 98.76% | 100.00% | 🟢 Excellent |
| `metrics/hooks.rs` | 95.30% | 95.83% | 94.12% | 🟢 Excellent |
| `metrics/mod.rs` | 96.99% | 96.40% | 100.00% | 🟢 Excellent |
| `metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/evaluator.rs` | 99.29% | 98.52% | 100.00% | 🟢 Excellent |
| `rules/interrupt.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/types.rs` | 95.14% | 91.63% | 100.00% | 🟢 Excellent |
| `storage/mod.rs` | 92.49% | 93.13% | 70.00% | 🟢 Excellent |
| `test_helpers.rs` | 93.89% | 90.47% | 95.45% | 🟢 Excellent |
| `tui/app.rs` | 83.83% | 84.44% | 87.50% | 🟡 Good |
| `tui/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `tui/tabs/events.rs` | 88.33% | 83.33% | 100.00% | 🟡 Good |
| `tui/tabs/files.rs` | 81.63% | 78.72% | 100.00% | 🟡 Good |
| `tui/tabs/overview.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `tui/tabs/phases.rs` | 95.18% | 95.04% | 100.00% | 🟢 Excellent |
| `tui/ui.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `tui/utils.rs` | 96.60% | 96.92% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/analyze/mod.rs` - 100.00%
- `commands/analyze/sections.rs` - 97.46%
- `commands/hook.rs` - 95.45%
- `engine/mod.rs` - 99.80%
- `engine/template.rs` - 95.47%
- `main.rs` - 97.14%
- `metrics/aggregation.rs` - 97.18%
- `metrics/graph.rs` - 99.20%
- `metrics/hooks.rs` - 95.30%
- `metrics/mod.rs` - 96.99%
- `metrics/states.rs` - 100.00%
- `metrics/transcript.rs` - 100.00%
- `rules/evaluator.rs` - 99.29%
- `rules/interrupt.rs` - 100.00%
- `rules/types.rs` - 95.14%
- `storage/mod.rs` - 92.49%
- `test_helpers.rs` - 93.89%
- `tui/tabs/overview.rs` - 100.00%
- `tui/tabs/phases.rs` - 95.18%
- `tui/ui.rs` - 100.00%
- `tui/utils.rs` - 96.60%

### 🟡 Good (70-89% lines)
- `commands/workflow.rs` - 89.88%
- `tui/app.rs` - 83.83%
- `tui/tabs/events.rs` - 88.33%
- `tui/tabs/files.rs` - 81.63%

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `tui/mod.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 95.51% | ✅ Met |
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
