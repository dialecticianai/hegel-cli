# Claim Refactor: JSON Objects → Simple Strings

**Status**: Planning
**Motivation**: Simplify claim API from `{"claim_name": true}` to just `"claim_name"`

---

## Current State

**Claim format**: JSON objects with boolean values
```bash
hegel next '{"spec_complete": true}'
hegel next '{"needs_refactor": true}'
```

**Type system**: `HashMap<String, bool>` throughout codebase

**Why this is unnecessarily complex:**
- The boolean value is always `true` (100% of cases)
- JSON parsing overhead for no benefit
- More error-prone (syntax errors, wrong value type)
- Harder to use from command line (quoting, escaping)
- Conceptually confusing (implies claims could be `false`, but they can't)

---

## Proposed State

**Claim format**: Simple strings
```bash
hegel next spec_complete
hegel next needs_refactor
```

**Type system**: `HashSet<String>` or `Vec<String>`

**Benefits:**
- Simpler CLI: no JSON parsing, no quoting needed
- Clearer semantics: claims are presence-based, not boolean
- Easier to use: `hegel next spec_complete` vs `hegel next '{"spec_complete": true}'`
- Less error-prone: no JSON syntax to get wrong
- More intuitive: aligns with how claims actually work (present/absent, not true/false)

---

## Migration Strategy

### Phase 1: Core Type Changes

**1.1 Update type definitions**
- [ ] `src/commands/workflow/claims.rs` - Change `HashMap<String, bool>` → `HashSet<String>`
- [ ] `src/engine/mod.rs` - Update `get_next_prompt()` signature
- [ ] `src/commands/workflow/transitions.rs` - Update transition evaluation
- [ ] `src/commands/workflow/mod.rs` - Update advance_workflow()

**1.2 Update ClaimAlias implementation**
```rust
// Before
pub fn to_claims(&self, current_node: &str) -> Result<HashMap<String, bool>> {
    match self {
        Self::Next => Ok(HashMap::from([(
            format!("{}_complete", current_node),
            true,
        )])),
        Self::Restart => Ok(HashMap::from([("restart_cycle".to_string(), true)])),
        Self::Custom(json) => serde_json::from_str(json)
            .context("Failed to parse claims JSON. Expected format: {\"claim_name\": true}"),
    }
}

// After
pub fn to_claims(&self, current_node: &str) -> Result<HashSet<String>> {
    match self {
        Self::Next => Ok(HashSet::from([
            format!("{}_complete", current_node),
        ])),
        Self::Restart => Ok(HashSet::from(["restart_cycle".to_string()])),
        Self::Custom(claim) => Ok(HashSet::from([claim.to_string()])),
    }
}
```

**1.3 Update engine transition logic**
```rust
// Before
if claims.get(&transition.when) == Some(&true) {
    next = transition.to.clone();
    break;
}

// After
if claims.contains(&transition.when) {
    next = transition.to.clone();
    break;
}
```

### Phase 2: CLI Argument Parsing

**2.1 Update main.rs command parsing**
```rust
// Before
Commands::Next { claims } => {
    commands::next_prompt(claims.as_deref(), &storage)?;
}

// After
Commands::Next { claim } => {
    commands::next_prompt(claim.as_deref(), &storage)?;
}
```

**2.2 Update clap argument definition**
```rust
// Before
Next {
    /// Optional claims as JSON string (e.g., '{"spec_complete": true}')
    /// If omitted, uses happy-path claim: {"{current}_complete": true}
    claims: Option<String>,
},

// After
Next {
    /// Optional claim name (e.g., 'spec_complete', 'needs_refactor')
    /// If omitted, uses happy-path claim: {current}_complete
    claim: Option<String>,
},
```

### Phase 3: Update Tests

**3.1 Test helper functions**
- [ ] `src/commands/workflow/tests/mod.rs` - Update `next_with()` helper
- [ ] `src/test_helpers/workflow.rs` - Update `claim()` helper to return HashSet

**3.2 Test cases to update**
```rust
// Before
next_with(r#"{"spec_complete": true}"#, &storage);

// After
next_with("spec_complete", &storage);
```

**Files to update:**
- `src/commands/workflow/tests/commands.rs` - ~10 test cases
- `src/commands/workflow/tests/transitions.rs` - ~5 test cases
- `src/commands/workflow/tests/integration.rs` - ~2 test cases
- `tests/cli_integration.rs` - Integration tests

### Phase 4: Update Documentation

**4.1 Workflow files**
- [ ] `workflows/execution.yaml` - Update review prompt instructions
  ```yaml
  # Before
  - Review complete, issues need fixing → `hegel next '{"needs_refactor": true}'`

  # After
  - Review complete, issues need fixing → `hegel next needs_refactor`
  ```

**4.2 Documentation files**
- [ ] `README.md` - Update all claim examples
- [ ] `HEGEL_CLAUDE.md` - Update command reference and examples
- [ ] `CLAUDE.md` - Update any claim examples if present

**4.3 Guide files**
- [ ] Search `guides/` for any claim examples (unlikely but check)

### Phase 5: Backward Compatibility (Optional)

**Decision point:** Support both formats during transition?

**Option A: Hard cutover** (recommended)
- Clean break, simpler implementation
- Version bump (breaking change)
- Clear migration path

**Option B: Temporary dual support**
```rust
Self::Custom(input) => {
    // Try parsing as JSON first (old format)
    if let Ok(map) = serde_json::from_str::<HashMap<String, bool>>(input) {
        Ok(map.keys().cloned().collect())
    } else {
        // Treat as simple string (new format)
        Ok(HashSet::from([input.to_string()]))
    }
}
```

**Recommendation**: Option A (hard cutover). This is pre-1.0 software, clean breaks are acceptable.

---

## Implementation Order

1. **Core types** (`claims.rs`, `engine/mod.rs`, `transitions.rs`) - Foundation
2. **CLI parsing** (`main.rs`, `commands/workflow/mod.rs`) - User interface
3. **Tests** (all test files) - Validation
4. **Documentation** (workflows, README, guides) - User-facing
5. **Final verification** - End-to-end testing

---

## Testing Strategy

**Unit tests:**
- [ ] ClaimAlias::to_claims() produces correct HashSet
- [ ] Engine transition evaluation with HashSet
- [ ] Empty claims (None) still uses implicit claim

**Integration tests:**
- [ ] `hegel next` (implicit claim) works
- [ ] `hegel next spec_complete` (explicit claim) works
- [ ] `hegel next invalid_claim` stays at current node
- [ ] Full workflow cycle with mixed implicit/explicit claims

**Manual testing:**
- [ ] Test discovery workflow (implicit claims only)
- [ ] Test execution workflow (needs_refactor explicit claim)
- [ ] Test research workflow (continue_study, continue_research claims)
- [ ] Verify error messages are helpful

---

## Risk Assessment

**Low risk:**
- Type system will catch most issues at compile time
- HashSet operations are well-tested in Rust stdlib
- Simpler code → fewer bugs

**Medium risk:**
- Test updates are tedious but mechanical
- Missing a test case could leave behavior untested

**Mitigation:**
- Run full test suite after each phase
- Manual testing of each workflow
- Search codebase for all claim-related strings before considering done

---

## Success Criteria

- [ ] All tests passing (415 unit + integration tests)
- [ ] Zero compiler warnings
- [ ] All workflows tested manually (discovery, execution, research, minimal, init-greenfield, init-retrofit)
- [ ] Documentation updated and accurate
- [ ] CLI help text reflects new format
- [ ] Error messages are clear and helpful

---

## Estimated Effort

**Core implementation**: 2-3 hours
**Test updates**: 1-2 hours
**Documentation**: 1 hour
**Testing & validation**: 1 hour

**Total**: 5-8 hours

---

## Follow-up Considerations

**After refactor:**
1. Consider supporting multiple claims: `hegel next claim1 claim2`
2. Consider claim validation (error if claim doesn't match any transition)
3. Consider `hegel next --list` to show available transitions from current node

**Documentation improvements:**
1. Add troubleshooting section for "stayed at current node" scenarios
2. Add workflow transition diagrams to README
3. Consider interactive workflow explorer (`hegel workflows --interactive`)

---

## Related Files

**Core claim handling:**
- `src/commands/workflow/claims.rs` - Claim alias types
- `src/engine/mod.rs` - Transition evaluation
- `src/commands/workflow/transitions.rs` - Transition execution
- `src/commands/workflow/mod.rs` - Workflow commands

**Test files:**
- `src/commands/workflow/tests/*.rs` - Unit tests
- `tests/cli_integration.rs` - Integration tests
- `src/test_helpers/workflow.rs` - Test utilities

**Documentation:**
- `workflows/*.yaml` - Workflow definitions (6 files)
- `README.md` - User guide
- `HEGEL_CLAUDE.md` - AI assistant guide
