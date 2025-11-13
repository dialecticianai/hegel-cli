#!/usr/bin/env bash
set -euo pipefail

# Debug phase count anomalies in .hegel directories
#
# Usage:
#   ./scripts/debug-phase-count.sh <path-to-hegel-dir>
#
# Example:
#   ./scripts/debug-phase-count.sh ~/Code/aecrim/.hegel

if [ $# -ne 1 ]; then
    echo "Usage: $0 <path-to-hegel-dir>"
    exit 1
fi

HEGEL_DIR="$1"

if [ ! -d "$HEGEL_DIR" ]; then
    echo "Error: Directory not found: $HEGEL_DIR"
    exit 1
fi

echo "=== Phase Count Debugging ==="
echo "Directory: $HEGEL_DIR"
echo ""

# Check current state
echo "--- Current State (state.json) ---"
if [ -f "$HEGEL_DIR/state.json" ]; then
    CURRENT_HISTORY=$(jq -r '.workflow_state.history | length' "$HEGEL_DIR/state.json" 2>/dev/null || echo "0")
    echo "Current history phases: $CURRENT_HISTORY"
    jq -r '.workflow_state.history[]?' "$HEGEL_DIR/state.json" 2>/dev/null | nl -v 1
else
    echo "No state.json found"
    CURRENT_HISTORY=0
fi
echo ""

# Check archives
echo "--- Archived Workflows ---"
ARCHIVE_DIR="$HEGEL_DIR/archive"
if [ -d "$ARCHIVE_DIR" ]; then
    ARCHIVE_COUNT=$(find "$ARCHIVE_DIR" -name "*.json" | wc -l | tr -d ' ')
    echo "Archive count: $ARCHIVE_COUNT workflows"

    TOTAL_ARCHIVED_PHASES=0
    find "$ARCHIVE_DIR" -name "*.json" | sort | while read -r archive; do
        # Try new format (.phases[])
        PHASES=$(jq -r '.phases | length' "$archive" 2>/dev/null || echo "0")
        WORKFLOW_ID=$(jq -r '.workflow_id' "$archive" 2>/dev/null || echo "unknown")

        # Fallback to old format (.workflow_state.history[])
        if [ "$PHASES" = "0" ] || [ "$PHASES" = "null" ]; then
            PHASES=$(jq -r '.workflow_state.history | length' "$archive" 2>/dev/null || echo "0")
            WORKFLOW_ID=$(jq -r '.workflow_state.workflow_id' "$archive" 2>/dev/null || echo "unknown")
        fi

        echo "  $WORKFLOW_ID: $PHASES phases"
    done

    # Calculate total separately (try new format first)
    TOTAL_ARCHIVED_PHASES=$(find "$ARCHIVE_DIR" -name "*.json" | while read -r archive; do
        PHASES=$(jq -r '.phases | length' "$archive" 2>/dev/null || echo "0")
        if [ "$PHASES" = "null" ]; then
            jq -r '.workflow_state.history | length' "$archive" 2>/dev/null || echo "0"
        else
            echo "$PHASES"
        fi
    done | paste -sd+ - | bc)

    echo "Total archived phases: $TOTAL_ARCHIVED_PHASES"
else
    echo "No archive directory found"
    TOTAL_ARCHIVED_PHASES=0
fi
echo ""

# Check states.jsonl (if exists)
echo "--- State Transitions (states.jsonl) ---"
if [ -f "$HEGEL_DIR/states.jsonl" ]; then
    STATES_COUNT=$(wc -l < "$HEGEL_DIR/states.jsonl" | tr -d ' ')
    echo "State transition events: $STATES_COUNT"
else
    echo "No states.jsonl found"
    STATES_COUNT=0
fi
echo ""

# Check hooks.jsonl
echo "--- Hook Events (hooks.jsonl) ---"
if [ -f "$HEGEL_DIR/hooks.jsonl" ]; then
    HOOKS_COUNT=$(wc -l < "$HEGEL_DIR/hooks.jsonl" | tr -d ' ')
    echo "Total hook events: $HOOKS_COUNT"

    WORKFLOW_EVENTS=$(grep -c '"workflow_event"' "$HEGEL_DIR/hooks.jsonl" || echo "0")
    echo "Workflow events in hooks: $WORKFLOW_EVENTS"
else
    echo "No hooks.jsonl found"
    HOOKS_COUNT=0
fi
echo ""

# Summary
echo "=== Summary ==="
EXPECTED_TOTAL=$((CURRENT_HISTORY + TOTAL_ARCHIVED_PHASES))
echo "Expected phase count: $EXPECTED_TOTAL"
echo "  Current workflow: $CURRENT_HISTORY"
echo "  Archived workflows: $TOTAL_ARCHIVED_PHASES"
echo ""

# Run hegel metrics if available
if command -v hegel &> /dev/null; then
    PROJECT_DIR=$(dirname "$HEGEL_DIR")
    echo "--- Actual Metrics (hegel pm discover show) ---"
    cd "$PROJECT_DIR" && hegel pm discover show "$(basename "$PROJECT_DIR")" 2>&1 | grep -E "Phase count|Total events|Total tokens" || echo "Could not fetch metrics"
fi
