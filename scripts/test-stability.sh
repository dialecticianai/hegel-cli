#!/bin/bash
# Test stability checker - runs test suite N times to detect flaky tests
# Usage: ./scripts/test-stability.sh [iterations]

ITERATIONS=${1:-50}
FAILURES=0
FAILED_RUNS=""
TEMP_DIR=$(mktemp -d)
FAILED_TESTS_FILE="$TEMP_DIR/failed_tests.txt"

echo "Running test suite $ITERATIONS times to check for flaky tests..."
echo "Started at: $(date)"
echo ""

for i in $(seq 1 $ITERATIONS); do
    printf "Run %3d/%d: " "$i" "$ITERATIONS"

    TEST_OUTPUT=$(cargo test --bin hegel --quiet 2>&1)

    if echo "$TEST_OUTPUT" | grep -q "test result: ok"; then
        echo "✓ PASS"
    else
        echo "✗ FAIL"
        FAILURES=$((FAILURES + 1))
        FAILED_RUNS="$FAILED_RUNS $i"

        # Extract failed test names and append to file
        echo "$TEST_OUTPUT" | grep " --- FAILED$" | sed 's/ --- FAILED$//' >> "$FAILED_TESTS_FILE"
    fi
done

echo ""
echo "Completed at: $(date)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Results: $((ITERATIONS - FAILURES))/$ITERATIONS passed"

if [ $FAILURES -eq 0 ]; then
    echo "Status: ✓ ALL STABLE - No flaky tests detected"
    rm -rf "$TEMP_DIR"
    exit 0
else
    echo "Status: ✗ FLAKY TESTS DETECTED"
    echo "Failed runs:$FAILED_RUNS"
    echo ""
    echo "Flaky tests (aggregated across all runs):"
    if [ -f "$FAILED_TESTS_FILE" ]; then
        sort "$FAILED_TESTS_FILE" | uniq -c | sort -rn | while read count test; do
            printf "  %2d/%d  %s\n" "$count" "$ITERATIONS" "$test"
        done
    fi
    rm -rf "$TEMP_DIR"
    exit 1
fi
