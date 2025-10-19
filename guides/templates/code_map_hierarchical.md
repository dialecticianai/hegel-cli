## Structure for Larger Projects (>50 files)

For larger codebases, use **one CODE_MAP.md per directory** for maintainability.

### Format

```markdown
# src/CODE_MAP.md

Brief description of this directory's purpose.

## Core Functionality

### **module1.ext**
Brief description (1-3 sentences).

### **module2.ext**
Brief description (1-3 sentences).

## Subdirectories

### **commands/**
Brief description of subdirectory purpose. See commands/CODE_MAP.md.

### **metrics/**
Brief description of subdirectory purpose. See metrics/CODE_MAP.md.
```

### Hierarchical Principles

1. **Non-recursive**: Each CODE_MAP describes only its direct children
2. **Scoped**: One directory's CODE_MAP = one file's responsibility
3. **Linked**: Reference subdirectory CODE_MAPs explicitly
4. **Grouped**: Organize files by logical functionality, not alphabetically

### When to Use Hierarchical Structure

- Total files >50
- Deep directory nesting (>2 levels)
- Multi-language or multi-module projects
- Clear separation of concerns across directories

### Maintenance

Update CODE_MAP.md in a directory whenever you:
- Add/remove/rename files in that directory
- Change file responsibilities
- Reorganize subdirectories
