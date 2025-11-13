#!/bin/bash
# Add #[serial] attribute to tests that mutate working directory
# This prevents parallel execution race conditions

set -e

# Files that contain tests using setup_workflow_env or setup_production_workflows
FILES=(
    "src/commands/meta.rs"
    "src/commands/workflow.rs"
)

for file in "${FILES[@]}"; do
    echo "Processing $file..."

    # Use Perl to add #[serial] to test functions in files with cwd-changing helpers
    # Only add if not already present
    perl -i -pe '
        # Track if we are in a test module
        if (/^mod tests \{/ || /^#\[cfg\(test\)\]/) {
            $in_test_mod = 1;
        }

        # If we see #[test] and we are in test module, add #[serial] before it
        if ($in_test_mod && /^\s+#\[test\]/) {
            # Look ahead to see if #[serial] is already there
            unless ($prev_line =~ /#\[serial\]/) {
                # Add the serial import at the top if not present (will be deduplicated)
                $needs_import = 1;

                # Insert #[serial] with same indentation
                my $indent = $_;
                $indent =~ s/#\[test\].*//;
                $_ = $indent . "use serial_test::serial;\n" . $indent . "#[serial]\n" . $_;
            }
        }

        $prev_line = $_;
    ' "$file"

    echo "✓ Done with $file"
done

echo ""
echo "✓ All files updated"
echo ""
echo "Next: cargo build to install serial_test crate"
