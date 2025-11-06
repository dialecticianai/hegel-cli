#!/usr/bin/env perl
use strict;
use warnings;
use File::Basename;
use Cwd 'abs_path';

# Usage: scan-gap-transcripts.pl <start_time> <end_time> [project_path]
# Example: scan-gap-transcripts.pl 2025-11-03T03:13:34Z 2025-11-04T22:11:56Z

my ($start_time, $end_time, $project_path) = @ARGV;

die "Usage: $0 <start_time> <end_time> [project_path]\n" unless $start_time && $end_time;

# If no project path provided, use current directory
$project_path //= abs_path('.');

# Convert project path to Claude Code directory name
# e.g., /Users/emadum/Code/github.com/dialecticianai/hegel-cli
#    -> -Users-emadum-Code-github-com-dialecticianai-hegel-cli
my $claude_dir_name = $project_path;
$claude_dir_name =~ s/^\///;  # Remove leading /
$claude_dir_name = "-$claude_dir_name";
$claude_dir_name =~ s/[\/\.]/-/g; # Replace / and . with -

my $transcripts_dir = "$ENV{HOME}/.claude/projects/$claude_dir_name";

die "Claude Code project directory not found: $transcripts_dir\n" unless -d $transcripts_dir;

print STDERR "Scanning transcripts in: $transcripts_dir\n";
print STDERR "Time range: $start_time to $end_time\n\n";

# Find all .jsonl files
opendir(my $dh, $transcripts_dir) or die "Can't open $transcripts_dir: $!";
my @transcript_files = grep { /\.jsonl$/ && -f "$transcripts_dir/$_" } readdir($dh);
closedir($dh);

print STDERR "Found " . scalar(@transcript_files) . " transcript files\n\n";

# Aggregate token metrics
my %totals = (
    input_tokens => 0,
    output_tokens => 0,
    cache_creation_tokens => 0,
    cache_read_tokens => 0,
    assistant_turns => 0,
);

foreach my $file (sort @transcript_files) {
    my $filepath = "$transcripts_dir/$file";

    print STDERR "Scanning $file...\n";

    # Use jq to filter assistant events in time range and extract token usage
    # Handle both .usage and .message.usage formats
    my $jq_filter = qq{
        select(.timestamp >= "$start_time" and .timestamp < "$end_time")
        | select(.type == "assistant")
        | (.usage // .message.usage // {})
        | select(. != {})
        | {
            input: (.input_tokens // 0),
            output: (.output_tokens // 0),
            cache_creation: (.cache_creation_input_tokens // 0),
            cache_read: (.cache_read_input_tokens // 0)
          }
    };

    open(my $jq, "-|", "jq", "-c", $jq_filter, $filepath) or die "Can't run jq: $!";

    my $file_events = 0;
    while (my $line = <$jq>) {
        chomp $line;
        next unless $line;

        # Parse JSON output from jq
        if ($line =~ /"input":(\d+).*"output":(\d+).*"cache_creation":(\d+).*"cache_read":(\d+)/) {
            $totals{input_tokens} += $1;
            $totals{output_tokens} += $2;
            $totals{cache_creation_tokens} += $3;
            $totals{cache_read_tokens} += $4;
            $totals{assistant_turns}++;
            $file_events++;
        }
    }
    close($jq);

    print STDERR "  → $file_events events matched\n" if $file_events > 0;
}

print STDERR "\n=== RESULTS ===\n";
print STDERR "Input tokens:          $totals{input_tokens}\n";
print STDERR "Output tokens:         $totals{output_tokens}\n";
print STDERR "Cache creation tokens: $totals{cache_creation_tokens}\n";
print STDERR "Cache read tokens:     $totals{cache_read_tokens}\n";
print STDERR "Assistant turns:       $totals{assistant_turns}\n";

# Output JSON to stdout for Rust to parse
print qq({"input_tokens":$totals{input_tokens},"output_tokens":$totals{output_tokens},"cache_creation_tokens":$totals{cache_creation_tokens},"cache_read_tokens":$totals{cache_read_tokens},"assistant_turns":$totals{assistant_turns}}\n);
