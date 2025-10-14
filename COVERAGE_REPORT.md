# Test Coverage Report

**Last Updated**: 2025-10-14 18:50
**Tool**: cargo-llvm-cov
**Overall Coverage**: **92.68%** lines | **91.45%** regions | **87.75%** functions

## Summary

```
TOTAL                              10361               886    91.45%         547                67    87.75%        6378               467    92.68%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `adapters/claude_code.rs` | 97.18% | 96.81% | 90.00% | 🟢 Excellent |
| `adapters/codex.rs` | 92.75% | 91.03% | 77.27% | 🟢 Excellent |
| `adapters/mod.rs` | 60.71% | 53.85% | 50.00% | 🟠 Moderate |
| `commands/analyze/mod.rs` | 100.00% | 99.52% | 100.00% | 🟢 Excellent |
| `commands/analyze/sections.rs` | 97.46% | 95.75% | 100.00% | 🟢 Excellent |
| `commands/astq.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `commands/git.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `commands/hook.rs` | 96.64% | 92.78% | 61.54% | 🟢 Excellent |
| `commands/reflect.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `commands/workflow.rs` | 89.88% | 88.17% | 80.56% | 🟡 Good |
| `commands/wrapped.rs` | 38.46% | 40.72% | 40.00% | 🔴 Needs Work |
| `engine/mod.rs` | 99.80% | 99.41% | 96.77% | 🟢 Excellent |
| `engine/template.rs` | 95.47% | 94.58% | 96.30% | 🟢 Excellent |
| `guardrails/parser.rs` | 97.78% | 97.65% | 83.33% | 🟢 Excellent |
| `guardrails/types.rs` | 94.38% | 93.60% | 100.00% | 🟢 Excellent |
| `main.rs` | 73.91% | 64.65% | 100.00% | 🟡 Good |
| `metrics/aggregation.rs` | 97.18% | 95.50% | 100.00% | 🟢 Excellent |
| `metrics/graph.rs` | 99.20% | 98.51% | 100.00% | 🟢 Excellent |
| `metrics/hooks.rs` | 94.22% | 93.69% | 89.47% | 🟢 Excellent |
| `metrics/mod.rs` | 96.99% | 96.40% | 100.00% | 🟢 Excellent |
| `metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/evaluator.rs` | 99.29% | 98.52% | 100.00% | 🟢 Excellent |
| `rules/interrupt.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/types.rs` | 95.14% | 91.63% | 100.00% | 🟢 Excellent |
| `storage/mod.rs` | 83.64% | 86.86% | 63.64% | 🟡 Good |
| `test_helpers.rs` | 93.89% | 90.69% | 95.45% | 🟢 Excellent |
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
- `adapters/claude_code.rs` - 97.18%
- `adapters/codex.rs` - 92.75%
- `commands/analyze/mod.rs` - 100.00%
- `commands/analyze/sections.rs` - 97.46%
- `commands/hook.rs` - 96.64%
- `engine/mod.rs` - 99.80%
- `engine/template.rs` - 95.47%
- `guardrails/parser.rs` - 97.78%
- `guardrails/types.rs` - 94.38%
- `metrics/aggregation.rs` - 97.18%
- `metrics/graph.rs` - 99.20%
- `metrics/hooks.rs` - 94.22%
- `metrics/mod.rs` - 96.99%
- `metrics/states.rs` - 100.00%
- `metrics/transcript.rs` - 100.00%
- `rules/evaluator.rs` - 99.29%
- `rules/interrupt.rs` - 100.00%
- `rules/types.rs` - 95.14%
- `test_helpers.rs` - 93.89%
- `tui/tabs/overview.rs` - 100.00%
- `tui/tabs/phases.rs` - 95.18%
- `tui/ui.rs` - 100.00%
- `tui/utils.rs` - 96.60%

### 🟡 Good (70-89% lines)
- `commands/workflow.rs` - 89.88%
- `main.rs` - 73.91%
- `storage/mod.rs` - 83.64%
- `tui/app.rs` - 83.83%
- `tui/tabs/events.rs` - 88.33%
- `tui/tabs/files.rs` - 81.63%

### 🟠 Moderate (40-69% lines)
- `adapters/mod.rs` - 60.71%

### 🔴 Needs Work (<40% lines)
- `commands/astq.rs` - 0.00%
- `commands/git.rs` - 0.00%
- `commands/reflect.rs` - 0.00%
- `commands/wrapped.rs` - 38.46%
- `tui/mod.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 92.68% | ✅ Met |
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
