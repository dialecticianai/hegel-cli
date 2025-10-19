## Structure for Small Projects (<50 files)

For smaller codebases, a **single root-level CODE_MAP.md** often suffices.

### Format

```markdown
# CODE_MAP.md

One-line project description.

## Source Code

### **file1.ext**
Brief description (1-3 sentences).

### **file2.ext**
Brief description (1-3 sentences).

## Tests

### **test_file.ext**
Brief description of what's tested.

## Configuration

### **.config_file**
Brief description of configuration purpose.
```

### When to Use Monolithic Structure

- Total files <50
- Flat or shallow directory structure
- Single-language projects
- Quick orientation is priority over hierarchical organization

### When to Switch to Hierarchical

If your monolithic CODE_MAP exceeds ~100 lines or you have >3 subdirectories with distinct purposes, switch to hierarchical structure (one CODE_MAP per directory).
