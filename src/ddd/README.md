# src/ddd/

DDD artifact management: parsing, validating, and scanning the `.ddd/` directory structure (feat/refactor/report/toy artifacts).

## Purpose

Reads the on-disk `.ddd/` layout into typed artifacts, validates their naming (`YYYYMMDD[-N]-name`), and reports malformed entries plus same-day index collisions. Consumed by `hegel new`, `hegel md`, and `hegel doctor`.

## Structure

```
ddd/
├── mod.rs              Module wiring + public re-exports; parse_ddd_structure() entry point
├── types.rs            Artifact structs/enums (Feat/Refactor/Report/Toy, DddArtifact, ValidationIssue) + path/name helpers
├── parse.rs            Name parsing (parse_feat_name, parse_single_file_name, parse_toy_name) + format validation
├── scan.rs             Directory scanning into a DddScanResult (generic scan_artifact_dir over .ddd/ subdirs)
├── index.rs            Same-day index disambiguation (orders artifacts by git-creation timestamp)
└── tests.rs            Unit tests for parsing, validation, and scanning
```

## Key Patterns

**Re-export facade**: `mod.rs` keeps submodules private and re-exports the public surface, so callers use `crate::ddd::{DddArtifact, parse_ddd_structure, ...}` regardless of internal layout. Parse/validate helpers used only internally stay module-private.
