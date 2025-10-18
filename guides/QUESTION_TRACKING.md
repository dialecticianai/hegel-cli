# Question Tracking Guide

**Purpose**: Catalogue open questions that bridge Research → Discovery modes.

**Context**: Research mode questions phase. You've studied external sources, now organize what remains uncertain.

---

## Why Track Questions Explicitly

**The problem without tracking**:
- Scattered "TODO" comments in learning docs
- Vague sense of "need to test X"
- Forget which questions are foundational vs nice-to-have
- No clear Discovery phase roadmap

**The solution with tracking**:
- Consolidated questions document
- Clear categorization (answered vs open, foundational vs advanced)
- Cross-references (theory → practice path)
- Discovery roadmap emerges naturally

The open questions document becomes a roadmap for practical work. Each question cross-references which learning doc has the theory and which experiment will provide the answer.

---

## File Operations

**CRITICAL**: Write your questions document to a file, don't paste in chat.

1. **Write to `learnings/.ddd/open_questions.md`** using the Write tool (or appropriate location for consolidated questions)
2. **Use numbered questions** (e.g., Q1.1, Q1.2) for cross-referencing
3. **Mark answered questions** with ✅ and keep them for tracking
4. **Launch review**: Run `hegel reflect learnings/.ddd/open_questions.md` to open GUI for user review
5. **Read feedback**: Check `.ddd/open_questions.review.*` files if user adds comments
6. **Incorporate feedback**: Update questions doc based on review before proceeding to Discovery phase

The review GUI allows the user to select text and add inline comments.

---

## Question Document Structure

### Header
```markdown
# Open Questions — [Project/Domain Name]

**Created**: [Date]
**Purpose**: Central tracking of questions from Research phase
**Status**: [N answered, M open, P total]

---

## Quick Summary

**Study complete**: [What was covered]
**Open questions**: [Count by category]
**Answered/decided**: [Count resolved during study]
**Primary blockers**: [Any showstoppers?]

**Categories**:
1. [Category Name] (X open, Y answered)
2. [Category Name] (X open, Y answered)
...
```

### Per-Category Structure

```markdown
## N. Category Name

### Question Text
**QN.M**: [Precise question]?
- [Context or elaboration if needed]
- [Why this matters]
- **Answer via**: [How will this be resolved - which toy/experiment/tool]

### ✅ Answered Questions
**QN.M**: [Question]?
- ✅ **ANSWERED**: [Decision/finding]
  - Source: [Where answer came from - learning doc, decision, measurement]
  - Alternative: [Other options if relevant]
- **Next step**: [What to do with this answer]
```

---

## Question Types

### Theory Questions (Answered During Study)
**Can be resolved by reading/synthesis**

Examples:
- Q: Which audio library should we use?
- Q: What rendering backend progression strategy makes sense?
- Q: Should we use experimental API features?

**Mark as answered** with rationale and source:
```markdown
**Q3.1**: Which audio library to use?
- ✅ **ANSWERED**: LibAudio (beginner-friendly)
  - Source: `learnings/audio.md` - 8 libraries compared
  - Alternative: AdvancedAudio if rich features needed later
- **Next step**: Integrate LibAudio in audio test program
```

### Practice Questions (Need Experimentation)
**Require building/measuring to answer**

Examples:
- Q: How many memory writes fit in vsync budget?
- Q: What's buffer copy actual performance?
- Q: How to use debugger/profiler effectively?

**Mark as open** with practice path:
```markdown
**Q1.4**: How to measure actual cycle usage?
- Debugger's cycle counter?
- Manual counting vs profiler?
- Validate frame budget adherence?
- **Answer via**: Profile test program routines (DMA, buffer copy, etc.)
```

### Decision Questions (Context-Dependent)
**Wait for more information**

Examples:
- Q: Buffered or streaming rendering for our application?
- Q: How much fast memory to allocate per subsystem?
- Q: Which advanced features do we need?

**Mark as deferred** with decision trigger:
```markdown
**Q5.2**: Buffered or streaming rendering?
- **Pending**: Wait for SPEC.md (application type decision)
- Real-time → Buffered (predictable)
- Interactive → Streaming (flexible)
- **Answer via**: Define application type, choose rendering strategy
```

---

## Categorization Strategies

### By Domain/Subsystem
Works well for system/architecture projects:
- Toolchain & Build
- Rendering Pipeline
- Audio System
- Input Handling
- Application Architecture
- Optimization & Performance
- Testing & Validation

### By Workflow Phase
Works well for methodology projects:
- Setup & Installation
- Initial Implementation
- Integration & Testing
- Optimization & Polish
- Deployment & Distribution

