#!/usr/bin/env bash
# Check for specific fields in Claude Code hook events
# Usage: ./scripts/check-hook-fields.sh

set -euo pipefail

HOOKS_FILE="${1:-.hegel/hooks.jsonl}"

echo "=== Field Availability Check ==="
echo ""

echo "1. Token usage fields:"
if jq -e 'select(keys[] | test("token"; "i"))' "$HOOKS_FILE" > /dev/null 2>&1; then
    echo "   ✓ Found token-related fields"
    jq -r '[keys[] | select(test("token"; "i"))] | unique | .[]' "$HOOKS_FILE" | head -5
else
    echo "   ✗ No token usage data found"
fi
echo ""

echo "2. Timestamp fields:"
if jq -e 'select(keys[] | test("time|date"; "i"))' "$HOOKS_FILE" > /dev/null 2>&1; then
    echo "   ✓ Found timestamp fields"
    jq -r '[keys[] | select(test("time|date"; "i"))] | unique | .[]' "$HOOKS_FILE" | head -5
else
    echo "   ✗ No timestamp fields found"
fi
echo ""

echo "3. Session/Workflow ID:"
if jq -e '.session_id' "$HOOKS_FILE" > /dev/null 2>&1; then
    echo "   ✓ session_id available"
    echo "   Unique sessions: $(jq -r '.session_id' "$HOOKS_FILE" | sort -u | wc -l)"
else
    echo "   ✗ No session_id found"
fi
echo ""

echo "4. Tool information:"
if jq -e '.tool_name' "$HOOKS_FILE" > /dev/null 2>&1; then
    echo "   ✓ tool_name available"
    echo "   Tools used:"
    jq -r '.tool_name // empty' "$HOOKS_FILE" | sort | uniq -c | sort -rn
else
    echo "   ✗ No tool_name found"
fi
echo ""

echo "5. Sample event structure (first PostToolUse):"
jq 'select(.hook_event_name == "PostToolUse") | keys' "$HOOKS_FILE" | head -1
echo ""

echo "6. Transcript path (for external token lookup):"
if jq -e '.transcript_path' "$HOOKS_FILE" > /dev/null 2>&1; then
    echo "   ✓ transcript_path available"
    jq -r '.transcript_path // empty' "$HOOKS_FILE" | sort -u | head -3
else
    echo "   ✗ No transcript_path found"
fi
