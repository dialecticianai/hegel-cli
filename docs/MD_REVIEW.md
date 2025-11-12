# Markdown Document Review

Hegel provides two complementary tools for reviewing Markdown documents:

- **`hegel reflect`** - Interactive GUI for adding review comments
- **`hegel review`** - CLI for reading/writing reviews, with polling support for IDE integration

---

## `hegel reflect` - GUI Review Tool

Launch ephemeral GUI for reviewing Markdown artifacts:

```bash
# Single file review
hegel reflect SPEC.md

# Multiple files
hegel reflect SPEC.md PLAN.md

# With output directory
hegel reflect SPEC.md --out-dir .reviews/

# Headless mode (testing)
hegel reflect SPEC.md --headless
```

**Powered by [mirror](https://github.com/dialecticianai/hegel-mirror)**, a zero-friction Markdown review UI. Requires `mirror` binary built and available (adjacent repo or in PATH).

**Review workflow:**
- Select text → comment → submit
- Comments saved to `.ddd/<filename>.review.N`
- Auto-exit on submit (like `git commit`)
- Session ID passthrough via `HEGEL_SESSION_ID`

---

## `hegel review` - CLI Review Tool

Read and write reviews for files stored in `.hegel/reviews.json`:

```bash
# Write reviews from JSONL (one ReviewComment per line)
echo '{"timestamp":"...","file":"...","selection":{...},"text":"...","comment":"..."}' | hegel review path/to/file.md

# Poll for new reviews (default - blocks until reviews appear)
hegel review file1.md file2.md

# Read existing reviews immediately (legacy behavior)
hegel review --immediate path/to/file.md

# Extension is optional (.md auto-appended if needed)
hegel review SPEC     # Works for SPEC.md
```

### Write Mode

When stdin is present, parses JSONL input as ReviewComment objects:

- Parses JSONL input as ReviewComment objects
- Appends to existing reviews in `.hegel/reviews.json`
- Outputs success JSON: `{"file":"relative/path","comments":N}`

### Polling Mode (Default)

When no stdin, polls for new reviews (designed for IDE integration):

- Polls `.hegel/reviews.json` every 200ms for new reviews
- Waits for reviews with timestamp > command start time
- Supports multiple files (waits until all have reviews)
- Outputs reviews as JSONL when found, exits 0
- Timeout after 5 minutes, exits 1

### Immediate Mode

Use `--immediate` (or `--no-wait`) to return existing reviews immediately:

- Returns existing reviews immediately
- Outputs all reviews for file as JSONL
- Empty output if no reviews exist
- Flattens comments across all review sessions

### Path Handling

- Accepts absolute or relative paths
- Optional `.md` extension (tries both variants)
- Clear error if file not found

---

## Integration with Hegel IDE

When `HEGEL_IDE_URL` environment variable is set, `hegel reflect` sends HTTP POST requests to the IDE instead of launching the mirror binary. The IDE then writes reviews to `.hegel/reviews.json`, and `hegel review` (in polling mode) detects and returns them.

This enables seamless integration between command-line workflows and GUI-based review processes.
