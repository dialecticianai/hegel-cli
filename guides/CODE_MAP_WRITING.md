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

## Retrofit Mode: Initial CODE_MAP Creation

When retrofitting DDD to an existing project, CODE_MAP creation is the second initialization step (after project detection).

### Project Size Assessment

**First, count source files**:
```bash
# Exclude tests, configs, generated code
find src/ -type f -name "*.rs" | wc -l  # Rust example
```

**Decision threshold**: <50 files vs >50+ files

### Structure Selection

#### Monolithic Structure (<50 files)

**Approach**: Single `CODE_MAP.md` at project root

**Organization**:
- Group files by category (Source Code, Tests, Configuration, Documentation)
- Brief descriptions (1-3 sentences per file)
- Flat structure, easy to scan

**Example**:
```markdown
# Project CODE_MAP

## Source Code (src/)

### **main.rs**
Entry point for CLI, parses arguments and dispatches commands.

### **engine.rs**
Workflow state machine implementation and transition logic.

## Tests (tests/)

### **integration_test.rs**
End-to-end workflow execution tests.
```

#### Hierarchical Structure (>50 files)

**Approach**: One CODE_MAP.md per significant directory

**Organization**:
- Create `src/CODE_MAP.md` for main source directory
- Create `src/subdirectory/CODE_MAP.md` for each significant subdirectory
- Non-recursive (each CODE_MAP only describes its direct children)
- Reference subdirectory CODE_MAPs explicitly

**Example** (`src/CODE_MAP.md`):
```markdown
# src/ — Main Source Code

## Core Modules

### **main.rs**
CLI entry point and argument parsing.

### **engine.rs**
Workflow state machine and transition logic.

### **commands/**
Command implementations (start, next, reset, etc.). See commands/CODE_MAP.md.

### **storage/**
State persistence and file operations. See storage/CODE_MAP.md.
```

### User Consultation Questions

Before creating CODE_MAPs, ask the user:

1. **Structure confirmation**: "I detected [N] source files. Should I use [monolithic/hierarchical] structure?"
   - If <50: Suggest monolithic
   - If >50: Suggest hierarchical
   - Respect user preference if they have one

2. **Directory scope**: "Which directories should I map?"
   - Common: `src/`, `lib/`, `app/`, `pkg/`
   - User may want to include: `scripts/`, `docs/`, `config/`

3. **Exclusions**: "Any directories to skip?"
   - Generated code directories
   - Vendor dependencies (`vendor/`, `node_modules/`)
   - Build artifacts (`target/`, `dist/`, `build/`)

### Adversarial Check

**Ask yourself before proceeding**:
- What directories or patterns am I missing?
- Any unusual project structure I should understand?
- Are there monorepo or polyglot considerations?
- Did I check for hidden directories (`.github/`, `.config/`)?

### Retrofit Workflow

1. **Count files** → Determine structure
2. **Ask user** → Confirm approach
3. **Create CODE_MAPs** → Following chosen structure
4. **Note for later** → Will refine during architecture phase

---

## Your Structure

{{templates/code_map_{{code_map_style}}}}

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
