# Knowledge Capture Guide

**Purpose**: Transform external sources into synthesized learning documents.

**Context**: Research mode study phase. You're reading wikis, papers, codebases - now extract what matters for building.

---

## The Core Principle

**Synthesis, not transcription.**

You're not creating a local copy of the wiki. You're building a mental model optimized for AI-human collaboration on real projects.

From LEXICON.md:
> "Artifacts disposable, clarity durable"

The external source is disposable (cached for reference). Your understanding is durable (captured in learning docs).

---

## What Knowledge Capture Is NOT

❌ **Copying wiki content verbatim**
- That's transcription, not synthesis
- Adds no value (source already exists)
- Wastes tokens (verbose without insight)

❌ **Historical storytelling**
- "In 1985, Nintendo released..."
- Interesting but not actionable
- Defer to external sources for history

❌ **Complete enumeration**
- Listing every instruction, every register bit
- Reference material, not learning material
- Point to source for exhaustive details

❌ **Isolated facts without context**
- "System has 8-object render limit per scanline"
- Missing: Why? What does this constrain? How do you work with it?

---

## What Knowledge Capture IS

✅ **Pattern extraction**
- What techniques emerge across multiple sources?
- What patterns repeat (init sequences, common loops)?
- What mental models help organize the domain?

✅ **Constraint identification**
- What are the hard limits (hardware, timing, memory)?
- What trade-offs exist (speed vs space, flexibility vs simplicity)?
- What bottlenecks matter (frame budget, resource limits)?

✅ **Gotcha documentation**
- What edge cases break naive implementations?
- What common mistakes do sources warn about?
- What surprises exist (write ordering, state toggles)?

✅ **Cross-referencing and integration**
- How do subsystems interact (unexpected side effects)?
- What dependencies exist (architectural choices cascade)?
- Where do sources conflict (documentation vs reality)?

✅ **Theory vs practice gaps**
- What can you understand from reading?
- What needs practical validation (cycle counts, timing)?
- What questions arise for Discovery phase?

---

## File Operations

**CRITICAL**: Write your learning documents to files, don't paste in chat.

1. **Write to `learnings/[topic].md`** using the Write tool
2. **Use markdown formatting** with clear headers and structure
3. **Cross-reference** other learning docs and external sources
4. **Launch review**: Run `hegel reflect learnings/[topic].md` to open GUI for user review
5. **Read feedback**: Check `.ddd/[topic].review.*` files if user adds comments
6. **Incorporate feedback**: Update learning doc based on review before proceeding

The review GUI allows the user to select text and add inline comments.

---

## Learning Document Structure

### Header
```markdown
# [Topic Name]

**Purpose**: [One sentence - what this doc covers]

**Audience**: AI agents (concise, technical) but human-friendly

**Sources**: [Primary wiki pages, papers, etc. with URLs]
```

### Core Content Sections

Common patterns (adapt to domain):

1. **Overview** - 2-3 paragraphs establishing mental model
2. **Key Concepts** - Essential terms and relationships
3. **Constraints** - Hard limits and trade-offs
4. **Common Patterns** - Techniques that work
5. **Gotchas** - Edge cases and surprises
6. **Open Questions** - Theory vs practice gaps
7. **References** - Attribution and further reading

### Footer Attribution
```markdown
---

**Sources**:
- [API Documentation](https://example.org/docs/api) (cached to .webcache/docs/api.html)
- [Performance Guide](https://example.org/docs/perf)

**Created**: October 2025
**Last updated**: [Date of significant revision]
```

---

## Caching External Sources

Use `.webcache/` for offline stable copies:

```bash
# Cache HTML
curl -s https://example.org/docs/api -o .webcache/docs/api.html

# Read cached HTML as clean text
lynx -dump -nolist .webcache/docs/api.html

# Or use WebFetch tool in Claude Code
```

**Why cache?**
- Offline access (no network dependency)
- Version stability (wiki pages change)
- Faster lookup (local file vs HTTP request)

**Reference cached files in attribution**, but don't make learning docs dependent on cache structure.

---

## Writing for AI Agents (Not Humans)

From DDD.md section "Design for LLM Consumption":

**LLM token economics are different:**
- Concise ≠ unclear (brevity reduces tokens without reducing understanding)
- Self-documenting code > verbose comments (LLMs parse both equally)
- Domain language > implementation details (speak concepts, not mechanics)

**Practical shifts for learning docs:**

**Bad (human-optimized verbosity)**:
```markdown
The configuration register at offset 0x100 controls several aspects
of the rendering system. When you write to this register, you can
control which buffer is active and whether objects use standard or
extended dimensions.
```

