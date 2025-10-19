# Test Coverage Report

**Last Updated**: 2025-10-19 17:15
**Tool**: cargo-llvm-cov
**Overall Coverage**: **91.99%** lines | **91.27%** regions | **88.07%** functions

## Summary

```
TOTAL                              12876              1124    91.27%         729                87    88.07%        7569               606    91.99%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `adapters/claude_code.rs` | 97.18% | 96.81% | 90.00% | 🟢 Excellent |
| `adapters/codex.rs` | 93.44% | 93.92% | 88.24% | 🟢 Excellent |
| `adapters/cursor.rs` | 92.42% | 92.55% | 89.47% | 🟢 Excellent |
| `adapters/mod.rs` | 60.71% | 55.56% | 50.00% | 🟠 Moderate |
| `commands/analyze/mod.rs` | 100.00% | 99.51% | 100.00% | 🟢 Excellent |
| `commands/analyze/sections.rs` | 97.56% | 95.62% | 100.00% | 🟢 Excellent |
| `commands/astq.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `commands/config.rs` | 91.95% | 90.30% | 63.64% | 🟢 Excellent |
| `commands/git.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `commands/hook.rs` | 96.64% | 92.78% | 61.54% | 🟢 Excellent |
| `commands/init.rs` | 86.21% | 84.32% | 92.86% | 🟡 Good |
| `commands/meta.rs` | 86.09% | 85.07% | 90.91% | 🟡 Good |
| `commands/reflect.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `commands/workflow.rs` | 93.68% | 92.19% | 89.04% | 🟢 Excellent |
| `commands/wrapped.rs` | 72.50% | 74.10% | 80.00% | 🟡 Good |
| `config.rs` | 91.67% | 91.04% | 76.92% | 🟢 Excellent |
| `embedded.rs` | 76.81% | 62.26% | 50.00% | 🟡 Good |
| `engine/mod.rs` | 99.80% | 99.58% | 96.88% | 🟢 Excellent |
| `engine/template.rs` | 96.80% | 96.18% | 97.06% | 🟢 Excellent |
| `guardrails/parser.rs` | 97.78% | 97.65% | 83.33% | 🟢 Excellent |
| `guardrails/types.rs` | 94.38% | 93.60% | 100.00% | 🟢 Excellent |
| `main.rs` | 47.73% | 42.16% | 100.00% | 🟠 Moderate |
| `metamodes/mod.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/aggregation.rs` | 97.18% | 95.50% | 100.00% | 🟢 Excellent |
| `metrics/graph.rs` | 99.20% | 98.51% | 100.00% | 🟢 Excellent |
| `metrics/hooks.rs` | 94.22% | 93.69% | 89.47% | 🟢 Excellent |
| `metrics/mod.rs` | 96.99% | 96.40% | 100.00% | 🟢 Excellent |
| `metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/evaluator.rs` | 98.44% | 97.74% | 96.15% | 🟢 Excellent |
| `rules/interrupt.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/types.rs` | 95.14% | 91.63% | 100.00% | 🟢 Excellent |
| `storage/mod.rs` | 96.69% | 96.14% | 80.70% | 🟢 Excellent |
| `test_helpers.rs` | 79.09% | 76.07% | 76.79% | 🟡 Good |
| `theme.rs` | 65.00% | 67.42% | 58.82% | 🟠 Moderate |
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
- `adapters/codex.rs` - 93.44%
- `adapters/cursor.rs` - 92.42%
- `commands/analyze/mod.rs` - 100.00%
- `commands/analyze/sections.rs` - 97.56%
- `commands/config.rs` - 91.95%
- `commands/git.rs` - 100.00%
- `commands/hook.rs` - 96.64%
- `commands/workflow.rs` - 93.68%
- `config.rs` - 91.67%
- `engine/mod.rs` - 99.80%
- `engine/template.rs` - 96.80%
- `guardrails/parser.rs` - 97.78%
- `guardrails/types.rs` - 94.38%
- `metamodes/mod.rs` - 100.00%
- `metrics/aggregation.rs` - 97.18%
- `metrics/graph.rs` - 99.20%
- `metrics/hooks.rs` - 94.22%
- `metrics/mod.rs` - 96.99%
- `metrics/states.rs` - 100.00%
- `metrics/transcript.rs` - 100.00%
- `rules/evaluator.rs` - 98.44%
- `rules/interrupt.rs` - 100.00%
- `rules/types.rs` - 95.14%
- `storage/mod.rs` - 96.69%
- `tui/tabs/overview.rs` - 100.00%
- `tui/tabs/phases.rs` - 95.18%
- `tui/ui.rs` - 100.00%
- `tui/utils.rs` - 96.60%

### 🟡 Good (70-89% lines)
- `commands/init.rs` - 86.21%
- `commands/meta.rs` - 86.09%
- `commands/wrapped.rs` - 72.50%
- `embedded.rs` - 76.81%
- `test_helpers.rs` - 79.09%
- `tui/app.rs` - 83.83%
- `tui/tabs/events.rs` - 88.33%
- `tui/tabs/files.rs` - 81.63%

### 🟠 Moderate (40-69% lines)
- `adapters/mod.rs` - 60.71%
- `main.rs` - 47.73%
- `theme.rs` - 65.00%

### 🔴 Needs Work (<40% lines)
- `commands/astq.rs` - 0.00%
- `commands/reflect.rs` - 0.00%
- `tui/mod.rs` - 0.00%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 91.99% | ✅ Met |
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
