# ai-status-archive-2026-05-02

## 2026-04-27 - Guard Git Sync dashboard live apply boundaries
- State: Done
- Scope: Rust dashboard browse local-mode routing, sync apply-intent/live-apply regressions, focused tests, and TODO trace. Public JSON, generated docs, Python implementation, and Git repository/PR automation are out of scope.
- Baseline: Sync live apply already blocked file-provisioned and Git Sync-owned dashboards, but workspace-backed dashboard browse trees did not share the same local-mode detection as explicit `--input-dir` local browse trees.
- Current Update: Centralized dashboard browse local-source detection so `--workspace` Git Sync review trees use read-only local mode, and added sync regressions proving Git Sync dashboard ownership survives apply-intent handoff and blocks live transport.
- Result: Focused browse, sync apply-intent, live-apply, and reusable-output tests pass.

## 2026-04-27 - Guard dashboard permissions as adjacent evidence
- State: Done
- Scope: Rust dashboard permission-artifact rejection, dashboard/raw-to-prompt/review regressions, sync/access workspace boundary tests, and TODO trace. Permission restore/apply behavior, public JSON changes, generated docs, and Python implementation are out of scope.
- Baseline: Directory-based dashboard flows skipped `permissions.json`, but single-object dashboard flows could still treat dashboard permission artifacts as dashboard JSON.
- Current Update: Rejected dashboard permission bundle/export artifacts in the shared dashboard object extractor, wired the inventory regression module into the Rust suite, and added sync/access tests proving permission bundles stay out of dashboard source and access-bundle collection.
- Result: Focused dashboard, raw-to-prompt, sync bundle, and access plan tests pass.

## 2026-04-27 - Move dashboard prompt transform boundary
- State: Done
- Scope: Rust dashboard prompt transform module layout, facade re-exports, focused prompt/export tests, full Rust validation, and TODO trace. Public CLI/docs, generated docs, Python implementation, and behavior changes are out of scope.
- Baseline: The dashboard re-layering inventory identified root-level `prompt*.rs` files as a shared prompt-lane transform boundary used by live export and offline raw-to-prompt.
- Current Update: Moved the prompt transform and helper files under `rust/src/commands/dashboard/export_prompt/`, kept `commands/dashboard/mod.rs` as the public facade, and rewired direct consumers plus test support to the new module.
- Result: Focused raw-to-prompt, export prompt, inventory, library-panel, and export-diff tests pass; full Rust validation is run for the commit.
