#!/bin/bash
# Generate COVERAGE_REPORT.md from llvm-cov output
# This creates a git-diff-friendly report with stable formatting

set -e

REPORT_FILE="COVERAGE_REPORT.md"
TEMP_FILE="${REPORT_FILE}.tmp"

# Check if cargo-llvm-cov is installed
if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "Error: cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov"
    exit 1
fi

# Generate coverage data silently with timeout and retry
echo "Generating coverage data..."

# Run coverage with timeout and retry (skip ignored tests to avoid flaky failures)
MAX_ATTEMPTS=3
ATTEMPT=1
TIMEOUT=30
SUCCESS=0

while [ $ATTEMPT -le $MAX_ATTEMPTS ]; do
    if [ $ATTEMPT -gt 1 ]; then
        echo "Retry $ATTEMPT/$MAX_ATTEMPTS..."
    fi

    # Run with timeout (30 seconds should be plenty for our small codebase)
    timeout $TIMEOUT cargo llvm-cov --summary-only -- --skip ignored > /tmp/coverage-raw.txt 2>&1
    EXIT_CODE=$?

    # Check if successful (exit code 0)
    if [ $EXIT_CODE -eq 0 ]; then
        SUCCESS=1
        break
    fi

    # Exit code 124 means timeout
    if [ $EXIT_CODE -eq 124 ]; then
        echo "Warning: cargo llvm-cov timed out after ${TIMEOUT}s"
    else
        echo "Warning: cargo llvm-cov failed with exit code $EXIT_CODE"
    fi

    ATTEMPT=$((ATTEMPT + 1))

    # If this was the last attempt, fail
    if [ $ATTEMPT -gt $MAX_ATTEMPTS ]; then
        echo "Error: Failed to generate coverage after $MAX_ATTEMPTS attempts"
        exit 1
    fi

    # Wait before retry
    sleep 1
done

# Double-check we actually succeeded
if [ $SUCCESS -eq 0 ]; then
    echo "Error: Coverage generation did not complete successfully"
    exit 1
fi

# Strip ANSI color codes from output
sed 's/\x1b\[[0-9;]*m//g' /tmp/coverage-raw.txt > /tmp/coverage-summary.txt

# Extract summary line (TOTAL row)
SUMMARY=$(grep "^TOTAL" /tmp/coverage-summary.txt)
LINE_COV=$(echo "$SUMMARY" | awk '{print $10}')      # Field 10 is line coverage %
REGION_COV=$(echo "$SUMMARY" | awk '{print $4}')     # Field 4 is region coverage %
FUNC_COV=$(echo "$SUMMARY" | awk '{print $7}')       # Field 7 is function coverage %

# Parse line coverage percentage
LINE_PCT=$(echo "$LINE_COV" | sed 's/%//')

# Get current date
DATE=$(date +"%Y-%m-%d %H:%M")

# Start generating report
cat > "$TEMP_FILE" <<EOF
# Test Coverage Report

**Last Updated**: $DATE
**Tool**: cargo-llvm-cov
**Overall Coverage**: **${LINE_COV}** lines | **${REGION_COV}** regions | **${FUNC_COV}** functions

## Summary

