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
    echo "     ✗ NO token usage data in transcript either"
    echo ""
fi

echo "## Critical Findings"
echo ""
echo "❌ Token usage NOT available"
echo "   - Hook events do not include token counts"
echo "   - Transcript files do not include token counts"
echo "   - Impact: Cannot implement token-based budget enforcement"
echo "   - Recommendation: Remove token metrics from Phase 1, defer to Phase 2"
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
echo "1. Update PLAN.md to remove token usage metrics"
echo "2. Use session_id as workflow_id (no new ID generation needed)"
echo "3. Add timestamp injection in hook command"
echo "4. Focus Phase 1 on: tool usage, file edits, bash commands, phase durations"
echo ""
