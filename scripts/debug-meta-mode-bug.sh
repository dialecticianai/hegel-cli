#!/bin/bash
# Reproduce the meta-mode bug scenario

set -e

TESTDIR="/tmp/debug-meta-mode-$(date +%s)"
echo "Creating test directory: $TESTDIR"
mkdir -p "$TESTDIR"
cd "$TESTDIR"

echo ""
echo "=== Step 1: Initialize project ==="
echo "Running: hegel init greenfield"
hegel init greenfield > /dev/null 2>&1

echo "Meta-mode after init:"
jq -r '.workflow_state.meta_mode.name // "null"' .hegel/state.json

echo ""
echo "=== Step 2: Complete init-greenfield workflow ==="
echo "Advancing through init phases..."
hegel next > /dev/null 2>&1
hegel next > /dev/null 2>&1
hegel next > /dev/null 2>&1
hegel next > /dev/null 2>&1
hegel next > /dev/null 2>&1  # Advance to done

echo "Status after completing init-greenfield:"
hegel status | grep -A 3 "Mode:"
echo "Meta-mode:"
jq -r '.workflow_state.meta_mode.name // "null"' .hegel/state.json

echo ""
echo "=== Step 3: Declare learning meta-mode ==="
echo "Running: hegel meta learning"
hegel meta learning > /dev/null 2>&1

echo "Meta-mode after 'hegel meta learning':"
jq -r '.workflow_state.meta_mode.name // "null"' .hegel/state.json
echo "Current workflow:"
jq -r '.workflow_state.mode // "null"' .hegel/state.json

echo ""
echo "=== Step 4: Complete research workflow ==="
echo "Advancing through research phases..."
hegel next > /dev/null 2>&1
hegel next > /dev/null 2>&1
hegel next > /dev/null 2>&1
hegel next > /dev/null 2>&1

echo "Status after completing research:"
hegel status | grep -A 3 "Mode:"
echo "Meta-mode after completing research:"
jq -r '.workflow_state.meta_mode.name // "null"' .hegel/state.json

echo ""
echo "=== Step 5: Try to transition to discovery ==="
echo "Running: hegel next"
hegel next > /tmp/next-output.txt 2>&1 || true

echo "Output:"
head -5 /tmp/next-output.txt

echo ""
echo "Final meta-mode:"
jq -r '.workflow_state.meta_mode.name // "null"' .hegel/state.json

echo ""
echo "=== SUMMARY ==="
echo "Expected meta-mode: learning"
echo "Actual meta-mode: $(jq -r '.workflow_state.meta_mode.name // "null"' .hegel/state.json)"

if [ "$(jq -r '.workflow_state.meta_mode.name' .hegel/state.json)" == "learning" ]; then
    echo "✅ Meta-mode preserved correctly!"
else
    echo "❌ BUG: Meta-mode was changed to $(jq -r '.workflow_state.meta_mode.name' .hegel/state.json)"
fi

echo ""
echo "Test directory: $TESTDIR"
echo "Full state: cat $TESTDIR/.hegel/state.json | jq"
