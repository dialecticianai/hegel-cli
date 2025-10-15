# AST-grep Pattern Library

Pattern library for systematic code refactoring using `hegel astq` (wrapper around ast-grep).

## Quick Start

```bash
# Find all .cyan() calls
hegel astq -l rust -p '$X.cyan()' src/

# Preview rewrite
hegel astq -l rust -p '$X.cyan()' -r 'Theme::highlight($X)' src/commands/analyze/sections.rs

# Count matches
hegel astq -l rust -p '$X.cyan()' src/ | wc -l
```

## Pattern Syntax Basics

- `$X` - Metavariable (matches any expression)
- `$UPPERCASE` - Named metavariable
- Pattern matches AST structure, not text

### Examples

| Pattern | Matches | Description |
|---------|---------|-------------|
| `$X.cyan()` | `"text".cyan()`, `var.cyan()`, `f().cyan()` | Any expression with .cyan() |
| `$X.bold().cyan()` | `"text".bold().cyan()` | Chained methods |
| `let $V = $E;` | `let x = 5;`, `let name = get_name();` | Variable declarations |

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

### Step 1: Apply chained patterns first
```bash
hegel astq -l rust -p '$X.bold().cyan()' -r 'Theme::header($X)' src/commands/analyze/sections.rs
```

### Step 2: Then simple colors
```bash
hegel astq -l rust -p '$X.cyan()' -r 'Theme::highlight($X)' src/commands/analyze/sections.rs
hegel astq -l rust -p '$X.green()' -r 'Theme::success($X)' src/commands/analyze/sections.rs
hegel astq -l rust -p '$X.yellow()' -r 'Theme::warning($X)' src/commands/analyze/sections.rs
```

### Step 3: Verify
```bash
cargo test
```

## Tips

- **Always preview** before applying (`-r` shows diff)
- **Backup files** before bulk changes
- **Run tests** after each pattern application
- **Commit incrementally** (one pattern type at a time)

## Learning Resources

- ast-grep playground: https://ast-grep.github.io/playground.html
- Vendored docs: `vendor/ast-grep/README.md`
- Tree-sitter grammar: https://github.com/tree-sitter/tree-sitter-rust

## Future: Subagent Integration

When ast-grep subagent support is added (Phase 2.2):
- Subagent loads this library as few-shot examples
- Iterative pattern refinement with feedback
- Pattern library grows automatically
