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

From ddd-nes blog #1:
> "The open questions document became a roadmap for practical work. Each question cross-references which learning doc has the theory and which test ROM will provide the answer."

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
- Q: Which sound engine should we use?
- Q: What mapper progression strategy makes sense?
- Q: Should we avoid unofficial opcodes?

**Mark as answered** with rationale and source:
```markdown
**Q3.1**: Which sound engine to use?
- ✅ **ANSWERED**: FamiTone2 (beginner-friendly)
  - Source: `learnings/audio.md` - 8 engines compared
  - Alternative: FamiStudio if rich features needed later
- **Next step**: Integrate FamiTone2 in audio test ROM
```

### Practice Questions (Need Experimentation)
**Require building/measuring to answer**

Examples:
- Q: How many VRAM writes fit in vblank budget?
- Q: What's CHR-RAM copy actual performance?
- Q: How to use Mesen debugger effectively?

**Mark as open** with practice path:
```markdown
**Q1.4**: How to measure actual cycle usage?
- Mesen's cycle counter?
- Manual counting vs profiler?
- Validate vblank budget adherence?
- **Answer via**: Profile test ROM routines (OAM DMA, tile copy, etc.)
```

### Decision Questions (Context-Dependent)
**Wait for more information**

Examples:
- Q: CHR-ROM or CHR-RAM for our game?
- Q: How much zero page to allocate per subsystem?
- Q: Which advanced mapper features do we need?

**Mark as deferred** with decision trigger:
```markdown
**Q5.2**: CHR-ROM or CHR-RAM for ddd-nes?
- **Pending**: Wait for SPEC.md (game genre decision)
- Action/platformer → CHR-ROM (speed)
- RPG/puzzle → CHR-RAM (flexibility)
- **Answer via**: Define game genre, choose CHR strategy
```

---

## Categorization Strategies

### By Domain/Subsystem
Works well for hardware/architecture projects:
- Toolchain & Build
- Graphics/PPU
- Audio/APU
- Input/Controllers
- Game Architecture
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
**Q6.6**: When to use math routines - cost/benefit?
- **Theory**: `learnings/math_routines.md` - All routines documented with cycle costs
```

**2. Practice path** (how it will be answered):
```markdown
- **Answer via**: Profile math usage in game, pre-compute tables where feasible
```

**3. Related questions** (dependencies or clusters):
```markdown
**Q6.3**: How to allocate 256 bytes of zero page?
- Related: Q6.4 (which variables deserve ZP)
- Depends on: Q4.2 (entity system size)
```

**Goal**: Every question traceable both backward (where theory is) and forward (how it gets answered).

---

## The Discovery Roadmap Pattern

Open questions naturally organize into Discovery phase structure:

**From ddd-nes `5_open_questions.md`**:
```markdown
## Next Steps to Answer These Questions

### Phase 1: Toolchain Setup (Answers Q1.1-Q1.8, Q3.4)
1. Install asm6f, Mesen, NEXXT, FamiTracker
2. Run blargg test ROM suite
3. Create build script (Makefile)
4. Document toolchain setup process

### Phase 2: First Test ROM (Answers Q1.3-Q1.6, Q2.1-Q2.2, Q6.3)
1. Build "hello world" NROM ROM
2. Display sprite (test graphics workflow)
3. Read controller (test input)
4. Play beep (test basic audio)
5. Profile cycle usage (measure actual costs)
6. Document findings in learning docs
```

**Pattern**: Group questions by what experiment/toy answers them. That's your Discovery phase plan.

---

## Question Evolution

Questions transform over time:

**Study phase** → **Discovery phase** → **Execution phase**

Example trajectory:

**Research (Study)**:
- Q: How does sprite DMA work?
- Answer: Read wiki, document in learning doc

**Discovery (Toy)**:
- Q: What's actual cycle cost of OAM DMA?
- Answer: Build toy1_sprite_dma, measure in emulator
- Finding: 513-514 cycles (confirms theory)

**Execution (Production)**:
- Q: How to integrate sprite DMA with scrolling?
- Answer: Implement in main game, handle edge cases
- Finding: Must reset scroll after DMA (PPU quirk)

**Update questions doc** as questions evolve/resolve:
- Mark answered questions
- Add new questions discovered during practice
- Remove obsolete questions (context changed)

---

## Pattern from ddd-nes

**43 total questions**:
- 36 open (need practice)
- 7 answered (resolved during study)

**Organization**:
- 7 categories by subsystem
- Each question numbered (Q1.1, Q1.2, etc.)
- Answered questions marked with ✅
- Practice path specified for open questions
- Discovery roadmap in final section

**Key structure**:
```markdown
## 1. Toolchain & Development Workflow

### Build Pipeline Integration
**Q1.1**: How to integrate asm6f + NEXXT + FamiTracker?
- Makefile? Shell script? Both?
- **Answer via**: Build first test ROM, document workflow

### ✅ Sound Engine Integration (ANSWERED)
**Q3.1**: Which sound engine to use?
- ✅ **ANSWERED**: FamiTone2 (beginner-friendly)
  - Source: `learnings/audio.md`
- **Next step**: Integrate in audio test ROM
```

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

From blog post #1:
> "Theory vs practice: The 43 questions. Study revealed what we understand versus what we need to validate through practice."

---

## Remember

From LEXICON.md:
> "Context is king. What's visible determines what's possible."

Question tracking makes uncertainty visible. Visible uncertainty becomes actionable roadmap.

From blog post #9 (Productivity FOOM):
> "Each toy validates one subsystem. The questions become experiments. The experiments become knowledge."

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
