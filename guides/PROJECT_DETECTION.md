# Project Detection Guide (Retrofit Mode)

Analyze existing project before retrofitting Dialectic-Driven Development.

---

## Tasks

1. Check if git repository exists (`.git/` directory)
2. Identify primary language (check for `Cargo.toml`, `package.json`, `pyproject.toml`, etc.)
3. Look for existing documentation (`README.md`, `docs/`, etc.)
4. Check for existing test infrastructure
5. Identify project structure patterns

## Report Findings

- Git status: [exists/missing, current branch if exists]
- Primary language: [detected language + version if available]
- Existing docs: [list key files]
- Test setup: [detected test framework/patterns]
- Project structure: [key directories and organization]

## User Questions

After presenting findings, ask:

"I've detected an existing [language] project. Before proceeding:
1. Should I create a new branch for this retrofit? (recommended)
2. Are there any existing conventions I should preserve?
3. Any files or directories I should avoid modifying?"

Respond with analysis and await user guidance.
