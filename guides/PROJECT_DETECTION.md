# Project Detection Guide for Retrofit Mode

Guide for analyzing existing projects before applying Dialectic-Driven Development methodology.

---

## When to Use This

**Mode**: Retrofit initialization only
**Phase**: First step of `hegel init-retrofit`
**Purpose**: Understand existing project structure before adding DDD documentation

---

## Purpose

Before retrofitting DDD to an existing project, you need to understand:
- What's already in place (git, docs, tests, structure)
- What conventions exist and should be preserved
- What risks exist in modifying the project

This phase gathers context to inform all subsequent initialization steps.

---

## Detection Tasks

Systematically analyze the project:

### 1. Git Repository Status
**Check**: `.git/` directory exists
**Gather**:
- Current branch name
- Working tree status (clean, uncommitted changes, etc.)
- Recent commit history (last 3-5 commits)

### 2. Primary Language Detection
**Check**: Language-specific manifest files
**Common patterns**:
- Rust: `Cargo.toml`
- JavaScript/TypeScript: `package.json`
- Python: `pyproject.toml`, `setup.py`, `requirements.txt`
- Go: `go.mod`
- Java: `pom.xml`, `build.gradle`

**Gather**:
- Language name
- Version specification (if available)
- Key dependencies

### 3. Existing Documentation
**Check**: Documentation files and directories
**Common locations**:
- Root: `README.md`, `CONTRIBUTING.md`, `ARCHITECTURE.md`
- Directories: `docs/`, `doc/`, `documentation/`

**Gather**:
- List of key documentation files
- Whether vision/goals/architecture are already documented

### 4. Test Infrastructure
**Check**: Test framework and patterns
**Common patterns**:
- Test directories: `tests/`, `test/`, `spec/`
- Inline tests: `src/**/*_test.rs`, `src/**/*.test.js`
- Test runners: `pytest`, `jest`, `cargo test`, `go test`

**Gather**:
- Test framework in use
- Test organization pattern (inline vs separate directory)
- Coverage tooling (if detectable)

### 5. Project Structure
**Check**: Key directories and organization
**Common patterns**:
- Source code: `src/`, `lib/`, `app/`, `pkg/`
- Configuration: `config/`, `.config/`, root config files
- Scripts: `scripts/`, `bin/`
- Build artifacts: `target/`, `dist/`, `build/`, `out/`

**Gather**:
- Primary source directory
- Configuration approach
- Build/script tooling

---

## Reporting Findings

Present structured analysis to user:

```markdown
## Project Detection Results

**Git Repository**: ✓ Exists (branch: main, clean working tree)

**Primary Language**: Rust 1.75+
- Detected via: Cargo.toml
- Key dependencies: serde, anyhow, clap

**Existing Documentation**:
- README.md (project overview)
- docs/architecture.md (technical design)
- No VISION.md or CLAUDE.md found

**Test Infrastructure**:
- Framework: cargo test (built-in)
- Pattern: Inline tests (#[cfg(test)] modules)
- Coverage: No tooling detected

**Project Structure**:
```
src/           - Main source code
tests/         - Integration tests
scripts/       - Build and utility scripts
target/        - Build artifacts (gitignored)
```
```

---

## User Consultation

After presenting findings, ask focused questions:

### 1. Branching Strategy
```
"I've detected an existing Rust project on branch 'main'. Before proceeding:

Should I create a new branch for this retrofit? (recommended)
- Suggested name: 'feat/add-ddd-methodology' or 'docs/ddd-retrofit'
- Allows PR review before merging to main
- Easy rollback if needed

Or should I proceed on current branch?"
```

### 2. Convention Preservation
```
"I've analyzed the existing project structure. Key questions:

1. Should I preserve existing documentation conventions?
   - You have docs/architecture.md. Integrate with ARCHITECTURE.md or keep separate?

2. Should I preserve existing test patterns?
   - Currently using inline #[cfg(test)] modules. Maintain this pattern?

3. Are there code organization rules I should respect?
   - File size limits, module structure, naming conventions?
```

### 3. Modification Boundaries
```
"Before I proceed:

Are there any files or directories I should avoid modifying?
- Configuration files that are environment-specific?
- Generated code directories?
- Vendor dependencies?
- Files with team-wide conventions I shouldn't alter?
```

---

## Adversarial Check

Before completing detection, ask yourself:

1. **Unusual project structures**: Am I missing non-standard layouts (monorepos, polyglot projects)?
2. **Hidden conventions**: Are there `.editorconfig`, `.prettierrc`, or similar convention files?
3. **CI/CD integration**: Will adding DDD docs break existing pipelines?
4. **Multi-language projects**: Did I detect secondary languages correctly?
5. **Build complexity**: Are there complex build scripts I should understand first?

---

## Output

Complete this phase by:

1. **Present structured findings** (git, language, docs, tests, structure)
2. **Ask focused questions** (branching, conventions, boundaries)
3. **Await user confirmation** before proceeding to code_map phase
4. **Create branch** if user requested it

---

## Common Pitfalls

**Assuming standard layouts**: Not all projects follow conventions → Check multiple patterns

**Missing existing docs**: Don't just check root → Look in docs/, doc/, documentation/

**Ignoring build complexity**: Complex build setups may have special requirements → Review Makefiles, build scripts

**Not asking about conventions**: Every project has unwritten rules → Explicit questions surface them

---

## Conclusion

Project detection establishes the foundation for successful DDD retrofit. Thorough analysis and explicit user consultation prevent conflicts with existing conventions and ensure smooth integration of DDD methodology into established projects.
