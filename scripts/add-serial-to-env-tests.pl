#!/usr/bin/env perl
# Add #[serial] to tests that use setup_workflow_env or setup_production_workflows
# This prevents env var races from parallel test execution

use strict;
use warnings;

my @files = (
    'src/commands/meta.rs',
    'src/commands/workflow.rs',
);

for my $file (@files) {
    my $content = read_file($file);
    my $modified = 0;

    # Pattern: find test functions that call setup_workflow_env or setup_production_workflows
    # and don't already have #[serial]
    if ($content =~ s/(\n\s+#\[test\]\n)(\s+fn\s+\w+.*?\{\s*\n\s+let.*?setup_(?:workflow_env|production_workflows))/\n    use serial_test::serial;\n$1    #[serial]\n$2/gms) {
        $modified = 1;
    }

    if ($modified) {
        write_file($file, $content);
        print "✓ Updated $file\n";
    } else {
        print "  Skipped $file (no changes needed)\n";
    }
}

sub read_file {
    my ($path) = @_;
    open my $fh, '<', $path or die "Cannot read $path: $!";
    local $/;
    return <$fh>;
}

sub write_file {
    my ($path, $content) = @_;
    open my $fh, '>', $path or die "Cannot write $path: $!";
    print $fh $content;
}
