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
- "PPU has 8 scanline sprite limit"
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
- What bottlenecks matter (vblank budget, sprite limit)?

✅ **Gotcha documentation**
- What edge cases break naive implementations?
- What common mistakes do sources warn about?
- What surprises exist (PPU write toggle, sprite Y-1 offset)?

✅ **Cross-referencing and integration**
- How do subsystems interact (DPCM glitches controller reads)?
- What dependencies exist (mapper choice affects CHR strategy)?
- Where do sources conflict (wiki vs practice)?

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
- [NESdev Wiki - PPU](https://www.nesdev.org/wiki/PPU) (cached to .webcache/nesdev/ppu.html)
- [Timing Guide](https://www.nesdev.org/wiki/Cycle_reference_chart)

**Created**: October 2025
**Last updated**: [Date of significant revision]
```

---

## Caching External Sources

Use `.webcache/` for offline stable copies:

```bash
# Cache HTML
curl -s https://www.nesdev.org/wiki/PPU -o .webcache/nesdev/ppu.html

# Read cached HTML as clean text
lynx -dump -nolist .webcache/nesdev/ppu.html

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
The PPU has a register at memory address $2000 called PPUCTRL.
When you write to this register, you can control various aspects
of how the PPU operates, including which nametable is used for
rendering and whether sprites are 8x8 or 8x16 pixels tall.
```

**Good (LLM-optimized density)**:
```markdown
**PPUCTRL ($2000)**: Controls PPU operation
- Bits 0-1: Base nametable ($2000/$2400/$2800/$2C00)
- Bit 5: Sprite size (0=8x8, 1=8x16)
- Bit 7: NMI enable (1=trigger on vblank)
```

**Key difference**: Facts + relationships in minimal tokens. No hand-holding.

---

## Synthesis Examples

### Bad: Transcription
```markdown
## PPU Registers

The PPU has 8 registers mapped to CPU memory:
- $2000: PPUCTRL
- $2001: PPUMASK
- $2002: PPUSTATUS
- $2003: OAMADDR
- $2004: OAMDATA
- $2005: PPUSCROLL
- $2006: PPUADDR
- $2007: PPUDATA

Each register has different functions...
[Continues with wiki copy-paste]
```

Why bad: Verbatim enumeration without insight. Just read the wiki.

### Good: Synthesis
```markdown
## PPU Register Programming

**Mental model**: CPU writes to 8 PPU registers ($2000-$2007) to control graphics.

**Key constraint**: Only safe during vblank (2273 cycles NTSC). Writing outside vblank causes corruption.

**Common patterns**:
- Bulk updates: Use $2006 (address) + $2007 (data) for VRAM writes
- DMA shortcut: $4014 copies 256-byte page to sprite memory (513 cycles)
- Toggle gotcha: $2005 and $2006 share internal toggle, reset via $2002 read

**Vblank priority order**:
1. OAM DMA (513 cycles) - sprites
2. Scroll updates ($2005) - camera position
3. VRAM writes ($2006/$2007) - nametable/CHR changes
4. Defer to main loop: audio updates, game logic

**Open questions**:
- Q: How many VRAM writes fit in remaining vblank budget? (Needs measurement)
- Q: What's the actual cycle cost of $2006/$2007 sequence? (Profile in emulator)
```

Why good: Mental model → constraints → patterns → gotchas → questions. Synthesized understanding, not transcribed facts.

---

## Cross-Referencing Strategy

Learning docs should interlink:

**Within doc**:
- "See section X below for details"
- "Constraints covered in Timing section"

**Across docs**:
- "See timing_and_interrupts.md for vblank budget"
- "Mapper choice affects CHR strategy (mappers.md)"

**To external sources**:
- "NESdev wiki PPU page (cached: .webcache/nesdev/ppu.html)"
- "Technique from Bisqwit's nescom.txt"

**To open questions**:
- "Q7.2 in .ddd/5_open_questions.md will measure CHR-RAM performance"

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

## Pattern from ddd-nes

**Study output**:
- 11 learning docs (wiki_architecture, sprite_techniques, audio, etc.)
- 5 meta-assessments (0_initial_questions → 4_mappers_complete)
- 1 consolidated questions doc (5_open_questions.md with 43 questions)

**Key insight**: Numbered meta-assessments (`0_`, `1_`, `2_`...) keep them filesystem-ordered without dates.

**Blog post #1 quote**:
> "Every document ends with an attribution footer linking back to the wiki. We're not replacing the community's knowledge—we're condensing it for a specific purpose: building working NES games as an AI-human pair."

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

From blog post #1:
> "Study revealed what we *understand* versus what we need to *validate through practice*."

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
