## Structure for Larger Projects (>50 files)

For larger codebases, use **one README.md per directory** with embedded code map.

### Format

Use a tree structure (like `tree` command output) with inline descriptions:

```markdown
# src/

Brief description of this directory's purpose.

## Structure

```
src/
├── lib.rs              Library root exposing core modules
├── main.rs             Binary entry point
├── module1.rs          Brief description (1 sentence)
├── module2.rs          Brief description (1 sentence)
│
├── commands/           CLI command implementations
│   └── See commands/README.md
│
├── metrics/            Metrics parsing and analysis
│   └── See metrics/README.md
│
└── test_helpers.rs     Shared test utilities
```

## Key Patterns

**Pattern name**: Brief explanation of architectural patterns used
```

### Hierarchical Principles

1. **Non-recursive**: Each README describes only its direct children
2. **Scoped**: One directory's README = one file's responsibility
3. **Linked**: Reference subdirectory READMEs explicitly (see X/README.md)
4. **Visual tree**: Use box-drawing characters (├──, └──, │) for structure
5. **Concise**: 1 sentence per file, use alignment for readability

### When to Use Hierarchical Mode

- Total files >50
- Deep directory nesting (>2 levels)
- Multi-language or multi-module projects
- Clear separation of concerns across directories

### Maintenance

Update README.md in a directory whenever you:
- Add/remove/rename files in that directory
- Change file responsibilities
- Reorganize subdirectories
