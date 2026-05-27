#!/usr/bin/env bash
# Diagnose archive phase corruption from the pre-Nov-3 re-archiving bug.
# Reports:
#   1. Cross-archive duplicate phases (same phase_name + start_time in >1 archive file)
#   2. Time-overlapping archives whose DISTINCT phases interleave (forces global sort)
# Read-only. No writes.
set -euo pipefail

ARCHIVE_DIR="${1:-.hegel/archive}"

if [[ ! -d "$ARCHIVE_DIR" ]]; then
  echo "No archive dir: $ARCHIVE_DIR" >&2
  exit 1
fi

echo "== Per-phase (phase_name@start_time) occurrence across archive files =="
# Emit "phase_name|start_time<TAB>archive_file" for every phase, then group.
for f in "$ARCHIVE_DIR"/*.json; do
  base=$(basename "$f")
  jq -r --arg f "$base" '.phases[]? | "\(.phase_name)|\(.start_time)\t\($f)"' "$f"
done | sort | awk -F'\t' '
  { key=$1; file=$2; if (key==prev) { files[key]=files[key]","file; cnt[key]++ } else { files[key]=file; cnt[key]=1 } prev=key }
  END { for (k in cnt) if (cnt[k]>1) printf "DUP x%d  %s\n          in: %s\n", cnt[k], k, files[k] }
' | sort

echo
echo "== Cross-archive duplicate summary =="
total_phase_rows=0
for f in "$ARCHIVE_DIR"/*.json; do
  n=$(jq '.phases | length' "$f")
  total_phase_rows=$((total_phase_rows + n))
done
distinct=$(for f in "$ARCHIVE_DIR"/*.json; do
  jq -r '.phases[]? | "\(.phase_name)|\(.start_time)"' "$f"
done | sort -u | wc -l | tr -d ' ')
echo "total phase rows across all archives: $total_phase_rows"
echo "distinct (phase_name,start_time):     $distinct"
echo "redundant rows (rows - distinct):     $((total_phase_rows - distinct))"

echo
echo "== Archive time spans (workflow_id  ->  [min_start .. max_end])  sorted by workflow_id =="
for f in "$ARCHIVE_DIR"/*.json; do
  jq -r '"\(.workflow_id)\t\([.phases[].start_time] | min // "-")\t\([.phases[].end_time // .phases[].start_time] | max // "-")\t\(.phases | length)"' "$f"
done | sort | awk -F'\t' '{ printf "%-40s  %-32s .. %-32s  (%s phases)\n", $1, $2, $3, $4 }'
