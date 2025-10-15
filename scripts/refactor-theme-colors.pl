#!/usr/bin/env perl
# Robust color refactoring: discovers all cases, auto-fixes simple ones, reports complex ones
# Uses identifier search for discovery, metavariable patterns for simple rewrites

use strict;
use warnings;
use File::Basename;

my $dry_run = (@ARGV && $ARGV[0] eq '--dry-run');

print "Theme Color Refactoring (Robust)\n";
print "=" x 50 . "\n\n";
print "DRY RUN MODE\n\n" if $dry_run;

# Files to refactor
my @files = qw(
    src/commands/analyze/sections.rs
    src/commands/workflow.rs
    src/main.rs
    src/commands/wrapped.rs
    src/commands/analyze/mod.rs
);

# Color methods to refactor
# Use grep for discovery
my %colors = (
    'bold().cyan' => { grep => '\.bold()\.cyan()', simple => '$X.bold().cyan()', rewrite => 'Theme::header($X)', name => 'header' },
    'bold().green' => { grep => '\.bold()\.green()', simple => '$X.bold().green()', rewrite => 'Theme::success($X).bold()', name => 'success.bold' },
    'cyan' => { grep => '\.cyan()', simple => '$X.cyan()', rewrite => 'Theme::highlight($X)', name => 'highlight' },
    'green' => { grep => '\.green()', simple => '$X.green()', rewrite => 'Theme::success($X)', name => 'success' },
    'yellow' => { grep => '\.yellow()', simple => '$X.yellow()', rewrite => 'Theme::warning($X)', name => 'warning' },
    'red' => { grep => '\.red()', simple => '$X.red()', rewrite => 'Theme::error($X)', name => 'error' },
    'bright_black' => { grep => '\.bright_black()', simple => '$X.bright_black()', rewrite => 'Theme::secondary($X)', name => 'secondary' },
    'bold' => { grep => '\.bold()', simple => '$X.bold()', rewrite => 'Theme::label($X)', name => 'label' },
);

my %stats = (
    total_found => 0,
    auto_fixed => 0,
    manual_needed => 0,
);

my @manual_cases;

# Phase 1: Discovery - find ALL occurrences using grep
print "Phase 1: Discovery (finding ALL color calls)\n";
print "-" x 50 . "\n";

for my $color_pattern (sort keys %colors) {
    my $grep_pattern = $colors{$color_pattern}{grep};

    for my $file (@files) {
        next unless -f $file;

        my $output = `grep -n '$grep_pattern' '$file' 2>/dev/null`;
        my @matches = split /\n/, $output;

        if (@matches) {
            print "  $color_pattern in $file: " . scalar(@matches) . " total\n";

            # Analyze each match
            for my $match (@matches) {
                if ($match =~ /^(\d+):(.*)/) {
                    my ($line, $code) = ($1, $2);
                    $code =~ s/^\s+//;

                    # Skip if this is a chained pattern that will be handled by a more specific pattern
                    # e.g., skip .cyan() if it's part of .bold().cyan()
                    next if ($color_pattern eq 'cyan' && $code =~ /\.bold\(\)\.cyan\(\)/);
                    next if ($color_pattern eq 'green' && $code =~ /\.bold\(\)\.green\(\)/);
                    next if ($color_pattern eq 'bold' && $code =~ /\.bold\(\)\.(?:cyan|green)\(\)/);

                    $stats{total_found}++;

                    # Simple heuristic: if it starts with a literal or simple identifier, it's auto-fixable
                    # Complex: starts with function call, macro, or has dots before the color method
                    my $is_simple = 0;
                    if ($code =~ /^"[^"]*"$grep_pattern/ || $code =~ /^\w+$grep_pattern/) {
                        $is_simple = 1;
                    }

                    if ($is_simple) {
                        print "    L$line: SIMPLE (auto-fixable)\n" if $dry_run;
                    } else {
                        print "    L$line: COMPLEX (manual) - $code\n";
                        push @manual_cases, {
                            file => $file,
                            line => $line,
                            code => $code,
                            color => $color_pattern,
                        };
                        $stats{manual_needed}++;
                    }
                }
            }
        }
    }
}

print "\n";

# Phase 2: Auto-refactor simple cases using metavariable patterns
print "Phase 2: Auto-refactor (simple cases only)\n";
print "-" x 50 . "\n";

# Process in order: most specific first
my @ordered_patterns = qw(bold().cyan bold().green cyan green yellow red bright_black bold);

for my $color_pattern (@ordered_patterns) {
    my $config = $colors{$color_pattern};
    my $simple_pattern = $config->{simple};
    my $rewrite = $config->{rewrite};

    print "Pattern: $simple_pattern → $rewrite\n";

    for my $file (@files) {
        next unless -f $file;

        my $matches = `hegel astq -l rust -p '$simple_pattern' '$file' 2>/dev/null | wc -l`;
        chomp $matches;
        $matches =~ s/^\s+//;

        if ($matches > 0) {
            print "  $file: $matches simple case(s)\n";
            $stats{auto_fixed} += $matches;

            unless ($dry_run) {
                system("cp '$file' '$file.bak'");
                system("hegel astq -l rust -p '$simple_pattern' -r '$rewrite' -U '$file'");
                print "    ✓ Applied\n";
            }
        }
    }
}

print "\n";

# Phase 3: Summary and manual work report
print "=" x 50 . "\n";
print "SUMMARY\n";
print "=" x 50 . "\n";
print "Total color calls found:    $stats{total_found}\n";
print "Auto-fixed (simple):        $stats{auto_fixed}\n";
print "Manual work needed:         $stats{manual_needed}\n";
print "\n";

if (@manual_cases) {
    print "MANUAL REFACTORING REQUIRED\n";
    print "-" x 50 . "\n";
    print "These cases are too complex for auto-refactoring:\n\n";

    my %by_file;
    for my $case (@manual_cases) {
        push @{$by_file{$case->{file}}}, $case;
    }

    for my $file (sort keys %by_file) {
        print "$file:\n";
        for my $case (@{$by_file{$file}}) {
            print "  L$case->{line}: $case->{code}\n";
            my $theme_method = $colors{$case->{color}}->{name};
            print "         → Wrap in Theme::$theme_method(...)\n";
        }
        print "\n";
    }

    print "Common patterns:\n";
    print "  format!(...).green()     → Theme::success(format!(...))\n";
    print "  x.to_string().cyan()     → Theme::highlight(x.to_string())\n";
    print "  obj.field.method().red() → Theme::error(obj.field.method())\n";
    print "\n";
}

if ($dry_run) {
    print "Run without --dry-run to apply auto-fixes\n";
} elsif ($stats{auto_fixed} > 0) {
    print "Next steps:\n";
    print "  1. cargo test\n";
    print "  2. Review auto-fixes: git diff\n";
    print "  3. Manually refactor $stats{manual_needed} complex cases above\n";
    print "  4. cargo test again\n";
    print "  5. Commit changes\n";
    print "\n";
    print "Backups saved as *.bak\n";
}
