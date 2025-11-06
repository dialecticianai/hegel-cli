## 🧠 Test Philosophy: Expressivity as Proof

In Hegel, tests are not just verification—they are **executable documentation**. Every test should *express* intent, not describe it.  
If you need comments to explain what a test does, that’s a design failure: the code should already say it clearly, *in code*.

### Principles

- **1. Expressivity over verbosity.**  
  Tests should read like English sentences in a purpose-built DSL.  
  Example:  

      let wf = workflow("discovery", "spec")
          .with_node("spec", node("Write SPEC.md", vec![transition("done", "plan")]))
          .build();

  Every token should carry semantic weight. Boilerplate is cognitive noise.

- **2. Documentation through execution.**  
  The only real proof of correctness is a running example.  
  Therefore, tests are the living documentation of system behavior.  
  They *are* the spec. They *are* the reference examples.  
  The test suite doubles as a “cognitive map” for LLMs and humans alike.

- **3. DSL as cognition amplifier.**  
  The test_helpers module is Hegel’s expressive substrate: a tiny internal language that lets tests *think efficiently.*  
  The goal is not to minimize lines of code—it’s to **maximize meaning per symbol**.

- **4. Comments are code smell.**  
  If a comment explains a test, the test’s structure has failed.  
  Rewrite it until it speaks clearly on its own.

- **5. Test failure as dialogue.**
  When a test fails, it should read as a coherent sentence:
  > "Expected workflow mode to be discovery, found execution."
  Tests are conversations between system and author, not error dumps.

### Test Organization

**Three-tier structure based on scale:**

1. **Inline tests** (`#[cfg(test)] mod tests {}`)
   - When: File <200 lines total, simple unit tests
   - Access: Can test private functions
   - Pattern: `src/engine/template.rs`

2. **Module test directory** (`src/<module>/tests/*.rs`)
   - When: Module growing, multiple scenarios, >200 line threshold
   - Access: Tests `pub(crate)` items, not private ones
   - Pattern: `src/commands/workflow/tests/transitions.rs`
   - Setup:
     ```rust
     // src/<module>/tests/mod.rs
     mod transitions;

     // src/<module>/mod.rs
     #[cfg(test)]
     mod tests;
     ```

3. **Integration tests** (`tests/` at project root)
   - When: End-to-end CLI behavior, public API verification
   - Separate compilation unit, slower builds

**Decision point:** When implementation file exceeds 200 lines, split tests to their own module. This preserves the implementation's readability while allowing test infrastructure to grow.

### Outcome

The result is a **self-explaining, self-verifying codebase**.  
Every test both instructs and proves.  
Every example is a runnable artifact of design intent.  
Documentation ceases to drift, because the truth *is* the running system.