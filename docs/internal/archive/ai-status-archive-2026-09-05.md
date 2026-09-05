# ai-status-archive-2026-09-05

## 2026-06-11 - Datasource browse TUI chrome split
- State: Done
- Scope: rust/src/commands/datasource/browse/render.rs; rust/src/commands/datasource/browse/render_chrome.rs; rust/src/commands/datasource/mod.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved datasource browse footer controls and search prompt rendering into a dedicated chrome module while preserving the existing renderer tests.
- Result: `datasource/browse/render.rs` dropped below the architecture warning threshold; datasource browse rendering remains behavior-compatible and the remaining architecture warnings are datasource browse support, review_contract, and shared browser session.
