#!/usr/bin/env bash
# Analyze Claude Code hook event schema from .hegel/hooks.jsonl
# Usage: ./scripts/analyze-hook-schema.sh

set -euo pipefail

HOOKS_FILE="${1:-.hegel/hooks.jsonl}"

if [[ ! -f "$HOOKS_FILE" ]]; then
    echo "Error: $HOOKS_FILE not found" >&2
    exit 1
fi

echo "=== Hook Event Schema Analysis ==="
echo "File: $HOOKS_FILE"
echo "Total events: $(wc -l < "$HOOKS_FILE")"
echo ""

echo "--- Top-level keys across all events ---"
jq -r 'keys[]' "$HOOKS_FILE" | sort | uniq -c | sort -rn
echo ""

echo "--- Event types (hook_event_name) ---"
jq -r '.hook_event_name' "$HOOKS_FILE" | sort | uniq -c | sort -rn
echo ""

echo "--- Sample PostToolUse event (full) ---"
jq 'select(.hook_event_name == "PostToolUse") | .' "$HOOKS_FILE" | head -n 50
echo ""

echo "--- Sample PreToolUse event (full) ---"
jq 'select(.hook_event_name == "PreToolUse") | .' "$HOOKS_FILE" | head -n 50
echo ""

echo "--- Checking for token usage fields ---"
echo "Events with 'token' in keys:"
jq 'select(keys[] | test("token"; "i")) | {hook_event_name, keys: keys}' "$HOOKS_FILE" | head -n 20
echo ""

echo "--- Checking for usage/metrics fields ---"
echo "Events with 'usage' or 'metric' in keys:"
jq 'select(keys[] | test("usage|metric"; "i")) | {hook_event_name, keys: keys}' "$HOOKS_FILE" | head -n 20
echo ""

echo "--- Tool names used (from PostToolUse) ---"
jq -r 'select(.hook_event_name == "PostToolUse") | .tool_call.name' "$HOOKS_FILE" 2>/dev/null | sort | uniq -c | sort -rn || echo "No tool_call.name found"
echo ""

echo "--- Session IDs ---"
jq -r '.session_id' "$HOOKS_FILE" | sort -u
