# Test Coverage Report

**Last Updated**: 2025-10-10 23:30
**Tool**: cargo-llvm-cov
**Overall Coverage**: **89.31%** lines | **88.63%** regions | **87.65%** functions

## Summary

```
TOTAL                            6298               716    88.63%         340                42    87.65%        3554               380    89.31%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/analyze.rs` | 82.22% | 88.36% | 100.00% | 🟡 Good |
| `commands/hook.rs` | 89.54% | 87.23% | 64.71% | 🟡 Good |
| `commands/workflow.rs` | 96.49% | 93.40% | 91.67% | 🟢 Excellent |
| `engine/mod.rs` | 99.67% | 99.43% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 95.47% | 94.58% | 96.30% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `metrics/graph.rs` | 99.20% | 98.76% | 100.00% | 🟢 Excellent |
| `metrics/hooks.rs` | 95.30% | 95.83% | 94.12% | 🟢 Excellent |
| `metrics/mod.rs` | 97.36% | 95.94% | 100.00% | 🟢 Excellent |
| `metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `storage/mod.rs` | 93.21% | 93.35% | 69.39% | 🟢 Excellent |
| `test_helpers.rs` | 90.10% | 87.47% | 90.48% | 🟢 Excellent |
| `tui/app.rs` | 83.64% | 84.52% | 90.91% | 🟡 Good |
| `tui/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `tui/ui.rs` | 69.32% | 64.29% | 78.95% | 🟠 Moderate |
| `tui/utils.rs` | 96.60% | 96.92% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/workflow.rs` - 96.49%
- `engine/mod.rs` - 99.67%
- `engine/template.rs` - 95.47%
- `metrics/graph.rs` - 99.20%
- `metrics/hooks.rs` - 95.30%
- `metrics/mod.rs` - 97.36%
- `metrics/states.rs` - 100.00%
- `metrics/transcript.rs` - 100.00%
- `storage/mod.rs` - 93.21%
- `test_helpers.rs` - 90.10%
- `tui/utils.rs` - 96.60%

### 🟡 Good (70-89% lines)
- `commands/analyze.rs` - 82.22%
- `commands/hook.rs` - 89.54%
- `tui/app.rs` - 83.64%

### 🟠 Moderate (40-69% lines)
- `tui/ui.rs` - 69.32%

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%
- `tui/mod.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 89.31% | ✅ Met |
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
