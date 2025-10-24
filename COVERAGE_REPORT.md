# Test Coverage Report

**Last Updated**: 2025-10-24 12:39
**Tool**: cargo-llvm-cov
**Overall Coverage**: **91.66%** lines | **90.49%** regions | **88.86%** functions

## Summary

```
TOTAL                                  14046              1336    90.49%         763                85    88.86%        8414               702    91.66%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `adapters/claude_code.rs` | 93.82% | 94.29% | 90.00% | 🟢 Excellent |
| `adapters/codex.rs` | 94.43% | 94.51% | 91.18% | 🟢 Excellent |
| `adapters/cursor.rs` | 93.84% | 93.41% | 94.74% | 🟢 Excellent |
| `adapters/mod.rs` | 98.91% | 99.35% | 100.00% | 🟢 Excellent |
| `commands/analyze/mod.rs` | 100.00% | 99.51% | 100.00% | 🟢 Excellent |
| `commands/analyze/sections.rs` | 97.56% | 95.62% | 100.00% | 🟢 Excellent |
| `commands/archive.rs` | 59.30% | 62.27% | 64.29% | 🟠 Moderate |
| `commands/astq.rs` | 38.57% | 35.71% | 60.00% | 🔴 Needs Work |
| `commands/config.rs` | 91.95% | 90.30% | 63.64% | 🟢 Excellent |
| `commands/fork.rs` | 95.65% | 96.23% | 100.00% | 🟢 Excellent |
| `commands/git.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `commands/hook.rs` | 96.64% | 92.78% | 61.54% | 🟢 Excellent |
| `commands/hooks_setup.rs` | 86.11% | 84.94% | 100.00% | 🟡 Good |
| `commands/init.rs` | 86.21% | 84.32% | 92.86% | 🟡 Good |
| `commands/meta.rs` | 80.75% | 80.69% | 83.33% | 🟡 Good |
| `commands/reflect.rs` | 52.70% | 52.94% | 100.00% | 🟠 Moderate |
| `commands/workflow/claims.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `commands/workflow/context.rs` | 100.00% | 93.06% | 100.00% | 🟢 Excellent |
| `commands/workflow/mod.rs` | 54.92% | 55.53% | 54.17% | 🟠 Moderate |
| `commands/workflow/transitions.rs` | 92.39% | 88.24% | 50.00% | 🟢 Excellent |
| `commands/wrapped.rs` | 72.50% | 74.10% | 80.00% | 🟡 Good |
| `config.rs` | 91.67% | 91.04% | 76.92% | 🟢 Excellent |
| `embedded.rs` | 79.22% | 63.64% | 50.00% | 🟡 Good |
| `engine/mod.rs` | 99.82% | 99.59% | 97.14% | 🟢 Excellent |
| `engine/template.rs` | 97.25% | 96.88% | 97.50% | 🟢 Excellent |
| `guardrails/parser.rs` | 97.78% | 97.65% | 83.33% | 🟢 Excellent |
| `guardrails/types.rs` | 94.38% | 93.60% | 100.00% | 🟢 Excellent |
| `main.rs` | 43.14% | 36.36% | 100.00% | 🟠 Moderate |
| `metamodes/mod.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/aggregation.rs` | 97.18% | 95.50% | 100.00% | 🟢 Excellent |
| `metrics/graph.rs` | 99.20% | 98.76% | 100.00% | 🟢 Excellent |
| `metrics/hooks.rs` | 94.22% | 93.69% | 89.47% | 🟢 Excellent |
| `metrics/mod.rs` | 98.63% | 97.60% | 100.00% | 🟢 Excellent |
| `metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/evaluator.rs` | 98.44% | 97.74% | 96.15% | 🟢 Excellent |
| `rules/interrupt.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `rules/types.rs` | 95.68% | 92.07% | 100.00% | 🟢 Excellent |
| `storage/archive.rs` | 90.20% | 89.15% | 69.23% | 🟢 Excellent |
| `storage/mod.rs` | 96.68% | 96.09% | 81.03% | 🟢 Excellent |
| `test_helpers/fixtures.rs` | 80.00% | 77.78% | 33.33% | 🟡 Good |
| `test_helpers/jsonl.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `test_helpers/metrics.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `test_helpers/storage.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `test_helpers/tui.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `test_helpers/workflow.rs` | 86.11% | 81.55% | 87.50% | 🟡 Good |
| `theme.rs` | 92.86% | 93.75% | 90.91% | 🟢 Excellent |
| `tui/app.rs` | 87.66% | 87.90% | 87.50% | 🟡 Good |
| `tui/mod.rs` | 11.36% | 13.70% | 16.67% | 🔴 Needs Work |
| `tui/tabs/events.rs` | 90.24% | 87.10% | 100.00% | 🟢 Excellent |
| `tui/tabs/files.rs` | 83.93% | 80.37% | 100.00% | 🟡 Good |
| `tui/tabs/overview.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `tui/tabs/phases.rs` | 95.51% | 95.30% | 100.00% | 🟢 Excellent |
| `tui/ui.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `tui/utils.rs` | 97.04% | 93.61% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `adapters/claude_code.rs` - 93.82%
- `adapters/codex.rs` - 94.43%
- `adapters/cursor.rs` - 93.84%
- `adapters/mod.rs` - 98.91%
- `commands/analyze/mod.rs` - 100.00%
- `commands/analyze/sections.rs` - 97.56%
- `commands/config.rs` - 91.95%
- `commands/fork.rs` - 95.65%
- `commands/git.rs` - 100.00%
- `commands/hook.rs` - 96.64%
- `commands/workflow/claims.rs` - 100.00%
- `commands/workflow/context.rs` - 100.00%
- `commands/workflow/transitions.rs` - 92.39%
- `config.rs` - 91.67%
- `engine/mod.rs` - 99.82%
- `engine/template.rs` - 97.25%
- `guardrails/parser.rs` - 97.78%
- `guardrails/types.rs` - 94.38%
- `metamodes/mod.rs` - 100.00%
- `metrics/aggregation.rs` - 97.18%
- `metrics/graph.rs` - 99.20%
- `metrics/hooks.rs` - 94.22%
- `metrics/mod.rs` - 98.63%
- `metrics/states.rs` - 100.00%
- `metrics/transcript.rs` - 100.00%
- `rules/evaluator.rs` - 98.44%
- `rules/interrupt.rs` - 100.00%
- `rules/types.rs` - 95.68%
- `storage/archive.rs` - 90.20%
- `storage/mod.rs` - 96.68%
- `test_helpers/jsonl.rs` - 100.00%
- `test_helpers/metrics.rs` - 100.00%
- `test_helpers/storage.rs` - 100.00%
- `test_helpers/tui.rs` - 100.00%
- `theme.rs` - 92.86%
- `tui/tabs/events.rs` - 90.24%
- `tui/tabs/overview.rs` - 100.00%
- `tui/tabs/phases.rs` - 95.51%
- `tui/ui.rs` - 100.00%
- `tui/utils.rs` - 97.04%

### 🟡 Good (70-89% lines)
- `commands/hooks_setup.rs` - 86.11%
- `commands/init.rs` - 86.21%
- `commands/meta.rs` - 80.75%
- `commands/wrapped.rs` - 72.50%
- `embedded.rs` - 79.22%
- `test_helpers/fixtures.rs` - 80.00%
- `test_helpers/workflow.rs` - 86.11%
- `tui/app.rs` - 87.66%
- `tui/tabs/files.rs` - 83.93%

### 🟠 Moderate (40-69% lines)
- `commands/archive.rs` - 59.30%
- `commands/reflect.rs` - 52.70%
- `commands/workflow/mod.rs` - 54.92%
- `main.rs` - 43.14%

### 🔴 Needs Work (<40% lines)
- `commands/astq.rs` - 38.57%
- `tui/mod.rs` - 11.36%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 91.66% | ✅ Met |
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
