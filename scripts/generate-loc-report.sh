#!/bin/bash
# Generate LOC_REPORT.md tracking Rust code and markdown documentation
# Automated by pre-commit hook

set -e

OUTPUT_FILE="LOC_REPORT.md"
TEMP_FILE="${OUTPUT_FILE}.tmp"

# Large file threshold (implementation lines)
LARGE_FILE_THRESHOLD=400

# Paths to exclude from documentation counts
EXCLUDE_PATHS=(-not -path "./target/*" -not -path "./.git/*" -not -path "./.webcache/*" -not -path "./vendor/*")

# Files allowed to exceed ${LARGE_FILE_THRESHOLD} impl lines (infrastructure, shared utilities, etc.)
ALLOWED_LARGE_FILES=(
    "main.rs"                                # Single concern: clap command tree + dispatch (no logic)
    "commands/workflow/transitions.rs"       # Single concern: state transition logic
    "storage/mod.rs"                         # Single concern: file-based state persistence
    # Test files: suites are naturally large; splitting buys little
    "commands/workflow/tests/commands.rs"    # Infrastructure: command tests
    "commands/workflow/tests/transitions.rs" # Infrastructure: transition and state logging tests
    "engine/tests/template.rs"               # Infrastructure: template rendering tests
    "metrics/tests/unified.rs"               # Infrastructure: unified metrics tests
    "rules/tests/evaluator.rs"               # Infrastructure: rule evaluator tests
    "storage/tests/storage.rs"               # Infrastructure: storage persistence tests
)

# Check if cloc is available
if ! command -v cloc &> /dev/null; then
    echo "Error: cloc not found. Install with: brew install cloc"
    exit 1
fi

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo "Error: jq not found. Install with: brew install jq"
    exit 1
fi

# Count Rust source code
echo "Counting Rust LOC..."
RUST_JSON=$(cloc --json --quiet src 2>/dev/null)

# Parse JSON with jq
RUST_CODE=$(echo "$RUST_JSON" | jq '.Rust.code // 0')
RUST_COMMENT=$(echo "$RUST_JSON" | jq '.Rust.comment // 0')
RUST_BLANK=$(echo "$RUST_JSON" | jq '.Rust.blank // 0')
RUST_FILES=$(echo "$RUST_JSON" | jq '.Rust.nFiles // 0')

RUST_TOTAL=$((RUST_CODE + RUST_COMMENT + RUST_BLANK))

# Count markdown documentation
echo "Counting documentation LOC..."
MD_FILES=$(find . -name "*.md" "${EXCLUDE_PATHS[@]}" 2>/dev/null | wc -l | tr -d ' ')
MD_LINES=0

