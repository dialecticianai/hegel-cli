#!/bin/bash
# Systematic refactoring of colored::Colorize methods to Theme tokens
# Uses ast-grep patterns from docs/astq_patterns/rust_colored.yaml

set -e

DRY_RUN=false
if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "DRY RUN MODE - No changes will be made"
    echo
fi

# Files to refactor (from HANDOFF.md)
FILES=(
    "src/commands/analyze/sections.rs"
    "src/commands/workflow.rs"
    "src/main.rs"
    "src/commands/wrapped.rs"
    "src/commands/analyze/mod.rs"
)

echo "Theme Color Refactoring Script"
echo "==============================="
echo

# Patterns (most specific first)
SEARCH_PATTERNS=(
    '$X.bold().cyan()'
    '$X.bold().green()'
    '$X.cyan()'
    '$X.green()'
    '$X.yellow()'
    '$X.red()'
    '$X.bright_black()'
    '$X.bold()'
)

REWRITE_PATTERNS=(
    'Theme::header($X)'
    'Theme::success($X).bold()'
    'Theme::highlight($X)'
    'Theme::success($X)'
    'Theme::warning($X)'
    'Theme::error($X)'
    'Theme::secondary($X)'
    'Theme::label($X)'
)

DESCRIPTIONS=(
    'Chained bold+cyan to header'
    'Chained bold+green'
    'cyan to highlight'
    'green to success'
    'yellow to warning'
    'red to error'
    'bright_black to secondary'
    'bold to label'
)

TOTAL_CHANGES=0

for i in "${!SEARCH_PATTERNS[@]}"; do
    pattern="${SEARCH_PATTERNS[$i]}"
    rewrite="${REWRITE_PATTERNS[$i]}"
    description="${DESCRIPTIONS[$i]}"

    echo "Pattern: $description"
    echo "  Search: $pattern"
    echo "  Rewrite: $rewrite"

    for file in "${FILES[@]}"; do
        if [[ ! -f "$file" ]]; then
            continue
        fi

        # Count matches
        MATCHES=$(hegel astq -l rust -p "$pattern" "$file" 2>/dev/null | wc -l | tr -d ' ')

        if [[ "$MATCHES" -gt 0 ]]; then
            echo "  → $file: $MATCHES match(es)"
            TOTAL_CHANGES=$((TOTAL_CHANGES + MATCHES))

            if [[ "$DRY_RUN" == false ]]; then
                # Create backup
                cp "$file" "${file}.bak"

                # Apply rewrite with --update-all flag
                hegel astq -l rust -p "$pattern" -r "$rewrite" -U "$file"
                echo "    ✓ Applied"
            fi
        fi
    done
    echo
done

echo "==============================="
echo "Total matches found: $TOTAL_CHANGES"

if [[ "$DRY_RUN" == true ]]; then
    echo
    echo "Run without --dry-run to apply changes"
    echo "Backup files will be created (.bak)"
fi

if [[ "$DRY_RUN" == false && "$TOTAL_CHANGES" -gt 0 ]]; then
    echo
    echo "Changes applied! Next steps:"
    echo "1. cargo test"
    echo "2. Review diffs: git diff"
    echo "3. Commit: git add -A && git commit -m 'refactor: migrate colors to Theme tokens'"
    echo
    echo "Backup files created with .bak extension"
    echo "Remove backups: rm src/**/*.bak"
fi
