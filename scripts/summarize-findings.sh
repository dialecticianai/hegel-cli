#!/usr/bin/env bash
# Summarize findings from hook event analysis
# Usage: ./scripts/summarize-findings.sh

set -euo pipefail

echo "=== Hook Event Analysis Summary ==="
echo ""
echo "Date: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

echo "## Data Sources"
echo ""
echo "1. Claude Code Hook Events (.hegel/hooks.jsonl)"
echo "   - Total events captured: $(wc -l < .hegel/hooks.jsonl)"
echo "   - Event types: PreToolUse, PostToolUse, UserPromptSubmit, Stop, SessionStart"
echo "   - Available fields:"
echo "     ✓ session_id (workflow run identifier)"
echo "     ✓ tool_name (Bash, Read, Write, Edit, Glob, TodoWrite, etc.)"
echo "     ✓ tool_input (command parameters)"
echo "     ✓ tool_response (stdout, stderr for Bash; content for reads)"
echo "     ✓ cwd (current working directory)"
echo "     ✓ transcript_path (path to full session transcript)"
echo "     ✗ NO timestamp field"
echo "     ✗ NO token usage data"
echo ""

TRANSCRIPT=$(jq -r 'select(.transcript_path) | .transcript_path' .hegel/hooks.jsonl | sort -u | head -1)
if [[ -f "$TRANSCRIPT" ]]; then
    echo "2. Claude Code Transcript Files (transcript_path)"
    echo "   - Events in sample transcript: $(wc -l < "$TRANSCRIPT")"
    echo "   - Event types: user, assistant, system, file-history-snapshot, summary"
    echo "   - Available fields:"
    USAGE_COUNT=$(jq 'select(.message.usage)' "$TRANSCRIPT" 2>/dev/null | wc -l | tr -d ' ')
    echo "     ✓ Token usage at message.usage ($USAGE_COUNT events)"
    echo "     ✓ Fields: input_tokens, output_tokens, cache_*_input_tokens"
    echo ""
fi

echo "## Critical Findings"
echo ""
echo "✓ Token usage AVAILABLE (corrected)"
echo "   - Hook events do NOT include token counts directly"
echo "   - BUT transcript files (via transcript_path) include full token data"
echo "   - Location: message.usage in transcript JSONL files"
echo "   - Fields: input_tokens, output_tokens, cache_*_input_tokens"
echo "   - Reference: claude-code-leaderboard npm package uses this approach"
echo ""

echo "✓ Workflow run identity RESOLVED"
echo "   - Use session_id as workflow_id"
echo "   - Format: UUID (e.g., 3712d8c3-5565-4775-9277-a5d8942d54ca)"
echo "   - Unique per Claude Code session"
echo ""

echo "⚠ Timestamps MISSING from hooks"
echo "   - Hook events have no timestamp field"
echo "   - Solution: Add timestamp on write (server-side)"
echo "   - Implementation: Update hook command to inject timestamp"
echo ""

echo "✓ Tool usage data COMPLETE"
echo "   - All tool calls captured with full parameters"
echo "   - File edits trackable via Edit/Write tool_input"
echo "   - Bash commands captured with stdout/stderr"
echo ""

echo "## Recommendations"
echo ""
echo "1. ✓ Token usage metrics CAN be implemented via transcript file parsing"
echo "2. Use session_id as workflow_id (no new ID generation needed)"
echo "3. Add timestamp injection in hook command"
echo "4. Phase 1 scope: tool usage, file edits, bash commands, phase durations, AND token metrics"
echo "5. Reference claude-code-leaderboard for transcript parsing implementation"
echo ""