if [ "$MD_FILES" -gt 0 ]; then
    MD_LINES=$(find . -name "*.md" "${EXCLUDE_PATHS[@]}" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
fi

# Calculate documentation ratio
if [ "$RUST_CODE" -gt 0 ]; then
    DOC_RATIO=$(awk "BEGIN {printf \"%.2f\", $MD_LINES / $RUST_CODE}")
else
    DOC_RATIO="N/A"
fi

# Get current date
DATE=$(date +"%Y-%m-%d %H:%M")

# Helper function to format numbers with commas
format_number() {
    printf "%'d" "$1" 2>/dev/null || echo "$1"
}

# Helper function to check if a file is in the allowed large files list
is_allowed_large_file() {
    local filename="$1"
    for allowed in "${ALLOWED_LARGE_FILES[@]}"; do
        if [[ "$filename" == "$allowed" ]]; then
            return 0
        fi
    done
    return 1
}

# Start generating report
cat > "$TEMP_FILE" <<EOF
# Lines of Code Report

**Last Updated**: $DATE
**Tool**: [cloc](https://github.com/AlDanial/cloc) + wc

---

## Overall Summary

| Metric | Rust Code | Documentation (.md) | Total |
|--------|-----------|---------------------|-------|
| **Lines** | $(format_number $RUST_CODE) | $(format_number $MD_LINES) | $(format_number $((RUST_CODE + MD_LINES))) |
| **Comments** | $(format_number $RUST_COMMENT) | - | $(format_number $RUST_COMMENT) |
| **Blank Lines** | $(format_number $RUST_BLANK) | - | $(format_number $RUST_BLANK) |
| **Total Lines** | $(format_number $RUST_TOTAL) | $(format_number $MD_LINES) | $(format_number $((RUST_TOTAL + MD_LINES))) |
| **Files** | $RUST_FILES | $MD_FILES | $((RUST_FILES + MD_FILES)) |

**Documentation Ratio**: ${DOC_RATIO} lines of docs per line of code

---

## Rust Code Breakdown

\`\`\`
$(cloc src 2>/dev/null | tail -n +3)
\`\`\`

---

## Rust File Details

| File | Total Lines | Impl Lines | Test Lines | Test % | Status |
|------|-------------|------------|------------|--------|--------|
EOF

# Generate per-file breakdown and track large files
LARGE_COUNT=0
while IFS= read -r file; do
    TOTAL=$(wc -l < "$file" | tr -d ' ')

    # Find line where the inline test module starts: a `#[cfg(test)]` attribute
    # immediately followed by an inline `mod ... {`. This ignores
    # `#[cfg(test)] pub use ...` re-exports and `mod tests;` declarations that
    # point at an external test file (both of which previously caused impl lines
    # to be miscounted as test lines).
    TEST_START=$(awk '
        armed && /^(pub )?mod .*\{/ { print cfgline; exit }
        /^#\[cfg\(test\)\]/ { cfgline = NR; armed = 1; next }
        { armed = 0 }
    ' "$file")

    if [ -n "$TEST_START" ]; then
        IMPL=$((TEST_START - 1))
        TEST=$((TOTAL - TEST_START + 1))
    else
        IMPL=$TOTAL
        TEST=0
    fi

    if [ "$TOTAL" -gt 0 ]; then
        TEST_PCT=$(awk "BEGIN {printf \"%.1f\", ($TEST / $TOTAL) * 100}")
    else
        TEST_PCT="0.0"
    fi

    DISPLAY_PATH=$(echo "$file" | sed 's|^src/||')

    # Flag files with >${LARGE_FILE_THRESHOLD} impl lines (unless whitelisted)
    if [ "$IMPL" -gt "$LARGE_FILE_THRESHOLD" ]; then
        if is_allowed_large_file "$DISPLAY_PATH"; then
            STATUS="✅ (infra)"
        else
            STATUS="⚠️ Large"
            LARGE_COUNT=$((LARGE_COUNT + 1))
        fi
    else
        STATUS="✅"
    fi

    echo "| \`$DISPLAY_PATH\` | $(format_number $TOTAL) | $(format_number $IMPL) | $(format_number $TEST) | ${TEST_PCT}% | $STATUS |" >> "$TEMP_FILE"
done < <(find src -name "*.rs" -type f | sort)

# Add warning section if there are large files
if [ "$LARGE_COUNT" -gt 0 ]; then
    echo "" >> "$TEMP_FILE"
    echo "**⚠️ Warning:** $LARGE_COUNT file(s) over $LARGE_FILE_THRESHOLD impl lines - consider splitting for maintainability" >> "$TEMP_FILE"
fi

cat >> "$TEMP_FILE" <<'EOF'

---

## Documentation Files

EOF

# List all markdown files with line counts
if [ "$MD_FILES" -gt 0 ]; then
    echo "| File | Lines |" >> "$TEMP_FILE"
    echo "|------|-------|" >> "$TEMP_FILE"

    find . -name "*.md" "${EXCLUDE_PATHS[@]}" 2>/dev/null | sort | while IFS= read -r file; do
        LINES=$(wc -l < "$file" 2>/dev/null | tr -d ' ')
        DISPLAY_PATH=$(echo "$file" | sed 's|^\./||')
        echo "| \`$DISPLAY_PATH\` | $(format_number $LINES) |" >> "$TEMP_FILE"
    done
else
    echo "*No markdown files found*" >> "$TEMP_FILE"
fi

cat >> "$TEMP_FILE" <<'EOF'

---

## Documentation Quality Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
EOF

# Determine doc ratio status
if [ "$DOC_RATIO" != "N/A" ]; then
    DOC_RATIO_NUM=$(echo "$DOC_RATIO" | awk '{print $1}')

    if awk "BEGIN {exit !($DOC_RATIO_NUM >= 0.3)}"; then
        DOC_STATUS="✅ Excellent"
    elif awk "BEGIN {exit !($DOC_RATIO_NUM >= 0.15)}"; then
        DOC_STATUS="🟡 Good"
    else
        DOC_STATUS="🔴 Needs Work"
    fi
else
    DOC_STATUS="⏳ N/A"
fi

cat >> "$TEMP_FILE" <<EOF
| Docs/Code Ratio | ≥0.3 | $DOC_RATIO | $DOC_STATUS |
| README exists | Yes | $([ -f README.md ] && echo "✅" || echo "❌") | $([ -f README.md ] && echo "Met" || echo "Missing") |
| Architecture docs | Yes | $([ -f ARCHITECTURE.md ] && echo "✅" || echo "❌") | $([ -f ARCHITECTURE.md ] && echo "Met" || echo "Optional") |

---

## How to Update This Report

\`\`\`bash
# Regenerate LOC report
./scripts/generate-loc-report.sh
\`\`\`

---

*This report is auto-generated from \`cloc\` and \`wc\` output.*
*Updated automatically by pre-commit hook when source files change.*
EOF

# Move temp file to final location
mv "$TEMP_FILE" "$OUTPUT_FILE"

echo "✅ LOC report generated: $OUTPUT_FILE"
echo ""
echo "Summary: $(format_number $RUST_CODE) Rust LOC | $(format_number $MD_LINES) docs | ${DOC_RATIO} ratio"
