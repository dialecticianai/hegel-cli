# Git Initialization Guide for Init Workflows

Guide for git repository setup and initial DDD documentation commits.

---

## When to Use This

**Modes**: Both greenfield and retrofit initialization
**Phase**: Final step of init workflows
**Purpose**: Create git commits for DDD documentation with proper structure

---

## Purpose

This phase finalizes initialization by:
- Ensuring git repository exists and is properly configured
- Creating `.gitignore` entries for Hegel local state
- Committing DDD documentation to version control
- Setting up for first workflow execution

---

## Mode-Specific Workflows

### Greenfield Mode: `git_init` Node

**Context**: New project, may not have git repository yet

**Tasks**:

1. **Check for existing git repository**
   ```bash
   if [ ! -d .git ]; then
     git init
   fi
   ```

2. **Create/update `.gitignore`**

   Add Hegel-specific entries (if not already present):
   ```
   # Hegel local state (not shared across machines)
   .hegel/state.json
   .hegel/hooks.jsonl
   .hegel/states.jsonl
   ```

   Add language-specific standard ignores:
   - **Rust**: `target/`, `Cargo.lock` (for libraries)
   - **JavaScript**: `node_modules/`, `dist/`
   - **Python**: `__pycache__/`, `*.pyc`, `.venv/`, `venv/`
   - **Go**: `bin/`, `*.exe`

3. **Stage DDD documentation**
   ```bash
   git add CLAUDE.md VISION.md ARCHITECTURE.md .gitignore
   ```

4. **Create initial commit**
   ```bash
   git commit -m "chore: initialize DDD project structure

   - Add CLAUDE.md with operational conventions
   - Add VISION.md defining project goals
   - Add ARCHITECTURE.md with tech stack decisions
   - Configure .gitignore for Hegel local state

   🤖 Generated with [Claude Code](https://claude.com/claude-code)

   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

5. **Inform user of next steps**
   ```
   Greenfield project initialized successfully.

   Next steps:
   - Review CLAUDE.md, VISION.md, ARCHITECTURE.md
   - Run 'hegel start discovery' to begin first feature exploration
   - Consider creating remote repository and pushing initial commit
   ```

---

### Retrofit Mode: `git_commit` Node

**Context**: Existing project with git repository, possibly on feature branch

**Git Workflow Considerations**:

Based on earlier detection phase:
- If new branch was created: commit there
- If on main/master: user approved direct commit
- Respect existing commit message conventions

**Tasks**:

1. **Create/update `.gitignore`**

   Check if entries already exist, add only if missing:
   ```bash
   # Check first
   if ! grep -q ".hegel/state.json" .gitignore 2>/dev/null; then
     # Add Hegel entries
   fi
   ```

   Add to `.gitignore` (if not present):
   ```
   # Hegel local state (not shared across machines)
   .hegel/state.json
   .hegel/hooks.jsonl
   .hegel/states.jsonl
   ```

2. **Stage files**
   ```bash
   git add CLAUDE.md VISION.md ARCHITECTURE.md CODE_MAP.md

   # Stage .gitignore only if modified
   if git diff --name-only .gitignore 2>/dev/null | grep -q .; then
     git add .gitignore
   fi
   ```

3. **Create retrofit commit**

   Use conventional commit format (adapt to project's existing convention if detected):
   ```bash
   git commit -m "docs: add DDD methodology documentation

   - Add CLAUDE.md integrating with existing conventions
   - Add VISION.md formalizing project goals
   - Add ARCHITECTURE.md documenting current tech stack
   - Add CODE_MAP.md for codebase navigation
   - Update .gitignore for Hegel local state

   🤖 Generated with [Claude Code](https://claude.com/claude-code)

   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

4. **Inform user of next steps**

   Branch-aware messaging:
   ```
   DDD retrofit complete.

   Next steps:
   - Review CLAUDE.md, VISION.md, ARCHITECTURE.md, CODE_MAP.md
   - Run 'hegel start discovery' to explore first refactoring or feature
   [IF ON FEATURE BRANCH]
   - Consider creating PR for team review: 'gh pr create' or push and open PR manually
   [IF ON MAIN]
   - Consider pushing to remote: 'git push origin main'
   ```

---

## .gitignore Strategy

### Why Ignore Hegel State Files?

**`.hegel/state.json`**: Current workflow position (machine-local, not shared)
**`.hegel/hooks.jsonl`**: Claude Code hook events (personal session data)
**`.hegel/states.jsonl`**: Workflow transition history (personal session data)

These files are **ephemeral session state**, not project artifacts:
- Different developers may be at different workflow positions
- Hook events contain personal usage patterns
- State history reflects individual work sessions

### What NOT to Ignore

**Workflows** (`workflows/*.yaml`): Version-controlled, shared across team
**Guides** (`guides/*.md`): Version-controlled, shared across team
**DDD Docs** (`CLAUDE.md`, `VISION.md`, etc.): Version-controlled, foundational

---

## Commit Message Guidelines

### Conventional Commit Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**For init commits**:
- **Type**: `chore` (greenfield) or `docs` (retrofit)
- **Scope**: Optional, omit for init
- **Subject**: Brief description
- **Body**: Bullet list of changes
- **Footer**: Claude Code attribution

### Subject Lines

**Greenfield**: `chore: initialize DDD project structure`
**Retrofit**: `docs: add DDD methodology documentation`

Use present tense, imperative mood ("add" not "added" or "adds")

### Body Format

Bullet list with clear, scannable items:
```
- Add CLAUDE.md with operational conventions
- Add VISION.md defining project goals
- Add ARCHITECTURE.md with tech stack decisions
- Configure .gitignore for Hegel local state
```

### Footer (Required)

Always include Claude Code attribution:
```
🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## Adversarial Checks

Before committing, verify:

1. **All files staged**: Did we miss any new DDD docs?
2. **No unwanted changes**: Are we accidentally staging other modified files?
3. **Proper .gitignore**: Did we correctly add/merge Hegel entries?
4. **Branch awareness**: Are we committing to the expected branch?
5. **User confirmation**: Did user approve the commit message and approach?

---

## Common Pitfalls

**Overwriting .gitignore**: Don't replace existing file → Append Hegel entries if missing

**Committing state files**: Never commit `.hegel/*.json[l]` → Verify .gitignore first

**Wrong branch**: Check current branch before committing → Use `git branch --show-current`

**Missing attribution**: Always include footer → Required by Hegel conventions

**Assuming git exists**: In greenfield, may need `git init` → Check `.git/` directory first

---

## Edge Cases

### Existing .gitignore with Conflicts

If `.gitignore` already has `.hegel/` entry with different pattern:
- Review existing pattern
- Ask user if override or merge is appropriate
- Document decision

### Uncommitted Changes

If working tree has other uncommitted changes:
- Warn user before staging
- Only stage DDD documentation files
- Remind user to handle other changes separately

### Detached HEAD

If repository is in detached HEAD state:
- Warn user
- Suggest creating branch first
- Don't commit until resolved

---

## Conclusion

Git initialization/commit is the final gate before workflow execution. Proper setup ensures:
- DDD documentation is version-controlled and shared
- Local state remains private and machine-specific
- Clean, conventional commit history
- Team can review and adopt DDD methodology through standard PR process
