# Test Coverage Report

**Last Updated**: 2025-10-10 17:02
**Tool**: cargo-llvm-cov
**Overall Coverage**: **94.97%** lines | **94.05%** regions | **89.29%** functions

## Summary

```
TOTAL                            4136               246    94.05%         224                24    89.29%        2247               113    94.97%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `commands/analyze.rs` | 100.00% | 99.54% | 100.00% | 🟢 Excellent |
| `commands/hook.rs` | 80.52% | 80.39% | 40.00% | 🟡 Good |
| `commands/workflow.rs` | 96.49% | 93.40% | 91.67% | 🟢 Excellent |
| `engine/mod.rs` | 99.67% | 99.43% | 94.74% | 🟢 Excellent |
| `engine/template.rs` | 95.47% | 94.58% | 96.30% | 🟢 Excellent |
| `main.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `metrics/hooks.rs` | 92.81% | 94.40% | 94.12% | 🟢 Excellent |
| `metrics/mod.rs` | 96.94% | 95.77% | 100.00% | 🟢 Excellent |
| `metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `storage/mod.rs` | 92.47% | 93.12% | 70.00% | 🟢 Excellent |
| `test_helpers.rs` | 98.26% | 96.91% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `commands/analyze.rs` - 100.00%
- `commands/workflow.rs` - 96.49%
- `engine/mod.rs` - 99.67%
- `engine/template.rs` - 95.47%
- `metrics/hooks.rs` - 92.81%
- `metrics/mod.rs` - 96.94%
- `metrics/states.rs` - 100.00%
- `metrics/transcript.rs` - 100.00%
- `storage/mod.rs` - 92.47%
- `test_helpers.rs` - 98.26%

### 🟡 Good (70-89% lines)
- `commands/hook.rs` - 80.52%

### 🟠 Moderate (40-69% lines)

### 🔴 Needs Work (<40% lines)
- `main.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 94.97% | ✅ Met |
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
