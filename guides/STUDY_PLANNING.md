# Study Planning Guide

**Purpose**: Structure research priorities before diving into external sources.

**Context**: Research mode is pre-Discovery knowledge gathering. You're studying external docs (wikis, papers, codebases) to build foundational understanding before building anything.

---

## Why Plan Before Studying

**The trap**: Random walk through documentation
- Jump from topic to topic
- Miss foundational concepts
- Accumulate facts without understanding
- Never feel "ready" to start building

**The solution**: Priority-driven systematic study
- Identify what's foundational vs what can wait
- Study in coherent chunks (complete a priority before moving on)
- Know when you're done (success criteria)
- Balance thoroughness with time boxing

---

## What to Include in Study Plan

### 1. Knowledge Domains

What areas need study? Examples:
- Core architecture (memory maps, timing, constraints)
- Subsystems (graphics, audio, input, networking)
- Toolchain (assemblers, compilers, debuggers, build systems)
- Patterns (common techniques, optimization strategies)
- Meta-knowledge (best practices, gotchas, edge cases)

Be specific. "Study graphics" is vague. "PPU architecture, sprite techniques, scrolling patterns" is clear.

### 2. Priority Ordering

What's foundational vs what can defer?

**Priority 0 (Foundational)**: Can't understand anything else without this
- Core architecture
- Memory organization
- Basic operation (how the system works at all)

**Priority 1-2 (Core)**: Need for any real work
- Key subsystems (graphics, input, timing)
- Common patterns (init sequences, main loops)
- Toolchain basics

**Priority 3-4 (Advanced)**: Optimization, special techniques
- Performance tuning
- Advanced effects
- Audio/music systems

**Priority 5+ (Specialized)**: Nice to know, not blocking
- Historical context
- Alternative approaches
- Deep dives on niche topics

Number priorities explicitly. This helps with progress tracking and prevents scope creep.

### 3. Success Criteria

How do you know when research phase is complete?

**Bad criteria**:
- "Read everything" (infinite scope)
- "Understand completely" (impossible without practice)
- "Feel confident" (subjective, blocks progress)

**Good criteria**:
- "Core priorities (0-2) studied and documented"
- "Toolchain validated (can build hello world)"
- "Open questions catalogued (theory vs practice gaps clear)"
- "Ready to start practical experiments"

Success = enough knowledge to start building, not perfect knowledge.

### 4. Time Boxing

How long should research take?

**General guidance**:
- Greenfield domain: 1-3 sessions (systematic coverage)
- Familiar domain: 0.5-1 session (refresh + gaps)
- Specialized topic: 0.5 session (focused study)

**Warning signs you're over-studying**:
- Diminishing returns (reading similar content repeatedly)
- Analysis paralysis (afraid to start building)
- Perfectionism (must understand everything before practice)

**The shift**: Study phase complete → Discovery phase begins. Theory meets practice. That's when real learning happens.

---

## Study Plan Format

Keep it simple. Numbered priorities work well:

```markdown
# Research Plan - [Domain Name]

**Goal**: Build foundational knowledge for [specific purpose]

**Success criteria**:
- Priorities 0-2 complete
- Learning docs created for key topics
- Toolchain validated (can build/run)
- Open questions catalogued

---

## Priority 0: Foundational (MUST KNOW)
- Core architecture (memory, timing, constraints)
- Basic operation (how system works)
- Target: 1 session

## Priority 1: Essential Subsystems (CORE)
- Graphics/PPU
- Input handling
- Timing/interrupts
- Target: 1 session

## Priority 2: Toolchain & Patterns (PRACTICAL)
- Assembler/compiler selection
- Build workflow
- Common init patterns
- Target: 0.5 session

## Priority 3: Optimization (ADVANCED) - DEFER
- Performance techniques
- Advanced effects
- Study after practical experience

## Priority 4: Audio (SPECIALIZED) - DEFER
- Sound engines
- Music systems
- Study when needed for project
```

---

## Pattern from ddd-nes

The ddd-nes project provides a reference implementation:

**Study structure**:
- 5 priority groups (0-4)
- 52 wiki pages studied systematically
- 11 learning documents created
- 5 meta-assessments tracking progress
- 43 questions catalogued (36 open, 7 answered)

**Meta-assessments after each priority**:
- `0_initial_questions.md` - Starting point
- `1_essential_techniques.md` - Priority 1-2 gaps
- `2_toolchain_optimization.md` - Priority 2.5-3 insights
- `3_audio_complete.md` - Priority 4 workflow
- `4_mappers_complete.md` - Priority 5 strategy

**Outcome**: Clear transition point to Discovery mode with roadmap of questions to answer through toys.

---

## Common Patterns

### Greenfield Domain (No Prior Knowledge)
- Start broad (architecture overview)
- Identify key subsystems
- Study each systematically
- Cross-reference as you go

### Specialized Topic (Adding to Existing Knowledge)
- Define specific gap (what don't you know)
- Study focused sources
- Integrate with existing mental model
- Quick cycle (0.5-1 session)

### Toolchain Validation
- Don't just read, test
- Build minimal example early
- Validate assumptions (macOS ARM64 vs docs claiming "macOS support")
- Document actual setup (not theoretical)

---

## Anti-Patterns

**Infinite Reading**: "Just one more page" → never start building
- **Fix**: Set priority ceiling, defer advanced topics

**Random Walk**: Follow interesting tangents, lose thread
- **Fix**: Stick to priority order, note tangents for later

**Transcription**: Copy wiki into learning docs verbatim
- **Fix**: Synthesize - extract patterns, constraints, gotchas

**Perfectionism**: Must understand 100% before practice
- **Fix**: Good enough to start building. Practice reveals gaps.

**No Meta-Tracking**: Can't remember what you've covered
- **Fix**: Create assessment docs after each priority

---

## Key Insight

Research mode is about building enough mental scaffolding to start practical work.

**Not**: Perfect understanding
**Yes**: Foundational knowledge + roadmap of unknowns

Theory meets practice in Discovery mode. That's when learning compounds.

---

## Remember

From LEXICON.md:
> "Artifacts disposable, clarity durable"

Learning docs are the durable artifact. External sources are disposable (cached for reference, but synthesized understanding is what matters).

From ddd-nes blog #1:
> "The deliverable isn't the game yet. It's the knowledge base."

Research mode embodies this: knowledge capture as primary deliverable.
