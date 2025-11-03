## Structure for Small Projects (<50 files)

For smaller codebases, include the code map as a **section in root README.md**.

### Format

Use a tree structure (like `tree` command output) with inline descriptions:

```markdown
# Project Name

One-line project description.

## Code Structure

```
src/
├── file1.ext              Brief description (1 sentence)
├── file2.ext              Brief description (1 sentence)
├── tests/
│   └── test_file.ext      Brief description of what's tested
└── config.ext             Brief description of configuration purpose
```
```

**Key principles:**
- Use actual tree formatting with box-drawing characters (├──, └──, │)
- Keep descriptions to 1 sentence per file
- Group related files under directories
- Show the structure visually, not in sections

### When to Use Monolithic Mode

- Total files <50
- Flat or shallow directory structure
- Single-language projects
- Quick orientation is priority over hierarchical organization

### When to Switch to Hierarchical

If your code map section exceeds ~100 lines or you have >3 subdirectories with distinct purposes, switch to hierarchical mode (separate README.md per directory).
