#!/usr/bin/env perl
# Archive one-off scripts to scripts/oneoffs/ with git addition date
# Usage: ./scripts/archive-oneoff.pl [--dry-run] <script-name> [<script-name> ...]
# Example: ./scripts/archive-oneoff.pl refactor-theme-colors.sh debug-meta-mode-bug.sh

use strict;
use warnings;
use File::Basename;
use File::Copy;
use Cwd 'abs_path';

# Parse arguments
my $dry_run = 0;
my @script_names;

for my $arg (@ARGV) {
    if ($arg eq '--dry-run') {
        $dry_run = 1;
    } else {
        push @script_names, $arg;
    }
}

# Validate arguments
unless (@script_names) {
    print STDERR "Usage: $0 [--dry-run] <script-name> [<script-name> ...]\n";
    print STDERR "Example: $0 refactor-theme-colors.sh debug-meta-mode-bug.sh\n";
    exit 1;
}

print "Archive One-Off Scripts\n";
print "=" x 50 . "\n";
print "DRY RUN MODE\n" if $dry_run;
print "\n";

# Determine script directory (scripts/)
my $script_dir = dirname(abs_path($0));
my $oneoffs_dir = "$script_dir/oneoffs";

# Create oneoffs directory if it doesn't exist
unless (-d $oneoffs_dir) {
    print "Creating oneoffs directory...\n";
    unless ($dry_run) {
        mkdir $oneoffs_dir or die "Can't create $oneoffs_dir: $!";
        print "  ✓ Created $oneoffs_dir\n";
    }
    print "\n";
}

# Track results
my @successful;
my @failed;

# Process each script
for my $script_name (@script_names) {
    print "Processing: $script_name\n";
    print "-" x 50 . "\n";

    my $source_path = "$script_dir/$script_name";

    # Validate source file exists
    unless (-f $source_path) {
        print STDERR "  ✗ Error: Script not found: $source_path\n";
        push @failed, $script_name;
        print "\n";
        next;
    }

    # Check if file is already in oneoffs
    if ($source_path =~ m{/oneoffs/}) {
        print STDERR "  ✗ Error: File is already in oneoffs directory\n";
        push @failed, $script_name;
        print "\n";
        next;
    }

    # Get git addition date
    my $git_date = `git log --follow --format=%ad --date=short '$source_path' 2>/dev/null | tail -1`;
    chomp $git_date;

    unless ($git_date) {
        print STDERR "  ✗ Warning: Could not find git history for $script_name\n";
        print STDERR "     This file may not be committed yet.\n";

        # For batch operations, skip files without git history
        if (@script_names > 1) {
            print STDERR "     Skipping (use single-file mode for interactive date selection)\n";
            push @failed, $script_name;
            print "\n";
            next;
        } else {
            # Single file mode - offer interactive fallback
            print "Use current date instead? (y/n): ";
            my $response = <STDIN>;
            chomp $response;

            if ($response =~ /^y/i) {
                $git_date = `date +%Y-%m-%d`;
                chomp $git_date;
                print "  Using current date: $git_date\n";
            } else {
                print "  Aborted.\n";
                push @failed, $script_name;
                print "\n";
                next;
            }
        }
    }

    print "  ✓ Git addition date: $git_date\n";

    # Construct destination filename
    my $dest_filename = "$git_date-$script_name";
    my $dest_path = "$oneoffs_dir/$dest_filename";

    # Check if destination already exists
    if (-e $dest_path) {
        print STDERR "  ✗ Error: Destination file already exists: $dest_path\n";
        push @failed, $script_name;
        print "\n";
        next;
    }

    # Show the move operation
    print "  Move: scripts/$script_name → oneoffs/$dest_filename\n";

    # Perform the move (unless dry-run)
    if ($dry_run) {
        print "  (DRY RUN - not executed)\n";
        push @successful, {name => $script_name, dest => $dest_filename, dry_run => 1};
    } else {
        # Use git mv if file is tracked
        my $is_tracked = `git ls-files '$source_path' 2>/dev/null`;
        chomp $is_tracked;

        if ($is_tracked) {
            my $result = system("git", "mv", $source_path, $dest_path);

            if ($result == 0) {
                print "  ✓ Moved successfully (git mv - staged for commit)\n";
                push @successful, {name => $script_name, dest => $dest_filename, git => 1};
            } else {
                print STDERR "  ✗ Error: git mv failed with exit code $result\n";
                push @failed, $script_name;
            }
        } else {
            if (move($source_path, $dest_path)) {
                print "  ✓ Moved successfully (untracked file)\n";
                push @successful, {name => $script_name, dest => $dest_filename, git => 0};
            } else {
                print STDERR "  ✗ Error: Move failed: $!\n";
                push @failed, $script_name;
            }
        }
    }

    print "\n";
}

# Summary
print "=" x 50 . "\n";
print "SUMMARY\n";
print "=" x 50 . "\n";
print "Successful: " . scalar(@successful) . "\n";
print "Failed:     " . scalar(@failed) . "\n";
print "\n";

if (@successful) {
    print "Successfully processed:\n";
    for my $item (@successful) {
        my $status = $item->{dry_run} ? "(dry-run)" :
                     $item->{git} ? "(staged)" : "(untracked)";
        print "  ✓ $item->{name} → oneoffs/$item->{dest} $status\n";
    }
    print "\n";
}

if (@failed) {
    print "Failed to process:\n";
    for my $name (@failed) {
        print "  ✗ $name\n";
    }
    print "\n";
}

# Next steps
if (@successful && !$dry_run) {
    my $any_git = grep { $_->{git} } @successful;
    if ($any_git) {
        print "Next steps:\n";
        print "  1. Review the moves: git status\n";
        print "  2. Commit: git commit -m 'chore(scripts): archive one-off scripts to oneoffs'\n";
    }
} elsif (@successful && $dry_run) {
    print "Run without --dry-run to perform the moves\n";
}

# Exit with error if any failed
exit(@failed ? 1 : 0);
