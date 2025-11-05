# CODE_MAP_WRITING.md — Guide for Writing Code Maps in READMEs

This guide explains how to create and maintain code maps within README files throughout the codebase.

---

## Purpose

A **code map** is a directory-level index that provides quick orientation:
- What files exist in this directory
- What each file does (1-3 sentences)
- How subdirectories are organized

**Key Properties**:
- Brief descriptions only (not tutorials)
- Single directory scope (not recursive)
- Lives inside README files (monolithic or hierarchical)
- Updated before structural changes

---

## When to Create

Code maps help navigate codebases. The structure depends on project size and configuration:

**Monolithic mode (<50 files)**: Code map section in root README.md
**Hierarchical mode (>50 files)**: Separate README.md per directory with code map

**Depth Guidelines (Hierarchical Mode)**:

Use judgment to determine how deep to nest README.md files:

- **<10 files in subdirectory**: Document inline in parent's code map (no separate README)
- **10+ files in subdirectory**: Consider creating separate README.md
- **Complex subsystems**: Create separate README even if <10 files (e.g., contains tests, multiple concerns)
- **Maximum depth**: Generally stop at 2-3 levels deep; beyond that, document in parent README

**Example**: If `src/commands/` has 15 files plus 3 subdirectories with 5-7 files each, you might document the small subdirectories inline rather than creating 3 separate READMEs. But if one subdirectory has 10+ files or is conceptually complex, give it its own README.

Update code maps **before** commits that:
- Add, remove, or rename files
- Change file responsibilities
- Reorganize directory structure

---

## Your Structure

{{templates/code_map_{{code_map_style}}}}

---

## Core Principles

### 1. Single Directory Scope
Describe only files and folders in the current directory. Do not describe subdirectory contents.

### 2. Non-Recursive
Subdirectories get brief descriptions with references to their own README.md files.

### 3. Concise Descriptions
Each entry: 1-3 sentences maximum. State purpose and key responsibilities.

### 4. Logical Grouping
Group related files under section headers that reflect their role in the codebase.

---

## Basic Structure

Code maps use **tree format** (like the `tree` command) with inline descriptions.

**Monolithic (code map section in root README.md):**
```markdown
# Project Name

Brief project description.

## Code Structure

```
src/
├── main.rs             Entry point for CLI application
├── config.rs           Configuration loading and validation
├── commands/
│   ├── init.rs         Initialize new workflows
│   └── start.rs        Start workflow execution
└── test_helpers.rs     Shared test utilities
```
```

**Hierarchical (directory README.md files):**
```markdown
# src/commands/

CLI command implementations for workflow management.

## Structure

```
commands/
├── mod.rs              Module exports and command registry
├── init.rs             Initialize workflows (greenfield/retrofit)
├── start.rs            Start workflow execution
├── next.rs             Advance to next workflow phase
└── status.rs           Display current workflow state
```
```

**Key formatting rules:**
- Use box-drawing characters: `├──`, `└──`, `│`
- Align descriptions at consistent column (usually after filename)
- Keep descriptions to 1 sentence per file
- Use blank lines with `│` to separate logical groups
- Reference subdirectories with "See subdir/README.md"

---

## Style Guidelines

- **Be direct**: State facts, not meta-commentary
- **Be brief**: 1-3 sentences per entry
- **Be specific**: Mention key operations or interfaces when helpful
- **Use active voice**: "Processes data" not "Data is processed"
- **Avoid obviousness**: Don't explain file extensions or state the obvious

---

## What Not to Do

**Don't over-detail**: Avoid implementation specifics
**Don't recurse**: Don't describe subdirectory contents
**Don't under-describe**: Give enough context to understand purpose
**Don't use project-specific jargon**: Keep descriptions accessible

---

{{templates/mirror_workflow}}

---

## Update Workflow

1. Make structural changes to codebase
2. Update affected README files with code map changes
3. Commit changes and README updates together

---

## Conclusion

Code maps are living documentation that stay synchronized with code structure. They require judgment to organize logically and describe concisely. Keep them current to maintain their value as navigation aids.
