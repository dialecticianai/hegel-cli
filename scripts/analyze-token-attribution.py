#!/usr/bin/env python3
"""
Aggregate analysis of token attribution from hegel analyze --debug --json

Usage:
    hegel analyze --debug START..END --json | ./scripts/analyze-token-attribution.py

Identifies patterns in zero-token phases to distinguish bugs from legitimate cases.
"""

import json
import sys
from collections import defaultdict
from datetime import datetime


def parse_timestamp(ts):
    """Parse RFC3339 timestamp, handle both Z and +00:00 formats"""
    if ts.endswith('Z'):
        ts = ts[:-1] + '+00:00'
    return datetime.fromisoformat(ts)


def main():
    # Read JSON from stdin
    data = json.load(sys.stdin)

    # Aggregate statistics
    total_phases = len(data)
    zero_token_phases = [p for p in data if p['tokens_attributed'] == 0]
    nonzero_token_phases = [p for p in data if p['tokens_attributed'] > 0]

    # Group by various dimensions
    by_workflow = defaultdict(list)
    by_phase_name = defaultdict(list)
    zero_by_archived = {'archived': [], 'live': []}

    for phase in data:
        by_workflow[phase['workflow_id']].append(phase)
        by_phase_name[phase['phase_name']].append(phase)

        if phase['tokens_attributed'] == 0:
            if phase['is_archived']:
                zero_by_archived['archived'].append(phase)
            else:
                zero_by_archived['live'].append(phase)

    # Calculate duration thresholds to identify suspicious phases
    # (Long duration + 0 tokens = probably a bug)
    suspicious_phases = [
        p for p in zero_token_phases
        if p['duration_seconds'] > 300  # >5 minutes with 0 tokens
    ]

    # Print analysis
    print("=" * 80)
    print("TOKEN ATTRIBUTION ANALYSIS")
    print("=" * 80)
    print()

    print(f"Total phases analyzed: {total_phases}")
    print(f"Zero-token phases:     {len(zero_token_phases)} ({len(zero_token_phases)/total_phases*100:.1f}%)")
    print(f"Non-zero token phases: {len(nonzero_token_phases)} ({len(nonzero_token_phases)/total_phases*100:.1f}%)")
    print()

    print("Zero-token breakdown:")
    print(f"  Archived phases: {len(zero_by_archived['archived'])}")
    print(f"  Live phases:     {len(zero_by_archived['live'])}")
    print()

    print(f"Suspicious phases (>5min duration, 0 tokens): {len(suspicious_phases)}")
    if suspicious_phases:
        print()
        print("Top 10 suspicious phases by duration:")
        suspicious_sorted = sorted(suspicious_phases, key=lambda p: p['duration_seconds'], reverse=True)
        for phase in suspicious_sorted[:10]:
            duration_mins = phase['duration_seconds'] / 60
            print(f"  {phase['workflow_id']} | {phase['phase_name']:15s} | {duration_mins:7.1f} min | 0 tokens")
    print()

    # Phase name analysis
    print("Zero-token rate by phase name:")
    phase_stats = []
    for phase_name, phases in by_phase_name.items():
        total = len(phases)
        zero_count = sum(1 for p in phases if p['tokens_attributed'] == 0)
        zero_rate = zero_count / total * 100 if total > 0 else 0
        phase_stats.append((phase_name, total, zero_count, zero_rate))

    phase_stats.sort(key=lambda x: x[3], reverse=True)  # Sort by zero rate
    for phase_name, total, zero_count, zero_rate in phase_stats:
        print(f"  {phase_name:15s}: {zero_count:3d}/{total:3d} ({zero_rate:5.1f}%)")
    print()

    # Workflow-level analysis
    workflows_all_zero = []
    workflows_mixed = []
    workflows_all_nonzero = []

    for wid, phases in by_workflow.items():
        zero_count = sum(1 for p in phases if p['tokens_attributed'] == 0)
        if zero_count == len(phases):
            workflows_all_zero.append(wid)
        elif zero_count == 0:
            workflows_all_nonzero.append(wid)
        else:
            workflows_mixed.append((wid, zero_count, len(phases)))

    print("Workflow-level patterns:")
    print(f"  All phases have 0 tokens:    {len(workflows_all_zero)} workflows")
    print(f"  All phases have >0 tokens:   {len(workflows_all_nonzero)} workflows")
    print(f"  Mixed (some 0, some >0):     {len(workflows_mixed)} workflows")
    print()

    # Live phase transcript matching (if any live phases present)
    live_phases = [p for p in data if not p['is_archived']]
    if live_phases:
        print("Live phase transcript matching:")
        for phase in live_phases:
            if 'transcript_events_examined' in phase:
                examined = phase.get('transcript_events_examined', 0)
                matched = phase.get('transcript_events_matched', 0)
                tokens = phase['tokens_attributed']
                print(f"  {phase['phase_name']:15s}: examined {examined:3d}, matched {matched:3d}, tokens {tokens:5d}")
        print()

    # Summary recommendations
    print("=" * 80)
    print("RECOMMENDATIONS")
    print("=" * 80)

    if len(suspicious_phases) > 10:
        print("⚠️  Many long-duration phases with 0 tokens detected.")
        print("    This suggests a systematic token attribution bug.")
        print("    Next steps:")
        print("    - Pick a suspicious workflow and examine its archive JSON directly")
        print("    - Check if transcript.jsonl has events in that timeframe")
        print("    - Use --verbose mode to see detailed event matching")
        print()

    if len(workflows_mixed) > 0:
        print("✓  Some workflows have mixed attribution (good sign - shows code path works)")
        print("    Focus debugging on workflows with all-zero phases.")
        print()

    if len(zero_by_archived['live']) > 0:
        print("⚠️  Live phases with 0 tokens detected - use --verbose to debug")


if __name__ == '__main__':
    main()
