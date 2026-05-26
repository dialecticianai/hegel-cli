# src/commands/markdown/

`hegel md` — renders a tree of markdown files with DDD artifact awareness (SPEC/PLAN indicators, ephemeral tags, validation warnings).

## Purpose

Walks the working directory for markdown files, classifies them as DDD artifacts or regular docs, and presents them as a terminal tree or JSON. Splits the pipeline into focused stages: scan → tree model → render/json.

## Structure

```
markdown/
├── mod.rs              Orchestrator: run_markdown, MarkdownArgs, categorize files into DDD vs regular
├── scan.rs             Filesystem walk (gitignore-aware), classification (FileCategory), MarkdownFile model
├── tree.rs             TreeNode model, tree construction, DDD-metadata attachment, issue detection
├── render.rs           Terminal tree rendering (box-drawing output, inline artifact indicators)
└── json.rs             Machine-readable output (MarkdownTree/FileEntry serialization)
```

## Flow

1. **scan** produces a flat `Vec<MarkdownFile>` with category + line/size metadata
2. **mod** splits them into DDD vs regular per the `--ddd`/`--no-ddd` flags
3. **tree** builds the directory tree and attaches per-artifact file metadata from the DDD scan
4. **render** (default) or **json** (`--json`) emits the result
