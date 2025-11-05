#!/usr/bin/env python3
"""Count commits per day from git history"""

import argparse
import subprocess
from collections import Counter

def get_commits_per_day():
    """Get commit counts grouped by day"""
    result = subprocess.run(
        ['git', 'log', '--date=short', '--pretty=format:%ad'],
        capture_output=True,
        text=True,
        check=True
    )

    dates = result.stdout.strip().split('\n')
    return Counter(dates)

def print_bars(sorted_days):
    """Print ASCII bar graph with up to 10 X's per day"""
    if not sorted_days:
        return

    max_count = max(count for _, count in sorted_days)

    # Print bars vertically (10 rows tall)
    for row in range(10, 0, -1):
        threshold = (row / 10) * max_count
        line = ""
        for _, count in sorted_days:
            if count >= threshold:
                line += "X "
            else:
                line += "  "
        print(line)

    # Print separator and dates
    width = len(sorted_days) * 2
    print("─" * width)

    # Print first and last date aligned to ends
    first_date = sorted_days[0][0]
    last_date = sorted_days[-1][0]
    spacing = " " * (width - len(first_date) - len(last_date))
    print(f"{first_date}{spacing}{last_date}")

def main():
    parser = argparse.ArgumentParser(description='Count commits per day from git history')
    parser.add_argument('--bars', action='store_true', help='Show ASCII bar graph')
    args = parser.parse_args()

    commits_per_day = get_commits_per_day()

    # Sort by date
    sorted_days = sorted(commits_per_day.items())

    if args.bars:
        print_bars(sorted_days)
        print()
    else:
        # Print counts
        for date, count in sorted_days:
            print(f"  {count:3d} {date}")
        print()

    # Summary
    total_commits = sum(commits_per_day.values())
    total_days = len(commits_per_day)
    avg = total_commits / total_days if total_days > 0 else 0

    print("Summary:")
    print("--------")
    print(f"Total commits: {total_commits}")
    print(f"Active days: {total_days}")
    print(f"Average commits/day: {avg:.2f}")

if __name__ == '__main__':
    main()
