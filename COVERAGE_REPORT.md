# Test Coverage Report

**Last Updated**: 2025-10-29 19:33
**Tool**: cargo-llvm-cov
**Overall Coverage**: **90.39%** lines | **89.41%** regions | **89.08%** functions

## Summary

```
TOTAL                                      14856              1573    89.41%         806                88    89.08%        8905               856    90.39%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `src/adapters/claude_code.rs` | 93.82% | 94.29% | 90.00% | 🟢 Excellent |
| `src/adapters/codex.rs` | 94.43% | 94.51% | 91.18% | 🟢 Excellent |
| `src/adapters/cursor.rs` | 93.84% | 93.41% | 94.74% | 🟢 Excellent |
| `src/adapters/mod.rs` | 98.91% | 99.35% | 100.00% | 🟢 Excellent |
| `src/commands/analyze/mod.rs` | 98.04% | 97.65% | 100.00% | 🟢 Excellent |
| `src/commands/analyze/sections.rs` | 90.00% | 86.27% | 88.24% | 🟢 Excellent |
| `src/commands/archive.rs` | 59.30% | 62.27% | 64.29% | 🟠 Moderate |
| `src/commands/astq.rs` | 17.78% | 9.78% | 66.67% | 🔴 Needs Work |
| `src/commands/config.rs` | 91.95% | 90.30% | 63.64% | 🟢 Excellent |
| `src/commands/fork/amp.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/fork/codex.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/fork/cody.rs` | 100.00% | 98.41% | 100.00% | 🟢 Excellent |
| `src/commands/fork/gemini.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/fork/generic.rs` | 100.00% | 97.14% | 100.00% | 🟢 Excellent |
| `src/commands/fork/mod.rs` | 60.70% | 62.39% | 80.95% | 🟠 Moderate |
| `src/commands/git.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/hook.rs` | 96.64% | 92.78% | 61.54% | 🟢 Excellent |
| `src/commands/hooks_setup.rs` | 86.11% | 84.94% | 100.00% | 🟡 Good |
| `src/commands/init.rs` | 93.44% | 93.56% | 100.00% | 🟢 Excellent |
| `src/commands/meta.rs` | 80.75% | 80.85% | 83.33% | 🟡 Good |
| `src/commands/reflect.rs` | 52.70% | 52.94% | 100.00% | 🟠 Moderate |
| `src/commands/status.rs` | 87.23% | 94.12% | 100.00% | 🟡 Good |
| `src/commands/workflow/claims.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/workflow/context.rs` | 100.00% | 93.06% | 100.00% | 🟢 Excellent |
| `src/commands/workflow/mod.rs` | 52.49% | 51.49% | 54.17% | 🟠 Moderate |
| `src/commands/workflow/transitions.rs` | 92.39% | 88.24% | 50.00% | 🟢 Excellent |
| `src/commands/wrapped.rs` | 72.50% | 74.10% | 80.00% | 🟡 Good |
| `src/config.rs` | 91.67% | 91.04% | 76.92% | 🟢 Excellent |
| `src/embedded.rs` | 76.42% | 61.54% | 50.00% | 🟡 Good |
| `src/engine/mod.rs` | 99.82% | 99.60% | 97.37% | 🟢 Excellent |
| `src/engine/template.rs` | 97.25% | 96.88% | 97.50% | 🟢 Excellent |
| `src/guardrails/parser.rs` | 97.78% | 97.65% | 83.33% | 🟢 Excellent |
| `src/guardrails/types.rs` | 94.38% | 93.60% | 100.00% | 🟢 Excellent |
| `src/main.rs` | 40.98% | 35.17% | 100.00% | 🟠 Moderate |
| `src/metamodes/mod.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/metrics/aggregation.rs` | 97.18% | 95.50% | 100.00% | 🟢 Excellent |
| `src/metrics/graph.rs` | 99.20% | 98.76% | 100.00% | 🟢 Excellent |
| `src/metrics/hooks.rs` | 94.22% | 93.69% | 89.47% | 🟢 Excellent |
| `src/metrics/mod.rs` | 98.63% | 97.60% | 100.00% | 🟢 Excellent |
| `src/metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/rules/evaluator.rs` | 98.44% | 97.74% | 96.15% | 🟢 Excellent |
| `src/rules/interrupt.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/rules/types.rs` | 95.68% | 92.07% | 100.00% | 🟢 Excellent |
| `src/storage/archive.rs` | 90.20% | 89.15% | 69.23% | 🟢 Excellent |
| `src/storage/mod.rs` | 97.16% | 96.60% | 81.03% | 🟢 Excellent |
| `src/test_helpers/fixtures.rs` | 80.00% | 77.78% | 33.33% | 🟡 Good |
| `src/test_helpers/jsonl.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/metrics.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/storage.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/tui.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/workflow.rs` | 86.11% | 81.44% | 87.50% | 🟡 Good |
| `src/theme.rs` | 92.86% | 93.75% | 90.91% | 🟢 Excellent |
| `src/tui/app.rs` | 88.94% | 89.05% | 91.67% | 🟡 Good |
| `src/tui/mod.rs` | 11.36% | 13.70% | 16.67% | 🔴 Needs Work |
| `src/tui/tabs/events.rs` | 90.24% | 87.10% | 100.00% | 🟢 Excellent |
| `src/tui/tabs/files.rs` | 83.93% | 80.37% | 100.00% | 🟡 Good |
| `src/tui/tabs/overview.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/tui/tabs/phases.rs` | 95.51% | 95.30% | 100.00% | 🟢 Excellent |
| `src/tui/ui.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/tui/utils.rs` | 97.04% | 93.61% | 100.00% | 🟢 Excellent |

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
- `src/adapters/claude_code.rs` - 93.82%
- `src/adapters/codex.rs` - 94.43%
- `src/adapters/cursor.rs` - 93.84%
- `src/adapters/mod.rs` - 98.91%
- `src/commands/analyze/mod.rs` - 98.04%
- `src/commands/analyze/sections.rs` - 90.00%
- `src/commands/config.rs` - 91.95%
- `src/commands/fork/amp.rs` - 100.00%
- `src/commands/fork/codex.rs` - 100.00%
- `src/commands/fork/cody.rs` - 100.00%
- `src/commands/fork/gemini.rs` - 100.00%
- `src/commands/fork/generic.rs` - 100.00%
- `src/commands/git.rs` - 100.00%
- `src/commands/hook.rs` - 96.64%
- `src/commands/init.rs` - 93.44%
- `src/commands/workflow/claims.rs` - 100.00%
- `src/commands/workflow/context.rs` - 100.00%
- `src/commands/workflow/transitions.rs` - 92.39%
- `src/config.rs` - 91.67%
- `src/engine/mod.rs` - 99.82%
- `src/engine/template.rs` - 97.25%
- `src/guardrails/parser.rs` - 97.78%
- `src/guardrails/types.rs` - 94.38%
- `src/metamodes/mod.rs` - 100.00%
- `src/metrics/aggregation.rs` - 97.18%
- `src/metrics/graph.rs` - 99.20%
- `src/metrics/hooks.rs` - 94.22%
- `src/metrics/mod.rs` - 98.63%
- `src/metrics/states.rs` - 100.00%
- `src/metrics/transcript.rs` - 100.00%
- `src/rules/evaluator.rs` - 98.44%
- `src/rules/interrupt.rs` - 100.00%
- `src/rules/types.rs` - 95.68%
- `src/storage/archive.rs` - 90.20%
- `src/storage/mod.rs` - 97.16%
- `src/test_helpers/jsonl.rs` - 100.00%
- `src/test_helpers/metrics.rs` - 100.00%
- `src/test_helpers/storage.rs` - 100.00%
- `src/test_helpers/tui.rs` - 100.00%
- `src/theme.rs` - 92.86%
- `src/tui/tabs/events.rs` - 90.24%
- `src/tui/tabs/overview.rs` - 100.00%
- `src/tui/tabs/phases.rs` - 95.51%
- `src/tui/ui.rs` - 100.00%
- `src/tui/utils.rs` - 97.04%

### 🟡 Good (70-89% lines)
- `src/commands/hooks_setup.rs` - 86.11%
- `src/commands/meta.rs` - 80.75%
- `src/commands/status.rs` - 87.23%
- `src/commands/wrapped.rs` - 72.50%
- `src/embedded.rs` - 76.42%
- `src/test_helpers/fixtures.rs` - 80.00%
- `src/test_helpers/workflow.rs` - 86.11%
- `src/tui/app.rs` - 88.94%
- `src/tui/tabs/files.rs` - 83.93%

### 🟠 Moderate (40-69% lines)
- `src/commands/archive.rs` - 59.30%
- `src/commands/fork/mod.rs` - 60.70%
- `src/commands/reflect.rs` - 52.70%
- `src/commands/workflow/mod.rs` - 52.49%
- `src/main.rs` - 40.98%

### 🔴 Needs Work (<40% lines)
- `src/commands/astq.rs` - 17.78%
- `src/tui/mod.rs` - 11.36%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 90.39% | ✅ Met |
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
