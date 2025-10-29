# Meta-Document: How to Customize CLAUDE.md for Your Project

_Guide to tailoring operational conventions from the Hegel template._

---

## When to Use This

**Phase**: Project initialization (first step of greenfield or retrofit)
**Location**: Root of project directory (`CLAUDE.md`)
**Sequence**: First document created, before VISION.md and ARCHITECTURE.md
**Purpose**: Establish project-specific operational conventions building on Hegel's base template

---

## Purpose

A **CLAUDE.md is an operational manual**: it defines how AI assistants should work on this project - conventions, constraints, patterns, and project-specific context.

The Hegel template (`HEGEL_CLAUDE.md`) provides DDD methodology foundations. Your job is to **add project-specific customizations** that make AI assistants immediately productive on your codebase.

---

## What CLAUDE.md Customization Is / Is Not

### ❌ Not
- Rewriting the entire Hegel template from scratch
- Documenting features or product goals (that's VISION.md)
- Defining architecture or tech stack (that's ARCHITECTURE.md)
- Writing a user manual for humans

### ✅ Is
- Project-specific conventions (naming, structure, patterns)
- Testing philosophy and coverage expectations
- Commit message format and git workflow
- Code organization rules (file size limits, module structure)
- Tools and scripts in your project's `scripts/` directory
- Language/framework-specific patterns

---

## The Template Starting Point

`hegel init` copies this project's `CLAUDE.md` as your starting point, renamed to `HEGEL_CLAUDE.md` in the template. It includes:

- DDD workflow philosophy (LEXICON principles)
- HANDOFF.md protocol
- Test infrastructure patterns
- Submodule organization guidance
- Documentation ordering rules

**Your task**: Add project-specific customizations that complement (not replace) this foundation.

---

## Core Customization Areas

### 1. Project Context

Add a brief header section before the Hegel content:

```markdown
# CLAUDE.md

**[Project Name]**: [One-line description]

**Key Context**:
- Language: [Primary language + version]
- Framework: [If applicable]
- Test runner: [e.g., pytest, cargo test, jest]
- Key conventions: [2-3 bullets of critical patterns]

---

# Hegel DDD Methodology

[Hegel template content follows...]
```

### 2. Testing Philosophy

Customize test expectations for your domain:

```markdown
## Testing Philosophy

**Coverage target**: ≥80% lines (enforced by pre-commit)

**Test categories**:
- Unit tests: Core logic, error handling
- Integration tests: API endpoints, database interactions
- Property tests: [If applicable, e.g., quickcheck/proptest]

**What to test**:
- ✅ Public APIs and user-facing behavior
- ✅ Error paths and edge cases
- ✅ Integration points between modules

**What NOT to test**:
- ❌ Private helper functions (test through public API)
- ❌ Third-party library behavior
- ❌ Generated code (mocks, fixtures)

**Test file organization**:
- [Describe your pattern: inline `#[cfg(test)]` vs `tests/` directory]
```

### 3. Code Organization

Define your project's structural rules:

```markdown
## Code Organization

**File size limits**:
- Implementation: ≤200 lines per file
- Tests: ≤300 lines per file
- Exceptions: [List any allowed large files with rationale]

**Module structure**:
- [Describe your directory layout]
- [Naming conventions for modules/packages]
- [Where different concerns live]

**Naming conventions**:
- Files: [snake_case, kebab-case, etc.]
- Functions: [camelCase, snake_case, etc.]
- Types: [PascalCase, etc.]
```

### 4. Git Workflow

Specify your commit and branching conventions:

```markdown
## Git Workflow

**Commit format**: Conventional commits (`type(scope): subject`)

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

**Scopes**: [Project-specific scopes, e.g., `api`, `cli`, `db`, `auth`]

**Branch strategy**:
- `main`: [Describe main branch policy]
- Feature branches: [Naming convention if any]
- [Any other branch conventions]

**Pre-commit hooks**:
- [List what runs automatically: formatting, linting, tests, etc.]
```

### 5. Tools and Scripts

Document project-specific tooling:

```markdown
## Project Tools

**Scripts** (in `scripts/` directory):
- `scripts/test.sh`: [What it does]
- `scripts/build.sh`: [What it does]
- `scripts/deploy.sh`: [What it does]

**Key commands**:
- Build: `[command]`
- Test: `[command]`
- Lint: `[command]`
- Format: `[command]`

**Critical patterns**:
- [Any project-specific patterns AI should know]
- [Common gotchas or footguns]
- [Preferred libraries or approaches]
```

### 6. Language/Framework Specifics

Add domain-specific guidance:

```markdown
## [Language/Framework] Conventions

**[Language-specific section title]**:
- [Key patterns for your stack]
- [Error handling approach]
- [Async/concurrency patterns if applicable]
- [Resource management patterns]
- [Framework-specific idioms]
```

---

## Adversarial Review Questions

Before finalizing your CLAUDE.md customizations, ask yourself:

1. **Onboarding efficiency**: Could a new AI assistant be productive immediately?
2. **Convention completeness**: Have I documented critical patterns?
3. **Footgun prevention**: Are there common mistakes I've warned about?
4. **Tool discoverability**: Do I list key scripts and commands?
5. **Missing context**: What project knowledge am I assuming?
6. **Scope creep**: Am I documenting product features instead of operational conventions?

---

## Common Pitfalls

**Over-specifying**: Don't document every style choice → Focus on critical conventions

**Duplicating other docs**: Don't repeat VISION/ARCHITECTURE content → Stay operational

**Assuming context**: "Follow our standard pattern" → Which pattern? Document it.

**Stale content**: Document conventions as they exist now → Update when they change

**Human-focused**: Writing for human developers → Write for AI assistants

---

## Conversation Flow

When `hegel init` reaches the CLAUDE.md customization step, the agent should:

1. **Present the template**: "I've created CLAUDE.md starting with Hegel's DDD methodology. Now let's customize it for your project."

2. **Ask focused questions**:
   - "What's your testing philosophy? Coverage targets? What should/shouldn't be tested?"
   - "Any critical code organization rules? File size limits? Module structure patterns?"
   - "What's your git workflow? Commit format? Branch strategy?"
   - "Any project-specific tools or scripts I should document?"
   - "Language/framework-specific conventions or patterns?"

3. **Adversarial check**: "What operational conventions am I missing? Common mistakes to avoid? Critical tools or patterns?"

4. **Append customizations**: Add project-specific sections to the template content

---

## Example Customization Fragment

```markdown
# CLAUDE.md

**TaskFlow API**: REST API for task orchestration with webhook integrations

**Key Context**:
- Language: Python 3.11+
- Framework: FastAPI + SQLAlchemy
- Test runner: pytest with pytest-asyncio
- Key tools: alembic (migrations), celery (background jobs)

---

# Hegel DDD Methodology

[Hegel template content...]

---

# TaskFlow-Specific Conventions

## Testing Philosophy

**Coverage target**: ≥85% lines (higher due to financial domain)

**Test categories**:
- Unit: Business logic, validation, serialization
- Integration: Database transactions, webhook delivery
- Contract: API response schemas (using schemathesis)

**Critical test requirements**:
- ✅ All webhook handlers must have failure/retry tests
- ✅ Database migrations must have up/down tests
- ✅ All API endpoints must have authentication tests

## Code Organization

**Module structure**:
- `app/api/`: FastAPI routers (one per resource)
- `app/models/`: SQLAlchemy models
- `app/services/`: Business logic (no DB or HTTP concerns)
- `app/webhooks/`: Webhook handlers (idempotent by design)

**File size limits**:
- Routers: ≤150 lines (split by resource if larger)
- Services: ≤200 lines (one service per business concern)
- Models: ≤100 lines (split by aggregate if larger)

## Git Workflow

**Scopes**: `api`, `models`, `services`, `webhooks`, `migrations`, `tests`

**Branch strategy**:
- `main`: Production-ready, auto-deploys to staging
- `prod`: Production deployment (via PR from main)
- Feature branches: `feat/short-description` (merged via PR)

## Critical Patterns

**Webhook idempotency**: All webhook handlers must check `event_id` before processing

**Transaction boundaries**: Use `@transactional` decorator for service methods that modify state

**API error responses**: Always return `{"error": {"code": "...", "message": "..."}}`
```

---

## Conclusion

CLAUDE.md customization is about **operational specificity**. The Hegel template provides DDD methodology foundations. Your additions make AI assistants immediately productive on your specific codebase by documenting conventions, tools, patterns, and critical context they need to work effectively.