### By Priority
Works for any project:
- Blockers (can't start without answers)
- Foundational (need early, but not blocking)
- Advanced (optimize later)
- Nice-to-have (defer indefinitely)

**Choose ONE categorization scheme** and stick with it. Don't mix.

---

## Cross-Referencing

Every question should link to:

**1. Theory source** (where the concept is documented):
```markdown
**Q6.6**: When to use expensive operations - cost/benefit?
- **Theory**: `learnings/performance.md` - All operations documented with costs
```

**2. Practice path** (how it will be answered):
```markdown
- **Answer via**: Profile operation usage in application, pre-compute tables where feasible
```

**3. Related questions** (dependencies or clusters):
```markdown
**Q6.3**: How to allocate limited fast memory?
- Related: Q6.4 (which data structures deserve fast memory)
- Depends on: Q4.2 (system architecture size)
```

**Goal**: Every question traceable both backward (where theory is) and forward (how it gets answered).

---

## The Discovery Roadmap Pattern

Open questions naturally organize into Discovery phase structure:

**Example Discovery roadmap**:
```markdown
## Next Steps to Answer These Questions

### Phase 1: Toolchain Setup (Answers Q1.1-Q1.8, Q3.4)
1. Install compiler, debugger, build tools
2. Run validation test suite
3. Create build script (Makefile/CMake)
4. Document toolchain setup process

### Phase 2: First Test Program (Answers Q1.3-Q1.6, Q2.1-Q2.2, Q6.3)
1. Build "hello world" minimal program
2. Render basic object (test rendering workflow)
3. Read input device (test input)
4. Play sound (test basic audio)
5. Profile performance (measure actual costs)
6. Document findings in learning docs
```

**Pattern**: Group questions by what experiment/toy answers them. That's your Discovery phase plan.

---

## Question Evolution

Questions transform over time:

**Study phase** → **Discovery phase** → **Execution phase**

Example trajectory:

**Research (Study)**:
- Q: How does batch DMA transfer work?
- Answer: Read documentation, document in learning doc

**Discovery (Toy)**:
- Q: What's actual cycle cost of batch DMA?
- Answer: Build toy1_dma_test, measure with profiler
- Finding: ~500 cycles (confirms theory)

**Execution (Production)**:
- Q: How to integrate DMA with scrolling updates?
- Answer: Implement in main application, handle edge cases
- Finding: Must reset scroll state after DMA (hardware quirk)

**Update questions doc** as questions evolve/resolve:
- Mark answered questions
- Add new questions discovered during practice
- Remove obsolete questions (context changed)

---

---

## Common Patterns

### Greenfield Projects
- Many open questions initially
- Rapid question resolution during Discovery
- New questions emerge from practice
- Questions shift from "how" to "which approach"

### Porting Projects
- Fewer architectural questions (reference exists)
- More translation questions (source idiom → target idiom)
- Validation questions (does translation preserve behavior)

### Research-Heavy Projects
- Deep question hierarchies
- Many answered during study (literature exists)
- Practice validates theoretical understanding
- Meta-questions about methodology emerge

---

## Anti-Patterns

**Question dumping**: Long list with no structure
- **Fix**: Categorize and number

**Vague questions**: "How does X work?"
- **Fix**: Specific, answerable questions with clear practice path

**Orphan questions**: No theory source or practice path
- **Fix**: Cross-reference to learning docs and experiments

**Static document**: Never updated as questions resolve
- **Fix**: Mark answered, add new questions, keep current

**Perfectionism**: Must answer all questions before building
- **Fix**: Distinguish blockers from nice-to-have, timebox research

---

## Key Insight

Questions document is the **bridge between Research and Discovery**.

**Research output**: Learning docs (theory) + Questions (uncertainties)
**Discovery input**: Questions (roadmap) → Toys (experiments)
**Discovery output**: Answers (findings) → Updated learning docs (validated theory)

---

## Remember

From LEXICON.md:
> "Context is king. What's visible determines what's possible."

Question tracking makes uncertainty visible. Visible uncertainty becomes actionable roadmap.

Each toy validates one subsystem. The questions become experiments. The experiments become knowledge.

Questions aren't just uncertainty. They're potential energy waiting to become understanding through practice.

---

## Template

```markdown
# Open Questions — [Project Name]

**Created**: [Date]
**Purpose**: Central tracking from Research phase
**Status**: [X open, Y answered, Z total]

---

## Quick Summary

**Study complete**: [What was covered]
**Open questions**: [Count by category]
**Primary blockers**: [Any?]

**Categories**:
1. [Category] (X open, Y answered)
...

---

## 1. [Category Name]

### [Subsection if needed]
**Q1.1**: [Question]?
- [Context]
- **Answer via**: [Practice path]

### ✅ [Answered Subsection]
**Q1.2**: [Question]?
- ✅ **ANSWERED**: [Decision]
  - Source: [Where]
- **Next step**: [Action]

---

## Next Steps to Answer These Questions

### Phase 1: [Name] (Answers Q1.1-Q1.3)
1. [Experiment/toy]
2. [Experiment/toy]

### Phase 2: [Name] (Answers Q2.1-Q2.4)
...

---

## Status: [Ready/Blocked/In Progress]

[Final note on readiness for Discovery phase]
```
