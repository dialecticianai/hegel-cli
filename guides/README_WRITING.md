# README Writing Guidelines

General principles for creating AI assistant-focused README documentation.

---

## Purpose

READMEs serve as **context refresh documents** for AI assistants working with code. They should quickly establish understanding of what the code does, how to use it, and what to watch out for.

**Target audience**: AI assistants needing to rapidly understand purpose and usage patterns
**Focus**: Dense, essential information only

**Note**: Specific workflows provide context-specific structure and requirements (length, sections, etc.). These are universal writing principles that apply across all README types.

---

## Writing Guidelines

### **Be Concise**
- Use bullet points over paragraphs
- Focus on essential information only
- Assume reader has basic programming knowledge

### **Be Specific**
- Include actual signatures, commands, or APIs, not generic descriptions
- Mention specific constraints (e.g., "performance degrades beyond 10K items")
- Reference specific files or tests for examples

### **Be Practical**
- Lead with most commonly used functionality
- Highlight integration points with other components
- Focus on "what you need to know to use this correctly"

### **Avoid**
- Marketing language or feature lists
- Detailed implementation explanations
- Extensive examples (prefer links to code/tests)
- Unnecessary setup details (unless workflow-specific)

---

{{templates/mirror_workflow}}

---

## Quality Check

A good README should allow an AI assistant to:
1. **Understand purpose** quickly (seconds, not minutes)
2. **Know primary operations** to perform
3. **Avoid common mistakes** through warnings or caveats
4. **Validate or test** the functionality

If understanding takes longer than expected, the README needs to be more concise or better organized.