\`\`\`
$(grep "^TOTAL" /tmp/coverage-summary.txt | head -1)
\`\`\`

## Coverage by Module

| Module | Line Coverage | Region Coverage | Functions | Status |
|--------|--------------|-----------------|-----------|--------|
EOF

# Parse module coverage (skip header and TOTAL, sort alphabetically)
# Match only lines starting with filename patterns (not warning lines)
grep -E "^[a-z_/]+\.rs " /tmp/coverage-summary.txt | grep -v "^TOTAL" | sort | while IFS= read -r line; do
    MODULE=$(echo "$line" | awk '{print $1}')
    LINE_COV=$(echo "$line" | awk '{print $10}')      # Field 10 is line coverage %
    REGION_COV=$(echo "$line" | awk '{print $4}')     # Field 4 is region coverage %
    FUNC_COV=$(echo "$line" | awk '{print $7}')       # Field 7 is function coverage %

    # Determine status emoji based on line coverage
    LINE_NUM=$(echo "$LINE_COV" | sed 's/%//')
    # Use awk for float comparison (more portable than bc)
    if awk "BEGIN {exit !($LINE_NUM >= 90)}"; then
        STATUS="🟢 Excellent"
    elif awk "BEGIN {exit !($LINE_NUM >= 70)}"; then
        STATUS="🟡 Good"
    elif awk "BEGIN {exit !($LINE_NUM >= 40)}"; then
        STATUS="🟠 Moderate"
    else
        STATUS="🔴 Needs Work"
    fi

    echo "| \`$MODULE\` | $LINE_COV | $REGION_COV | $FUNC_COV | $STATUS |" >> "$TEMP_FILE"
done

# Add coverage tiers section
cat >> "$TEMP_FILE" <<'EOF'

## Coverage Tiers

### 🟢 Excellent (≥90% lines)
EOF

grep -E "^[a-z_/]+\.rs " /tmp/coverage-summary.txt | grep -v "^TOTAL" | while IFS= read -r line; do
    LINE_COV=$(echo "$line" | awk '{print $10}' | sed 's/%//')
    if awk "BEGIN {exit !($LINE_COV >= 90)}"; then
        MODULE=$(echo "$line" | awk '{print $1}')
        echo "- \`$MODULE\` - $(echo "$line" | awk '{print $10}')" >> "$TEMP_FILE"
    fi
done

cat >> "$TEMP_FILE" <<'EOF'

### 🟡 Good (70-89% lines)
EOF

grep -E "^[a-z_/]+\.rs " /tmp/coverage-summary.txt | grep -v "^TOTAL" | while IFS= read -r line; do
    LINE_COV=$(echo "$line" | awk '{print $10}' | sed 's/%//')
    if awk "BEGIN {exit !($LINE_COV >= 70 && $LINE_COV < 90)}"; then
        MODULE=$(echo "$line" | awk '{print $1}')
        echo "- \`$MODULE\` - $(echo "$line" | awk '{print $10}')" >> "$TEMP_FILE"
    fi
done

cat >> "$TEMP_FILE" <<'EOF'

### 🟠 Moderate (40-69% lines)
EOF

grep -E "^[a-z_/]+\.rs " /tmp/coverage-summary.txt | grep -v "^TOTAL" | while IFS= read -r line; do
    LINE_COV=$(echo "$line" | awk '{print $10}' | sed 's/%//')
    if awk "BEGIN {exit !($LINE_COV >= 40 && $LINE_COV < 70)}"; then
        MODULE=$(echo "$line" | awk '{print $1}')
        echo "- \`$MODULE\` - $(echo "$line" | awk '{print $10}')" >> "$TEMP_FILE"
    fi
done

cat >> "$TEMP_FILE" <<'EOF'

### 🔴 Needs Work (<40% lines)
EOF

grep -E "^[a-z_/]+\.rs " /tmp/coverage-summary.txt | grep -v "^TOTAL" | while IFS= read -r line; do
    LINE_COV=$(echo "$line" | awk '{print $10}' | sed 's/%//')
    if awk "BEGIN {exit !($LINE_COV < 40)}"; then
        MODULE=$(echo "$line" | awk '{print $1}')
        echo "- \`$MODULE\` - $(echo "$line" | awk '{print $10}')" >> "$TEMP_FILE"
    fi
done

# Add targets and guidelines
cat >> "$TEMP_FILE" <<'EOF'

## Coverage Targets

| Tier | Target | Current | Status |
|------|--------|---------|--------|
EOF

# Calculate current tier coverage
OVERALL_PCT=$LINE_PCT

if awk "BEGIN {exit !($OVERALL_PCT >= 80)}"; then
    OVERALL_STATUS="✅ Met"
else
    OVERALL_STATUS="⏳ In Progress"
fi

cat >> "$TEMP_FILE" <<EOF
| Overall | ≥80% | ${LINE_COV} | ${OVERALL_STATUS} |
| Critical Paths | ≥95% | Check modules above | Policy |
| New Modules | ≥80% | - | Policy |

## How to Update This Report

\`\`\`bash
# Regenerate coverage report
./scripts/generate-coverage-report.sh
\`\`\`

## Quick Commands

\`\`\`bash
# Run tests with coverage
cargo llvm-cov --html      # Detailed HTML
cargo llvm-cov --summary-only  # Terminal summary

# Update this markdown report
./scripts/generate-coverage-report.sh
\`\`\`

---

*This report is auto-generated from \`cargo llvm-cov\` output.*
EOF

# Move temp file to final location
mv "$TEMP_FILE" "$REPORT_FILE"

echo "✅ Coverage report generated: $REPORT_FILE"
echo ""
echo "Summary: ${LINE_COV} lines | ${REGION_COV} regions | ${FUNC_COV} functions"
