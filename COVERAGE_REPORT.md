# Test Coverage Report

**Last Updated**: 2026-05-26 15:55
**Tool**: cargo-llvm-cov
**Overall Coverage**: **74.54%** lines | **73.98%** regions | **76.20%** functions

## Summary

```
TOTAL                                         19509              5076    73.98%        1084               258    76.20%       12183              3102    74.54%           0                 0         -
```

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
| `src/adapters/claude_code.rs` | 86.38% | 86.22% | 83.33% | 🟡 Good |
| `src/adapters/codex.rs` | 94.43% | 94.51% | 91.18% | 🟢 Excellent |
| `src/adapters/cursor.rs` | 93.84% | 93.41% | 94.74% | 🟢 Excellent |
| `src/adapters/mod.rs` | 98.91% | 99.35% | 100.00% | 🟢 Excellent |
| `src/analyze/cleanup/aborted.rs` | 97.22% | 97.42% | 90.91% | 🟢 Excellent |
| `src/analyze/cleanup/duplicate_cowboy.rs` | 69.94% | 66.98% | 46.15% | 🟠 Moderate |
| `src/analyze/cleanup/git.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/analyze/cleanup/mod.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/analyze/gap_detection.rs` | 86.03% | 83.29% | 91.67% | 🟡 Good |
| `src/analyze/repair.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/analyze/sections.rs` | 30.21% | 27.98% | 61.11% | 🔴 Needs Work |
| `src/analyze/totals.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/commands/analyze/mod.rs` | 93.50% | 92.73% | 100.00% | 🟢 Excellent |
| `src/commands/archive.rs` | 43.35% | 47.54% | 50.00% | 🟠 Moderate |
| `src/commands/astq.rs` | 17.78% | 9.78% | 66.67% | 🔴 Needs Work |
| `src/commands/config.rs` | 91.95% | 90.30% | 63.64% | 🟢 Excellent |
| `src/commands/doctor/fix_ddd.rs` | 4.47% | 3.98% | 7.69% | 🔴 Needs Work |
| `src/commands/doctor/fix_state.rs` | 32.20% | 25.00% | 100.00% | 🔴 Needs Work |
| `src/commands/doctor/mod.rs` | 90.00% | 81.82% | 100.00% | 🟢 Excellent |
| `src/commands/doctor/tests.rs` | 99.44% | 99.25% | 100.00% | 🟢 Excellent |
| `src/commands/external_bin.rs` | 33.98% | 27.85% | 37.50% | 🔴 Needs Work |
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
| `src/commands/ide.rs` | 62.50% | 58.33% | 50.00% | 🟠 Moderate |
| `src/commands/init.rs` | 93.44% | 93.56% | 100.00% | 🟢 Excellent |
| `src/commands/markdown.rs` | 30.06% | 28.71% | 42.22% | 🔴 Needs Work |
| `src/commands/meta.rs` | 64.88% | 70.59% | 83.33% | 🟠 Moderate |
| `src/commands/new.rs` | 73.81% | 80.65% | 68.42% | 🟡 Good |
| `src/commands/pm.rs` | 60.00% | 68.42% | 50.00% | 🟠 Moderate |
| `src/commands/reflect.rs` | 19.55% | 14.17% | 25.00% | 🔴 Needs Work |
| `src/commands/review.rs` | 89.25% | 90.73% | 81.25% | 🟡 Good |
| `src/commands/status.rs` | 44.83% | 37.42% | 50.00% | 🟠 Moderate |
| `src/commands/workflow/claims.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/commands/workflow/context.rs` | 97.92% | 93.15% | 75.00% | 🟢 Excellent |
| `src/commands/workflow/mod.rs` | 60.51% | 57.25% | 47.22% | 🟠 Moderate |
| `src/commands/workflow/transitions.rs` | 71.32% | 67.56% | 34.48% | 🟡 Good |
| `src/commands/wrapped.rs` | 71.79% | 72.44% | 80.00% | 🟡 Good |
| `src/config.rs` | 76.47% | 75.54% | 57.89% | 🟡 Good |
| `src/ddd.rs` | 77.04% | 80.23% | 87.30% | 🟡 Good |
| `src/doctor/migrations.rs` | 82.98% | 83.93% | 75.00% | 🟡 Good |
| `src/doctor/rescue.rs` | 0.00% | 0.00% | 0.00% | 🔴 Needs Work |
| `src/embedded.rs` | 82.79% | 75.56% | 50.00% | 🟡 Good |
| `src/engine/handlebars.rs` | 94.25% | 89.16% | 100.00% | 🟢 Excellent |
| `src/engine/mod.rs` | 86.98% | 88.16% | 86.67% | 🟡 Good |
| `src/engine/template.rs` | 98.81% | 98.88% | 100.00% | 🟢 Excellent |
| `src/guardrails/parser.rs` | 97.78% | 97.65% | 83.33% | 🟢 Excellent |
| `src/guardrails/types.rs` | 94.38% | 93.60% | 100.00% | 🟢 Excellent |
| `src/main.rs` | 42.31% | 35.77% | 100.00% | 🟠 Moderate |
| `src/metamodes/mod.rs` | 99.29% | 99.56% | 100.00% | 🟢 Excellent |
| `src/metrics/aggregation.rs` | 77.88% | 78.69% | 92.86% | 🟡 Good |
| `src/metrics/cowboy.rs` | 100.00% | 99.33% | 100.00% | 🟢 Excellent |
| `src/metrics/git.rs` | 91.89% | 87.59% | 100.00% | 🟢 Excellent |
| `src/metrics/graph.rs` | 90.06% | 89.20% | 100.00% | 🟢 Excellent |
| `src/metrics/hooks.rs` | 94.22% | 93.69% | 89.47% | 🟢 Excellent |
| `src/metrics/mod.rs` | 54.55% | 55.89% | 12.50% | 🟠 Moderate |
| `src/metrics/states.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/metrics/transcript.rs` | 100.00% | 99.16% | 100.00% | 🟢 Excellent |
| `src/rules/evaluator.rs` | 93.63% | 90.70% | 88.24% | 🟢 Excellent |
| `src/rules/interrupt.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/rules/types.rs` | 96.14% | 93.05% | 100.00% | 🟢 Excellent |
| `src/storage/archive/aggregation.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/storage/archive/builder.rs` | 98.25% | 96.38% | 77.78% | 🟢 Excellent |
| `src/storage/archive/mod.rs` | 90.34% | 91.46% | 75.00% | 🟢 Excellent |
| `src/storage/archive/validation.rs` | 94.44% | 94.12% | 100.00% | 🟢 Excellent |
| `src/storage/log_cleanup.rs` | 75.00% | 73.08% | 33.33% | 🟡 Good |
| `src/storage/mod.rs` | 80.88% | 76.21% | 58.46% | 🟡 Good |
| `src/storage/reviews.rs` | 96.08% | 96.27% | 92.31% | 🟢 Excellent |
| `src/test_helpers/archive.rs` | 94.12% | 92.31% | 85.71% | 🟢 Excellent |
| `src/test_helpers/fixtures.rs` | 80.00% | 77.78% | 33.33% | 🟡 Good |
| `src/test_helpers/jsonl.rs` | 97.22% | 98.55% | 100.00% | 🟢 Excellent |
| `src/test_helpers/metrics.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/storage.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/tui.rs` | 100.00% | 100.00% | 100.00% | 🟢 Excellent |
| `src/test_helpers/workflow.rs` | 87.29% | 81.77% | 88.89% | 🟡 Good |
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
- `src/adapters/codex.rs` - 94.43%
- `src/adapters/cursor.rs` - 93.84%
- `src/adapters/mod.rs` - 98.91%
- `src/analyze/cleanup/aborted.rs` - 97.22%
- `src/commands/analyze/mod.rs` - 93.50%
- `src/commands/config.rs` - 91.95%
- `src/commands/doctor/mod.rs` - 90.00%
- `src/commands/doctor/tests.rs` - 99.44%
- `src/commands/fork/amp.rs` - 100.00%
- `src/commands/fork/codex.rs` - 100.00%
- `src/commands/fork/cody.rs` - 100.00%
- `src/commands/fork/gemini.rs` - 100.00%
- `src/commands/fork/generic.rs` - 100.00%
- `src/commands/git.rs` - 100.00%
- `src/commands/hook.rs` - 96.64%
- `src/commands/init.rs` - 93.44%
- `src/commands/workflow/claims.rs` - 100.00%
- `src/commands/workflow/context.rs` - 97.92%
- `src/engine/handlebars.rs` - 94.25%
- `src/engine/template.rs` - 98.81%
- `src/guardrails/parser.rs` - 97.78%
- `src/guardrails/types.rs` - 94.38%
- `src/metamodes/mod.rs` - 99.29%
- `src/metrics/cowboy.rs` - 100.00%
- `src/metrics/git.rs` - 91.89%
- `src/metrics/graph.rs` - 90.06%
- `src/metrics/hooks.rs` - 94.22%
- `src/metrics/states.rs` - 100.00%
- `src/metrics/transcript.rs` - 100.00%
- `src/rules/evaluator.rs` - 93.63%
- `src/rules/interrupt.rs` - 100.00%
- `src/rules/types.rs` - 96.14%
- `src/storage/archive/aggregation.rs` - 100.00%
- `src/storage/archive/builder.rs` - 98.25%
- `src/storage/archive/mod.rs` - 90.34%
- `src/storage/archive/validation.rs` - 94.44%
- `src/storage/reviews.rs` - 96.08%
- `src/test_helpers/archive.rs` - 94.12%
- `src/test_helpers/jsonl.rs` - 97.22%
- `src/test_helpers/metrics.rs` - 100.00%
- `src/test_helpers/storage.rs` - 100.00%
- `src/test_helpers/tui.rs` - 100.00%
- `src/tui/tabs/events.rs` - 90.24%
- `src/tui/tabs/overview.rs` - 100.00%
- `src/tui/tabs/phases.rs` - 95.51%
- `src/tui/ui.rs` - 100.00%
- `src/tui/utils.rs` - 97.04%

