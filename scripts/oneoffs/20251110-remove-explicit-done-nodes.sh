#!/bin/bash
# Remove explicit done nodes from all workflow YAML files
# Done nodes are now implicit and auto-injected by the engine

set -e

for workflow in workflows/*.yaml; do
    echo "Processing $workflow..."

    # Remove the done node definition (including the line before if it's a blank line)
    # Pattern: optional blank line, then "  done:", then "    transitions: []"
    sed -i '' '/^$/N;/\n  done:$/!P;D' "$workflow"
    sed -i '' '/^  done:$/,/^    transitions: \[\]$/d' "$workflow"

    echo "  ✓ Removed explicit done node"
done

echo ""
echo "✓ All workflows updated - done nodes are now implicit"