**Good (LLM-optimized density)**:
```markdown
**CONFIG_REG (0x100)**: Controls rendering behavior
- Bits 0-1: Active buffer selection (0-3)
- Bit 5: Object size (0=standard, 1=extended)
- Bit 7: Interrupt enable (1=trigger on vsync)
```

**Key difference**: Facts + relationships in minimal tokens. No hand-holding.

---

## Synthesis Examples

### Bad: Transcription
```markdown
## Hardware Registers

The system has 8 registers mapped to memory:
- 0x100: CONFIG
- 0x101: MASK
- 0x102: STATUS
- 0x103: ADDR_LO
- 0x104: ADDR_HI
- 0x105: SCROLL
- 0x106: DATA_ADDR
- 0x107: DATA

Each register has different functions...
[Continues with documentation copy-paste]
```

Why bad: Verbatim enumeration without insight. Just read the docs.

### Good: Synthesis
```markdown
## Hardware Register Programming

**Mental model**: CPU writes to 8 hardware registers (0x100-0x107) to control rendering.

**Key constraint**: Only safe during vsync period (~2.3ms). Writing outside vsync causes corruption.

**Common patterns**:
- Bulk updates: Use ADDR (0x106) + DATA (0x107) for memory writes
- DMA shortcut: 0x200 copies page to object memory (efficient batch transfer)
- Toggle gotcha: SCROLL and ADDR share internal state, reset via STATUS read

**Vsync priority order**:
1. DMA transfer (batch objects)
2. Scroll updates (camera position)
3. Memory writes (tilemap/texture changes)
4. Defer to main loop: audio, input, game logic

**Open questions**:
- Q: How many memory writes fit in remaining vsync budget? (Needs measurement)
- Q: What's the actual cycle cost of ADDR+DATA sequence? (Profile on hardware)
```

Why good: Mental model → constraints → patterns → gotchas → questions. Synthesized understanding, not transcribed facts.

---

## Cross-Referencing Strategy

Learning docs should interlink:

**Within doc**:
- "See section X below for details"
- "Constraints covered in Timing section"

**Across docs**:
- "See timing_and_interrupts.md for frame budget"
- "Rendering mode choice affects texture strategy (rendering.md)"

**To external sources**:
- "API documentation (cached: .webcache/docs/api.html)"
- "Technique from reference implementation"

**To open questions**:
- "Q7.2 in .ddd/open_questions.md will measure buffer copy performance"

**Goal**: Every fact has a source. Every gap has a question. Nothing is unsupported.

---

## The Assessment Loop

After studying a priority group, step back and assess:

**Create meta-assessment doc** (e.g., `learnings/.ddd/1_essential_techniques.md`):

```markdown
# PHASE 1 — Essential Techniques Assessment

**Date**: [Session date]
**Phase**: Study of Priority 1 topics
**Status**: [Complete/In Progress]

---

## Questions Answered
[List questions from previous phase that this study resolved]

## Questions Raised
[New questions that emerged during study]

## Decisions Made
[Any tool/technique choices made during this phase]

## Next Steps
[Continue to Priority 2? Start practical work?]
```

**Why assess?**
- Tracks progress (what's covered, what remains)
- Surfaces learning (insights, not just facts)
- Identifies gaps (theory vs practice)
- Prevents drift (stay on track with priorities)

---

---

## Common Patterns by Domain

### Hardware/Architecture
- Memory maps and register lists
- Timing constraints and cycle budgets
- Subsystem interactions and gotchas

### Toolchain/Build
- Tool selection with rationale
- Setup commands (reproducible)
- Build workflow and automation

### Techniques/Patterns
- Common code patterns
- Optimization strategies
- When to use (decision criteria)

### Theoretical Foundations
- Mental models and abstractions
- Design principles
- Trade-off analysis

---

## Anti-Patterns

**Over-abstracting**: Don't invent your own terminology when domain has established terms.

**Under-synthesizing**: If your learning doc is 90% quotes, you haven't extracted patterns yet.

**Perfectionism**: Don't aim for textbook completeness. Aim for: enough to start building + awareness of gaps.

**Hoarding**: Don't cache/document everything "just in case". Follow priority plan. Defer advanced topics.

---

## Key Insight

Research mode documents your understanding. Discovery mode validates it.

**Theory** (learning docs): What sources say, patterns identified, constraints understood

**Practice** (toy experiments): What actually works, edge cases discovered, measurements taken

The gap between them = open questions = Discovery roadmap.

---

## Remember

Knowledge capture is about **building a mental model optimized for building**.

Not: Local wiki copy
Not: Historical narrative
Not: Complete reference manual

Yes: Patterns + constraints + gotchas + questions
Yes: Enough to start building
Yes: Awareness of what you don't know yet

From LEXICON.md:
> "Documents! Documents! Documents!"

But documents that capture UNDERSTANDING, not just INFORMATION.
