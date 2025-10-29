# Git Initialization Guide

Final step for init workflows: commit DDD documentation to repository.

---

## Greenfield Mode (`git_init`)

**Context**: New project, may not have git repository yet

**Tasks**:
1. Run `git init` (if not already a git repo)
2. Create `.gitignore` with standard entries:
   - `.hegel/state.json` (local state)
   - `.hegel/hooks.jsonl` (local metrics)
   - `.hegel/states.jsonl` (local history)
   - Language-specific ignores (target/, node_modules/, etc.)
3. Stage CLAUDE.md, VISION.md, ARCHITECTURE.md, .gitignore
4. Create initial commit: "chore: initialize DDD project structure"

**After completing**, inform user:
"Greenfield project initialized. Next steps:
- Review CLAUDE.md, VISION.md, ARCHITECTURE.md
- Run 'hegel start discovery' to begin first feature exploration"

---

## Retrofit Mode (`git_commit`)

**Context**: Existing project with git repository

**Git workflow considerations** (based on earlier detection):
- If new branch was created: commit there
- If on main/master: user approved direct commit

**Tasks**:
1. Create `.gitignore` entries for Hegel state (if not exists):
   - `.hegel/state.json`
   - `.hegel/hooks.jsonl`
   - `.hegel/states.jsonl`
2. Stage CLAUDE.md, VISION.md, ARCHITECTURE.md, CODE_MAP.md
3. Stage .gitignore updates (if any)
4. Create commit: "docs: add DDD methodology documentation"

**After committing**, inform user:
"DDD retrofit complete. Next steps:
- Review CLAUDE.md, VISION.md, ARCHITECTURE.md, CODE_MAP.md
- Run 'hegel start discovery' to explore first refactoring or feature
- If on feature branch: consider creating PR for team review"
