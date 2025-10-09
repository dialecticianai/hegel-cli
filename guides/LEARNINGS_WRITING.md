# Meta-Document: How to Write Effective LEARNINGS.md

_Guide for capturing architectural insights from experiments and features._

---

## When to Use This

**Discovery Mode**: **Required** — `toys/toyN_name/.ddd/LEARNINGS.md`
Every toy must have LEARNINGS.md documenting what worked, what failed, what remains uncertain.

**Execution Mode**: **Optional** — `.ddd/feat/<feature_name>/LEARNINGS.md`
Only write if architectural insights emerged (pivots, surprises, failures, lessons learned).

**When to Skip (Execution)**: Routine feature work with no unexpected learnings — most features won't need this.

**When to Write (Execution)**:
- Major architectural pivot during implementation
- Unexpected technical challenges or failures
- Design decision that impacts future features
- Discovery of significant technical debt or system limitations

---

## Purpose

A **LEARNINGS.md** is a short, dense retrospective.  
Its job: extract maximum value from an experiment by recording **what worked, what failed, what remains uncertain, and why.**

---

## What It Is / Is Not

### ❌ Not
- A feature list  
- Implementation details  
- A user manual  
- Purely positive  
- Hype or speculation without evidence  

### ✅ Is
- A record of validated insights  
- A log of failures and limitations  
- A map of open questions  
- A pointer to architectural reuse  
- A calibration tool for future experiments  

---

## Essential Sections

### Header

**Discovery Mode**:
```markdown
# Toy Model N: [System Name] – Learnings
Duration: X days | Status: [Complete/Incomplete] | Estimate: Y days
```

**Execution Mode**:
```markdown
# [Feature Name] – Learnings
Duration: X days | Status: Complete | Estimate: Y days
```

### Summary
- Built: [1 line]  
- Worked: [1–2 key successes]  
- Failed: [1–2 key failures]  
- Uncertain: [open question]

### Evidence
- ✅ Validated: [concise finding with evidence]  
- ⚠️ Challenged: [difficulty, workaround, lesson]  
- ❌ Failed: [explicit dead end]  
- 🌀 Uncertain: [still unresolved]

### Pivots
- [Original approach] → [New approach], why, and what remains unknown.

### Impact
- Reusable pattern or asset  
- Architectural consequence  
- Estimate calibration (time/effort vs. outcome)

---

## Style

- Keep it **short and factual**.  
- Prefer **bullet points** over prose.  
- Note **failures and unknowns** as explicitly as successes.  
- One page max — dense, parsimonious, reusable.  