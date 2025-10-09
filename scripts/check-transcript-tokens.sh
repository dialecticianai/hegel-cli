#!/usr/bin/env bash
# Check if Claude Code transcript files contain token usage data
# Usage: ./scripts/check-transcript-tokens.sh

set -euo pipefail

HOOKS_FILE="${1:-.hegel/hooks.jsonl}"

echo "=== Transcript Token Check ==="
echo ""

# Get first transcript path
TRANSCRIPT=$(jq -r 'select(.transcript_path) | .transcript_path' "$HOOKS_FILE" | sort -u | head -1)

if [[ -z "$TRANSCRIPT" ]]; then
    echo "No transcript_path found in hooks"
    exit 1
fi

echo "Transcript file: $TRANSCRIPT"

if [[ ! -f "$TRANSCRIPT" ]]; then
    echo "✗ Transcript file not found"
    exit 1
fi

echo "✓ Transcript file exists"
echo "Events in transcript: $(wc -l < "$TRANSCRIPT")"
echo ""

echo "1. Checking for token-related keys:"
TOKEN_COUNT=$(jq 'select(keys[] | test("token"; "i"))' "$TRANSCRIPT" 2>/dev/null | wc -l)
echo "   Events with 'token' in keys: $TOKEN_COUNT"
echo ""

echo "2. Checking for usage-related keys:"
USAGE_COUNT=$(jq 'select(keys[] | test("usage"; "i"))' "$TRANSCRIPT" 2>/dev/null | wc -l)
echo "   Events with 'usage' in keys: $USAGE_COUNT"
echo ""

echo "3. Sample event types:"
jq -r '.type // "no-type"' "$TRANSCRIPT" | sort | uniq -c | head -10
echo ""

if [[ $USAGE_COUNT -gt 0 ]]; then
    echo "4. Sample usage data:"
    jq 'select(.usage) | {type, usage}' "$TRANSCRIPT" 2>/dev/null | head -30
fi
