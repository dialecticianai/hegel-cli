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
    """Print ASCII bar graph with up to 10 X's per day, wrapping on month boundaries"""
    if not sorted_days:
        return

    # Get first commit day to determine wrap boundaries
    first_date = sorted_days[0][0]
    first_day = int(first_date.split('-')[2])

    # Split into month chunks (wrapping on the anniversary day)
    chunks = []
    current_chunk = []

    for date, count in sorted_days:
        day = int(date.split('-')[2])

        # Start new chunk if we've reached the wrap day and we have data
        if current_chunk and day == first_day:
            chunks.append(current_chunk)
            current_chunk = []

        current_chunk.append((date, count))

    # Add final chunk
    if current_chunk:
        chunks.append(current_chunk)

    # Print each chunk
    for i, chunk in enumerate(chunks):
        if i > 0:
            print()  # Blank line between chunks

        max_count = max(count for _, count in chunk)

        # Print bars vertically (10 rows tall)
        for row in range(10, 0, -1):
            threshold = (row / 10) * max_count
            line = ""
            for _, count in chunk:
                if count >= threshold:
                    line += "X "
                else:
                    line += "  "
            print(line)

        # Print separator and dates
        width = len(chunk) * 2
        print("─" * width)

        # Print first and last date aligned to ends
        chunk_first = chunk[0][0]
        chunk_last = chunk[-1][0]
        spacing = " " * (width - len(chunk_first) - len(chunk_last))
        print(f"{chunk_first}{spacing}{chunk_last}")

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
