# CODE_MAP_WRITING.md — Guide for Writing CODE_MAP.md Files

This guide explains how to create and maintain CODE_MAP.md files throughout the codebase.

---

## Purpose

A **CODE_MAP.md** is a directory-level index that provides quick orientation:
- What files exist in this directory
- What each file does (1-3 sentences)
- How subdirectories are organized

**Key Properties**:
- Brief descriptions only (not tutorials)
- Single directory scope (not recursive)
- Updated before structural changes

---

## When to Create

CODE_MAP.md files help navigate codebases. The structure depends on project size:

**Small projects (<50 files)**: Single root-level CODE_MAP.md
**Larger projects (>50 files)**: One CODE_MAP.md per directory

Update CODE_MAP.md **before** commits that:
- Add, remove, or rename files
- Change file responsibilities
- Reorganize directory structure

---

## Choosing Your Structure

### Monolithic (Small Projects)

{{templates/code_map_monolithic}}

### Hierarchical (Larger Projects)

{{templates/code_map_hierarchical}}

---

## Core Principles

### 1. Single Directory Scope
Describe only files and folders in the current directory. Do not describe subdirectory contents.

### 2. Non-Recursive
Subdirectories get brief descriptions with references to their own CODE_MAP.md files.

### 3. Concise Descriptions
Each entry: 1-3 sentences maximum. State purpose and key responsibilities.

### 4. Logical Grouping
Group related files under section headers that reflect their role in the codebase.

---

## Basic Structure

```markdown
# directory_name/CODE_MAP.md

One-line description of directory purpose.

## Section Name

### **filename**
Brief description of what this file does and why it exists.

### **subdirectory/**
Brief description of subdirectory purpose. See subdirectory/CODE_MAP.md.
```

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
2. Update affected CODE_MAP.md files
3. Commit changes and CODE_MAP updates together

---

## Conclusion

CODE_MAP.md files are living documentation that stay synchronized with code structure. They require judgment to organize logically and describe concisely. Keep them current to maintain their value as navigation aids.
