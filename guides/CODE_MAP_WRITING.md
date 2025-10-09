# CODE_MAP_WRITING.md — Guide for Writing CODE_MAP.md Files

This guide explains how to create and maintain CODE_MAP.md files throughout the codebase.

---

## When to Use This

**Required Locations**: Every directory containing `.py` files must have a CODE_MAP.md

**Common Locations**:
- `./CODE_MAP.md` — Root-level files and folders
- `scripts/CODE_MAP.md` — Utility scripts
- `src/docdd_mcp/CODE_MAP.md` — Production modules
- `tests/CODE_MAP.md` — Test suite organization
- `tests/unit/CODE_MAP.md` — Unit test files
- `tests/integration/CODE_MAP.md` — Integration test files
- `toys/toyN_name/CODE_MAP.md` — Toy implementation files (if applicable)

**Update Trigger**: **CRITICAL** — Update CODE_MAP.md **before any structural commit** that:
- Adds new files or modules
- Removes files or modules
- Renames files or modules
- Changes module purpose or responsibilities

---

## Purpose

A **CODE_MAP.md** is a concise, directory-level index that helps developers and AI assistants quickly understand:
- What files exist in this directory
- What each file does (1-3 sentences)
- How files relate to each other

**Not a tutorial** — brief descriptions only.
**Not recursive** — describes only the current directory's contents, not subdirectories.

---

## Core Principles

### 1. Single Directory Scope
Each CODE_MAP.md describes **only files and folders in its own directory**, not subdirectories.

**Example**: `./CODE_MAP.md` describes root files like `CLAUDE.md`, `pyproject.toml`, and folders like `src/`, `tests/`. It does NOT describe files inside `src/` or `tests/`.

### 2. Non-Recursive
Subdirectories are mentioned with a reference to their own CODE_MAP.md:
```markdown
### **src/**
Production source code. Contains docdd_mcp/ package. See src/docdd_mcp/CODE_MAP.md.
```

### 3. Concise Descriptions
Each file gets 1-3 sentences maximum:
- What it is
- What it does
- Why it matters (if not obvious)

### 4. Logical Grouping
Group related files under section headers:
```markdown
## Configuration & Build
### **pyproject.toml**
...

## Documentation
### **CLAUDE.md**
...
```

---

## Structure Template

```markdown
# [directory]/CODE_MAP.md

[One-line description of this directory's purpose]

## [Logical Group 1]

### **filename.py**
[1-3 sentence description. What it does, key responsibilities, core API if relevant.]

### **another_file.py**
[1-3 sentence description.]

## [Logical Group 2]

### **subdirectory/**
[1-2 sentence description. Reference subdirectory's CODE_MAP.md if it has one.]

### **another_subdir/**
[1-2 sentence description. See another_subdir/CODE_MAP.md.]
```

---

## Example Patterns

### Python Modules
```markdown
### **engine.py**
YAML workflow state machine engine. Provides functional API for workflow orchestration:
- `load_workflow()`: Parse YAML workflow definitions
- `init_state()`: Initialize workflow state from definition
- `get_next_prompt()`: Evaluate claims and transition between workflow nodes
```

### Test Files
```markdown
### **test_template_parsing.py**
Placeholder extraction tests. Validates `parse_template()` correctly identifies required (`{{name}}`) and optional (`{{? name}}`) placeholders.
```

### Configuration Files
```markdown
### **pyproject.toml**
Python project configuration. Defines package metadata, dependencies (fastmcp, pytest, pyyaml), and build system. Used by pip for installation.
```

### Subdirectories
```markdown
### **workflows/**
YAML workflow definitions for DocDD orchestration:
- `discovery.yaml`: Discovery mode workflow (experimental iteration)
- `execution.yaml`: Execution mode workflow (production features)
- `minimal.yaml`: Minimal workflow for testing
```

---

## Common Section Headers

Use these headers for logical grouping (adapt as needed):

**For root directories**:
- Configuration & Build
- Documentation
- Scripts
- Source Code
- Tests
- Reference Implementations
- Project Artifacts

**For source directories**:
- Core Modules
- Utilities
- Workflow Definitions
- Configuration
- Exceptions/Types

**For test directories**:
- Test Files by Module
- Fixtures
- Test Utilities
- Test Naming Conventions
- Running Tests
- Test Strategy

---

## Quality Checklist

Before committing a CODE_MAP.md update:

- [ ] Every `.py` file in this directory is documented
- [ ] Every subdirectory is mentioned (with reference to its CODE_MAP.md if it exists)
- [ ] Descriptions are concise (1-3 sentences per item)
- [ ] No implementation details (just purpose and key API if relevant)
- [ ] Logical grouping with clear section headers
- [ ] No recursive content (subdirectory details belong in their own CODE_MAP.md)
- [ ] Updated **before** structural commit (not after)

---

## Update Workflow

**When adding a new file**:
1. Implement and test the file
2. Update this directory's CODE_MAP.md with new entry
3. Commit both together

**When renaming/removing a file**:
1. Perform rename/removal
2. Update CODE_MAP.md to reflect change
3. Commit both together

**When changing module purpose**:
1. Implement changes
2. Update CODE_MAP.md description
3. Commit both together

---

## Style Guide

**Be direct**: "Provides X" not "This module provides X"
**Be specific**: Mention key functions/classes by name when helpful
**Be brief**: 1-3 sentences maximum per item
**Use active voice**: "Validates inputs" not "Inputs are validated"
**Skip obvious info**: Don't explain what `.py` means or that tests test things

---

## Anti-Patterns

❌ **Over-detail**: Don't document internal implementation
```markdown
### **engine.py**
Contains the load_workflow function which opens a YAML file, parses it with PyYAML,
validates the schema with custom validation logic that checks for required fields...
```

✅ **Right level**: Focus on purpose and key API
```markdown
### **engine.py**
YAML workflow state machine engine. Provides `load_workflow()`, `init_state()`,
`get_next_prompt()` for claims-driven orchestration.
```

❌ **Recursive content**: Don't describe subdirectory contents
```markdown
### **src/**
Contains server.py with the MCP server, engine.py with workflow logic,
template.py with template parsing...
```

✅ **Subdirectory reference**: Point to subdirectory's CODE_MAP.md
```markdown
### **src/**
Production source code. Contains docdd_mcp/ package with MCP server implementation.
See src/docdd_mcp/CODE_MAP.md.
```

❌ **Too terse**: Don't skip essential context
```markdown
### **server.py**
The server.
```

✅ **Informative brevity**: Essential info in 1-3 sentences
```markdown
### **server.py**
Main MCP server definition. Creates FastMCP instance with authentication, mounts
middleware, and exposes workflow orchestration tools.
```

---

## Conclusion

CODE_MAP.md files are **living documentation** that must stay synchronized with the codebase. They're not generated artifacts — they require human judgment to group files logically and describe purposes concisely. Update them **before every structural commit** to maintain their value as navigational aids.
