#!/usr/bin/env bash
# Debug why hegel analyze shows "No token data found"
# Usage: ./scripts/debug-analyze.sh

set -euo pipefail

HOOKS_FILE="${1:-.hegel/hooks.jsonl}"

echo "=== Debug: hegel analyze token parsing ==="
echo ""

echo "1. First valid hook event has transcript_path?"
# Use grep to find lines with transcript_path, then parse first valid one
TRANSCRIPT_PATH=$(grep -m 1 '"transcript_path"' "$HOOKS_FILE" | jq -r '.transcript_path // empty' 2>/dev/null)

if [[ -n "$TRANSCRIPT_PATH" ]] && [[ "$TRANSCRIPT_PATH" != "null" ]]; then
    echo "   ✓ Yes: $TRANSCRIPT_PATH"
else
    echo "   ✗ No transcript_path found in hooks.jsonl"
    echo "   Trying to find ANY valid JSON line..."
    grep '"session_id"' "$HOOKS_FILE" | head -3 | jq -r '.session_id // "PARSE_FAILED"' 2>/dev/null || echo "   All lines malformed"
    exit 1
fi
echo ""

echo "2. Transcript file exists?"
if [[ -f "$TRANSCRIPT_PATH" ]]; then
    echo "   ✓ Yes"
else
    echo "   ✗ No: $TRANSCRIPT_PATH"
    exit 1
fi
echo ""

echo "3. Transcript has token usage?"
echo "   Checking for .usage field..."
jq 'select(.usage) | {type, has_usage: true}' "$TRANSCRIPT_PATH" 2>/dev/null | head -3
echo ""
echo "   Checking for .message.usage field..."
jq 'select(.message.usage) | {type, has_usage: true}' "$TRANSCRIPT_PATH" 2>/dev/null | head -3
echo ""

echo "4. Sample token usage from transcript:"
jq 'select(.usage) | {type, usage}' "$TRANSCRIPT_PATH" | head -15
echo ""

echo "5. Check ALL unique transcript paths for token data:"
UNIQUE_TRANSCRIPTS=$(grep '"transcript_path"' "$HOOKS_FILE" | jq -r '.transcript_path // empty' 2>/dev/null | sort -u)
echo "$UNIQUE_TRANSCRIPTS" | while IFS= read -r transcript; do
    if [[ -f "$transcript" ]]; then
        TOKEN_COUNT=$(jq 'select(.usage)' "$transcript" 2>/dev/null | wc -l | tr -d ' ')
        if [[ "$TOKEN_COUNT" -gt 0 ]]; then
            echo "   ✓ $transcript: $TOKEN_COUNT events with tokens"
        else
            echo "   ✗ $transcript: NO token data"
        fi
    else
        echo "   ✗ $transcript: FILE NOT FOUND"
    fi
done
echo ""

echo "6. Root cause:"
echo "   parse_unified_metrics only checks FIRST hook event's transcript"
echo "   But that transcript might not have token data!"
echo "   FIX: Loop through all hooks to find a transcript WITH tokens"
