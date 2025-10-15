# AST-grep Pattern Library

Pattern library for systematic code refactoring using `hegel astq` (wrapper around ast-grep).

## ⚠️ Important Limitations

**ast-grep cannot match code inside Rust macro invocations:**
- ✅ **Works**: `let x = "text".cyan();` - simple statement context
- ❌ **Doesn't work**: `println!("{}", "text".cyan())` - inside macro
- ❌ **Doesn't work**: `format!(...).cyan()` - macro result with method

**Root cause**: Macros aren't fully expanded in the AST, so patterns can't reach inside them.

**For comprehensive searches, use identifier patterns instead:**
```bash
# Find ALL .cyan() calls (regardless of receiver complexity)
hegel astq -l rust -p 'cyan' src/

# Count all occurrences
hegel astq -l rust -p 'cyan' src/ | wc -l
```

## Quick Start

```bash
# Find all occurrences (recommended - catches everything)
hegel astq -l rust -p 'cyan' src/

# Find SIMPLE cases only (limited - misses macros/chains)
hegel astq -l rust -p '$X.cyan()' src/

# Preview rewrite (only affects simple cases)
hegel astq -l rust -p '$X.cyan()' -r 'Theme::highlight($X)' src/file.rs
```

## Pattern Syntax Basics

- `$X` - Metavariable (matches SIMPLE expressions only: literals, identifiers)
- `$UPPERCASE` - Named metavariable
- Pattern matches AST structure, not text

### Examples (with limitations noted)

| Pattern | Matches | Doesn't Match | Description |
|---------|---------|---------------|-------------|
| `$X.cyan()` | `let x = "text".cyan();` | `println!("{}", x.cyan())` | Works in statement context, not inside macros |
| `$X.bold().cyan()` | `let y = var.bold().cyan();` | `format!("{}", x).bold().cyan()` | Works outside macros |
| `let $V = $E;` | `let x = 5;` | N/A | Variable declarations (not affected by macro limitation) |

## Pattern Libraries

- `rust_colored.yaml` - colored::Colorize method replacements

## Pattern Discovery Workflow

1. **Find example** in code you want to refactor
2. **Debug AST structure**:
   ```bash
   hegel astq -l rust --debug-query=ast -p 'your_example' file.rs
   ```
3. **Generalize** with metavariables (`$X`, `$Y`, etc.)
4. **Test** against known matches
5. **Document** in pattern library

## Application Order

When applying multiple patterns, order matters:

1. **Most specific first** (e.g., `.bold().cyan()`)
2. **Then simpler** (e.g., `.cyan()`)
3. **Generic last** (e.g., `.bold()`)

This prevents partial replacements that break code.

## Real Example: Color Refactoring

### Step 1: Discover all occurrences
```bash
# Find ALL .cyan() calls (including complex cases)
hegel astq -l rust -p 'cyan' src/commands/analyze/sections.rs

# Output shows:
# - "text".cyan()              ← simple (auto-refactorable)
# - format!(...).cyan()        ← complex (manual refactor needed)
# - x.to_string().cyan()       ← complex (manual refactor needed)
```

### Step 2: Auto-refactor simple cases only
```bash
# Apply pattern (only affects simple receivers)
hegel astq -l rust -p '$X.cyan()' -r 'Theme::highlight($X)' -U src/file.rs
```

### Step 3: Manual refactor complex cases
```bash
# For format!(...).cyan() → Theme::highlight(format!(...))
# For x.to_string().cyan() → Theme::highlight(x.to_string())
# Edit directly in your editor
```

### Step 4: Verify
```bash
cargo test
```

## Tips

- **Use identifier search first** (`'cyan'`) to find ALL occurrences
- **Macro boundaries block pattern matching** - code inside `println!()`, `format!()`, etc. can't be matched
- **Always preview** before applying (`-r` shows diff)
- **Backup files** before bulk changes
- **Run tests** after each pattern application
- **Commit incrementally** (one pattern type at a time)
- **Expect manual work** for any code inside macros (most Rust code)

## Learning Resources

- Vendored docs: `vendor/ast-grep/README.md`
- Tree-sitter grammar: https://github.com/tree-sitter/tree-sitter-rust
- Our pattern library: `docs/astq_patterns/rust_colored.yaml`

## Future: Subagent Integration

When ast-grep subagent support is added (Phase 2.2):
- Subagent loads this library as few-shot examples
- Iterative pattern refinement with feedback
- Pattern library grows automatically
