#!/usr/bin/env perl
use strict;
use warnings;
use File::Spec;
use Getopt::Long;

# Fix archive phase duplication bug by using jq for JSON manipulation
#
# Usage:
#   ./scripts/oneoffs/20251103-fix-archive-duplication.pl <hegel-dir> [--dry-run]
#
# This script fixes the exponential phase duplication bug where each archive
# incorrectly includes all phases from previous archives.
#
# Strategy:
# 1. Read all archives in chronological order using jq
# 2. Track unique phases by "phase_name:start_time" signature
# 3. For each archive, filter out phases that appeared in earlier archives
# 4. Rewrite cleaned archives with recalculated totals

my $dry_run = 0;
GetOptions('dry-run' => \$dry_run) or die "Usage: $0 <hegel-dir> [--dry-run]\n";

my $hegel_dir = shift @ARGV or die "Usage: $0 <hegel-dir> [--dry-run]\n";
my $archive_dir = File::Spec->catdir($hegel_dir, 'archive');

unless (-d $archive_dir) {
    die "Error: Archive directory not found: $archive_dir\n";
}

print "=== Archive Duplication Fix ===\n";
print "Directory: $hegel_dir\n";
print "Mode: " . ($dry_run ? "DRY RUN (no changes)" : "LIVE (will modify files)") . "\n";
print "\n";

# Get list of archive files sorted by filename (chronological)
my @archive_files = sort glob("$archive_dir/*.json");
print "Found " . scalar(@archive_files) . " archive files\n\n";

# Track phases we've seen: "phase_name:start_time" => workflow_id
my %seen_phases;

my $total_phases_before = 0;
my $total_phases_after = 0;
my $archives_modified = 0;

my $file_num = 0;
foreach my $filepath (@archive_files) {
    $file_num++;
    my $filename = (File::Spec->splitpath($filepath))[2];

    print "[$file_num/" . scalar(@archive_files) . "] Processing $filename\n";

    # Get workflow_id using jq
    my $workflow_id = `jq -r '.workflow_id' "$filepath"`;
    chomp $workflow_id;

    # Get phase count using jq
    my $original_count = `jq '.phases | length' "$filepath"`;
    chomp $original_count;
    $total_phases_before += $original_count;

    print "  Workflow: $workflow_id\n";
    print "  Original phases: $original_count\n";

    # Get all phases as "phase_name:start_time" pairs
    my @phase_sigs = `jq -r '.phases[] | "\\(.phase_name):\\(.start_time)"' "$filepath"`;
    chomp @phase_sigs;

    # Determine which phases to keep
    my @keep_indices;
    my $removed_count = 0;

    for (my $i = 0; $i < @phase_sigs; $i++) {
        my $sig = $phase_sigs[$i];

        if (exists $seen_phases{$sig}) {
            # Duplicate from earlier archive
            $removed_count++;
        } else {
            # New phase - keep it
            push @keep_indices, $i;
            $seen_phases{$sig} = $workflow_id;
        }
    }

    my $cleaned_count = scalar(@keep_indices);
    $total_phases_after += $cleaned_count;

    print "  Cleaned phases: $cleaned_count\n";

    if ($removed_count > 0) {
        $archives_modified++;
        print "  Removed duplicates: $removed_count\n";

        unless ($dry_run) {
            # Build jq filter to keep only non-duplicate phases
            my $indices_str = join(',', @keep_indices);

            # Create cleaned archive with only kept phases
            my $jq_filter = ".phases = [.phases[$indices_str]]";

            # Recalculate totals
            $jq_filter .= ' | .totals.tokens = {
                input: ([.phases[].tokens.input] | add // 0),
                output: ([.phases[].tokens.output] | add // 0),
                cache_creation: ([.phases[].tokens.cache_creation] | add // 0),
                cache_read: ([.phases[].tokens.cache_read] | add // 0),
                assistant_turns: ([.phases[].tokens.assistant_turns] | add // 0)
            }';

            # Write to temp file then replace original
            my $temp_file = "$filepath.tmp";
            system("jq '$jq_filter' '$filepath' > '$temp_file'") == 0
                or die "jq failed: $!";

            rename($temp_file, $filepath)
                or die "Failed to replace $filepath: $!";

            print "  ✓ Archive cleaned and rewritten\n";
        }
    } else {
        print "  ✓ No duplicates found\n";
    }

    print "\n";
}

# Summary
print "=== Summary ===\n";
print "Total phases before: $total_phases_before\n";
print "Total phases after:  $total_phases_after\n";
print "Phases removed:      " . ($total_phases_before - $total_phases_after) . "\n";
print "Archives modified:   $archives_modified / " . scalar(@archive_files) . "\n";
print "\n";

if ($dry_run) {
    print "DRY RUN: No files were modified.\n";
    print "Run without --dry-run to apply changes.\n";
} else {
    print "✓ Archives have been cleaned!\n";
    print "\nVerify with: ./scripts/debug-phase-count.sh $hegel_dir\n";
}
