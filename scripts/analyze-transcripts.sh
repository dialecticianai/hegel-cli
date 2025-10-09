#!/usr/bin/env bash
# Analyze Claude Code transcript files for token usage
# Usage: ./scripts/analyze-transcripts.sh [hooks_file]

set -euo pipefail

HOOKS_FILE="${1:-.hegel/hooks.jsonl}"

if [[ ! -f "$HOOKS_FILE" ]]; then
    echo "Error: $HOOKS_FILE not found" >&2
    exit 1
fi

echo "=== Transcript Analysis ==="
echo ""

# Extract unique transcript paths
TRANSCRIPTS=$(jq -r 'select(.transcript_path) | .transcript_path' "$HOOKS_FILE" | sort -u)

if [[ -z "$TRANSCRIPTS" ]]; then
    echo "No transcript paths found in hooks file"
    exit 0
fi

echo "Found transcript paths:"
echo "$TRANSCRIPTS"
echo ""

# Analyze first transcript
FIRST_TRANSCRIPT=$(echo "$TRANSCRIPTS" | head -1)

if [[ ! -f "$FIRST_TRANSCRIPT" ]]; then
    echo "Warning: Transcript file not found: $FIRST_TRANSCRIPT"
    exit 1
fi

echo "--- Analyzing: $FIRST_TRANSCRIPT ---"
echo "Total events: $(wc -l < "$FIRST_TRANSCRIPT")"
echo ""

echo "Event types:"
jq -r '.type // .event_type // "unknown"' "$FIRST_TRANSCRIPT" | sort | uniq -c | sort -rn
echo ""

echo "Sample event with token usage:"
jq 'select(.message.usage) | {type, usage: .message.usage}' "$FIRST_TRANSCRIPT" | head -20
echo ""

echo "Token usage summary:"
echo "Events with usage data: $(jq 'select(.message.usage)' "$FIRST_TRANSCRIPT" | wc -l)"
echo ""

echo "Sample usage fields:"
jq 'select(.message.usage) | .message.usage | keys' "$FIRST_TRANSCRIPT" | head -5 | jq -s 'add | unique'
