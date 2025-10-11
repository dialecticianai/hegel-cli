# Test Coverage Report

**Last Updated**: 2025-10-10 20:55
**Tool**: cargo-llvm-cov
**Overall Coverage**: **90.19%** lines | **89.05%** regions | **87.96%** functions

## Summary

```
TOTAL                            6044               662    89.05%         324                39    87.96%        3404               334    90.19%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/analyze.rs` | 98.15% | 97.09% | 100.00% | 🟢 Excellent |
| `commands/hook.rs` | 80.52% | 80.39% | 40.00% | 🟡 Good |
| `commands/workflow.rs` | 96.49% | 93.40% | 91.67% | 🟢 Excellent |
| `engine/mod.rs` | 99.67% | 99.43% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 95.47% | 94.58% | 96.30% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `metrics/graph.rs` | 99.20% | 98.76% | 100.00% | 🟢 Excellent |
| `metrics/hooks.rs` | 95.30% | 95.83% | 94.12% | 🟢 Excellent |
| `metrics/mod.rs` | 96.94% | 95.77% | 100.00% | 🟢 Excellent |
| `metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `storage/mod.rs` | 92.47% | 93.12% | 70.00% | 🟢 Excellent |
| `test_helpers.rs` | 90.10% | 87.47% | 90.48% | 🟢 Excellent |
| `tui/app.rs` | 83.64% | 84.52% | 90.91% | 🟡 Good |
| `tui/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `tui/ui.rs` | 69.32% | 64.29% | 78.95% | 🟠 Moderate |
| `tui/utils.rs` | 96.60% | 96.92% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/analyze.rs` - 98.15%
- `commands/workflow.rs` - 96.49%
- `engine/mod.rs` - 99.67%
- `engine/template.rs` - 95.47%
- `metrics/graph.rs` - 99.20%
- `metrics/hooks.rs` - 95.30%
- `metrics/mod.rs` - 96.94%
- `metrics/states.rs` - 100.00%
- `metrics/transcript.rs` - 100.00%
- `storage/mod.rs` - 92.47%
- `test_helpers.rs` - 90.10%
- `tui/utils.rs` - 96.60%

### 🟡 Good (70-89% lines)
- `commands/hook.rs` - 80.52%
- `tui/app.rs` - 83.64%

### 🟠 Moderate (40-69% lines)
- `tui/ui.rs` - 69.32%

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%
- `tui/mod.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 90.19% | ✅ Met |
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
