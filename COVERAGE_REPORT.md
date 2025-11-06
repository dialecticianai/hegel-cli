# Test Coverage Report

**Last Updated**: 2025-11-06 17:24
**Tool**: cargo-llvm-cov
**Overall Coverage**: **84.04%** lines | **83.65%** regions | **84.52%** functions

## Summary

```
TOTAL                                         19214              3142    83.65%        1014               157    84.52%       11754              1876    84.04%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `src/adapters/claude_code.rs` | 93.82% | 94.29% | 90.00% | 🟢 Excellent |
| `src/adapters/codex.rs` | 94.43% | 94.51% | 91.18% | 🟢 Excellent |
| `src/adapters/cursor.rs` | 93.84% | 93.41% | 94.74% | 🟢 Excellent |
| `src/adapters/mod.rs` | 98.91% | 99.35% | 100.00% | 🟢 Excellent |
| `src/analyze/cleanup/aborted.rs` | 97.30% | 97.42% | 90.91% | 🟢 Excellent |
| `src/analyze/cleanup/duplicate_cowboy.rs` | 69.94% | 66.98% | 46.15% | 🟠 Moderate |
| `src/analyze/cleanup/git.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/analyze/cleanup/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/analyze/gap_detection.rs` | 84.29% | 82.37% | 90.91% | 🟡 Good |
| `src/analyze/repair.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/analyze/sections.rs` | 30.21% | 27.98% | 61.11% | 🔴 Needs Work |
| `src/analyze/totals.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/commands/analyze/mod.rs` | 93.50% | 92.73% | 100.00% | 🟢 Excellent |
| `src/commands/archive.rs` | 43.35% | 47.54% | 50.00% | 🟠 Moderate |
| `src/commands/astq.rs` | 17.78% | 9.78% | 66.67% | 🔴 Needs Work |
| `src/commands/config.rs` | 91.95% | 90.30% | 63.64% | 🟢 Excellent |
| `src/commands/external_bin.rs` | 53.19% | 48.68% | 50.00% | 🟠 Moderate |
| `src/commands/fork/amp.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/fork/codex.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/fork/cody.rs` | 100.00% | 98.41% | 100.00% | 🟢 Excellent |
| `src/commands/fork/gemini.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/fork/generic.rs` | 100.00% | 97.14% | 100.00% | 🟢 Excellent |
| `src/commands/fork/mod.rs` | 73.68% | 73.06% | 100.00% | 🟡 Good |
| `src/commands/fork/runtime.rs` | 44.20% | 52.80% | 63.64% | 🟠 Moderate |
| `src/commands/git.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/hook.rs` | 96.64% | 92.78% | 61.54% | 🟢 Excellent |
| `src/commands/hooks_setup.rs` | 86.11% | 84.94% | 100.00% | 🟡 Good |
| `src/commands/init.rs` | 93.44% | 93.56% | 100.00% | 🟢 Excellent |
| `src/commands/meta.rs` | 64.88% | 70.71% | 83.33% | 🟠 Moderate |
| `src/commands/pm.rs` | 60.00% | 68.42% | 50.00% | 🟠 Moderate |
| `src/commands/reflect.rs` | 60.53% | 47.62% | 75.00% | 🟠 Moderate |
| `src/commands/status.rs` | 43.21% | 37.42% | 50.00% | 🟠 Moderate |
| `src/commands/workflow/claims.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/workflow/context.rs` | 100.00% | 93.24% | 100.00% | 🟢 Excellent |
| `src/commands/workflow/mod.rs` | 59.54% | 56.46% | 50.00% | 🟠 Moderate |
| `src/commands/workflow/transitions.rs` | 86.79% | 82.50% | 62.96% | 🟡 Good |
| `src/commands/wrapped.rs` | 71.79% | 72.44% | 80.00% | 🟡 Good |
| `src/config.rs` | 91.67% | 91.04% | 76.92% | 🟢 Excellent |
| `src/embedded.rs` | 82.79% | 75.56% | 50.00% | 🟡 Good |
| `src/engine/handlebars.rs` | 98.31% | 97.26% | 100.00% | 🟢 Excellent |
| `src/engine/mod.rs` | 99.72% | 99.52% | 97.92% | 🟢 Excellent |
| `src/engine/template.rs` | 97.25% | 96.88% | 97.50% | 🟢 Excellent |
| `src/guardrails/parser.rs` | 97.78% | 97.65% | 83.33% | 🟢 Excellent |
| `src/guardrails/types.rs` | 94.38% | 93.60% | 100.00% | 🟢 Excellent |
| `src/main.rs` | 55.56% | 41.57% | 100.00% | 🟠 Moderate |
| `src/metamodes/mod.rs` | 99.29% | 99.56% | 100.00% | 🟢 Excellent |
| `src/metrics/aggregation.rs` | 75.80% | 79.74% | 93.75% | 🟡 Good |
| `src/metrics/cowboy.rs` | 100.00% | 99.42% | 100.00% | 🟢 Excellent |
| `src/metrics/git.rs` | 92.45% | 91.68% | 95.45% | 🟢 Excellent |
| `src/metrics/graph.rs` | 90.06% | 89.20% | 100.00% | 🟢 Excellent |
| `src/metrics/hooks.rs` | 94.22% | 93.69% | 89.47% | 🟢 Excellent |
| `src/metrics/mod.rs` | 84.04% | 83.66% | 69.57% | 🟡 Good |
| `src/metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/metrics/transcript.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/rules/evaluator.rs` | 98.43% | 97.74% | 96.15% | 🟢 Excellent |
| `src/rules/interrupt.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/rules/types.rs` | 96.22% | 92.51% | 100.00% | 🟢 Excellent |
| `src/storage/archive/aggregation.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/storage/archive/builder.rs` | 98.25% | 96.38% | 77.78% | 🟢 Excellent |
| `src/storage/archive/mod.rs` | 93.44% | 93.85% | 76.47% | 🟢 Excellent |
| `src/storage/archive/validation.rs` | 94.44% | 94.12% | 100.00% | 🟢 Excellent |
| `src/storage/log_cleanup.rs` | 87.50% | 84.62% | 33.33% | 🟡 Good |
| `src/storage/mod.rs` | 89.96% | 91.03% | 70.00% | 🟡 Good |
| `src/test_helpers/archive.rs` | 94.12% | 92.31% | 85.71% | 🟢 Excellent |
| `src/test_helpers/fixtures.rs` | 80.00% | 77.78% | 33.33% | 🟡 Good |
| `src/test_helpers/jsonl.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/metrics.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/storage.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/tui.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/workflow.rs` | 86.24% | 81.55% | 87.50% | 🟡 Good |
| `src/theme.rs` | 85.71% | 85.94% | 81.82% | 🟡 Good |
| `src/tui/app.rs` | 87.66% | 87.68% | 87.50% | 🟡 Good |
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
- `src/analyze/cleanup/aborted.rs` - 97.30%
- `src/commands/analyze/mod.rs` - 93.50%
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
- `src/config.rs` - 91.67%
- `src/engine/handlebars.rs` - 98.31%
- `src/engine/mod.rs` - 99.72%
- `src/engine/template.rs` - 97.25%
- `src/guardrails/parser.rs` - 97.78%
- `src/guardrails/types.rs` - 94.38%
- `src/metamodes/mod.rs` - 99.29%
- `src/metrics/cowboy.rs` - 100.00%
- `src/metrics/git.rs` - 92.45%
- `src/metrics/graph.rs` - 90.06%
- `src/metrics/hooks.rs` - 94.22%
- `src/metrics/states.rs` - 100.00%
- `src/metrics/transcript.rs` - 100.00%
- `src/rules/evaluator.rs` - 98.43%
- `src/rules/interrupt.rs` - 100.00%
- `src/rules/types.rs` - 96.22%
- `src/storage/archive/aggregation.rs` - 100.00%
- `src/storage/archive/builder.rs` - 98.25%
- `src/storage/archive/mod.rs` - 93.44%
- `src/storage/archive/validation.rs` - 94.44%
- `src/test_helpers/archive.rs` - 94.12%
- `src/test_helpers/jsonl.rs` - 100.00%
- `src/test_helpers/metrics.rs` - 100.00%
- `src/test_helpers/storage.rs` - 100.00%
- `src/test_helpers/tui.rs` - 100.00%
- `src/tui/tabs/events.rs` - 90.24%
- `src/tui/tabs/overview.rs` - 100.00%
- `src/tui/tabs/phases.rs` - 95.51%
- `src/tui/ui.rs` - 100.00%
- `src/tui/utils.rs` - 97.04%

### 🟡 Good (70-89% lines)
- `src/analyze/gap_detection.rs` - 84.29%
- `src/commands/fork/mod.rs` - 73.68%
- `src/commands/hooks_setup.rs` - 86.11%
- `src/commands/workflow/transitions.rs` - 86.79%
- `src/commands/wrapped.rs` - 71.79%
- `src/embedded.rs` - 82.79%
- `src/metrics/aggregation.rs` - 75.80%
- `src/metrics/mod.rs` - 84.04%
- `src/storage/log_cleanup.rs` - 87.50%
- `src/storage/mod.rs` - 89.96%
- `src/test_helpers/fixtures.rs` - 80.00%
- `src/test_helpers/workflow.rs` - 86.24%
- `src/theme.rs` - 85.71%
- `src/tui/app.rs` - 87.66%
- `src/tui/tabs/files.rs` - 83.93%

### 🟠 Moderate (40-69% lines)
- `src/analyze/cleanup/duplicate_cowboy.rs` - 69.94%
- `src/commands/archive.rs` - 43.35%
- `src/commands/external_bin.rs` - 53.19%
- `src/commands/fork/runtime.rs` - 44.20%
- `src/commands/meta.rs` - 64.88%
- `src/commands/pm.rs` - 60.00%
- `src/commands/reflect.rs` - 60.53%
- `src/commands/status.rs` - 43.21%
- `src/commands/workflow/mod.rs` - 59.54%
- `src/main.rs` - 55.56%

### 🔴 Needs Work (<40% lines)
- `src/analyze/cleanup/git.rs` - 0.00%
- `src/analyze/cleanup/mod.rs` - 0.00%
- `src/analyze/repair.rs` - 0.00%
- `src/analyze/sections.rs` - 30.21%
- `src/analyze/totals.rs` - 0.00%
- `src/commands/astq.rs` - 17.78%
- `src/tui/mod.rs` - 11.36%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 84.04% | ✅ Met |
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