### 🟡 Good (70-89% lines)
- `src/adapters/claude_code.rs` - 86.38%
- `src/analyze/gap_detection.rs` - 86.03%
- `src/commands/fork/mod.rs` - 73.68%
- `src/commands/hooks_setup.rs` - 86.11%
- `src/commands/new.rs` - 73.81%
- `src/commands/review.rs` - 89.25%
- `src/commands/workflow/transitions.rs` - 71.32%
- `src/commands/wrapped.rs` - 71.79%
- `src/config.rs` - 76.47%
- `src/ddd.rs` - 77.04%
- `src/doctor/migrations.rs` - 82.98%
- `src/embedded.rs` - 82.79%
- `src/engine/mod.rs` - 86.98%
- `src/metrics/aggregation.rs` - 77.88%
- `src/storage/log_cleanup.rs` - 75.00%
- `src/storage/mod.rs` - 80.88%
- `src/test_helpers/fixtures.rs` - 80.00%
- `src/test_helpers/workflow.rs` - 87.29%
- `src/theme.rs` - 85.71%
- `src/tui/app.rs` - 87.66%
- `src/tui/tabs/files.rs` - 83.93%

### 🟠 Moderate (40-69% lines)
- `src/analyze/cleanup/duplicate_cowboy.rs` - 69.94%
- `src/commands/archive.rs` - 43.35%
- `src/commands/fork/runtime.rs` - 44.20%
- `src/commands/ide.rs` - 62.50%
- `src/commands/meta.rs` - 64.88%
- `src/commands/pm.rs` - 60.00%
- `src/commands/status.rs` - 44.83%
- `src/commands/workflow/mod.rs` - 60.51%
- `src/main.rs` - 42.31%
- `src/metrics/mod.rs` - 54.55%

### 🔴 Needs Work (<40% lines)
- `src/analyze/cleanup/git.rs` - 0.00%
- `src/analyze/cleanup/mod.rs` - 0.00%
- `src/analyze/repair.rs` - 0.00%
- `src/analyze/sections.rs` - 30.21%
- `src/analyze/totals.rs` - 0.00%
- `src/commands/astq.rs` - 17.78%
- `src/commands/doctor/fix_ddd.rs` - 4.47%
- `src/commands/doctor/fix_state.rs` - 32.20%
- `src/commands/external_bin.rs` - 33.98%
- `src/commands/markdown.rs` - 30.06%
- `src/commands/reflect.rs` - 19.55%
- `src/doctor/rescue.rs` - 0.00%
- `src/tui/mod.rs` - 11.36%

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
| Overall | ≥80% | 74.54% | ⏳ In Progress |
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
