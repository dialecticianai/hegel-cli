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

echo "1. Checking for message.usage fields (correct path):"
USAGE_COUNT=$(jq 'select(.message.usage) | .message.usage' "$TRANSCRIPT" 2>/dev/null | wc -l)
echo "   Events with message.usage: $USAGE_COUNT"
echo ""

if [[ $USAGE_COUNT -gt 0 ]]; then
    echo "2. Sample token usage data:"
    jq 'select(.message.usage) | {timestamp, model: .message.model, usage: .message.usage}' "$TRANSCRIPT" 2>/dev/null | head -40
else
    echo "2. No token usage found at message.usage path"
fi
