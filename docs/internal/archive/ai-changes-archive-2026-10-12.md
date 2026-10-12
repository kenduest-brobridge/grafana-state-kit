# AI changes archive — 2026-10-12

## 2026-06-11 - Status overview TUI support split
- Summary: Split status overview TUI runtime and focused tests out of the state/search module while preserving existing interactive behavior.
- Tests: cargo test --quiet overview_tui --features tui; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `status/overview/tui.rs` is now under the architecture warning threshold, leaving status overview with the same facade/runtime/tests shape as the newer status TUI module. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low; this is a structural split of existing code with focused TUI tests covering search, Project Home handoff, initial item focus, and render output.
- Follow-up: Continue reducing the remaining TUI/shared warning-threshold files: datasource browse render/support and shared browser session.
