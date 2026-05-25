# ai-status.md

Current AI-maintained status only.

- Older trace history moved to [`archive/ai-status-archive-2026-03-24.md`](docs/internal/archive/ai-status-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-status-archive-2026-03-27.md`](docs/internal/archive/ai-status-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-status-archive-2026-03-28.md`](docs/internal/archive/ai-status-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-status-archive-2026-03-31.md`](docs/internal/archive/ai-status-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-status-archive-2026-04-12.md`](docs/internal/archive/ai-status-archive-2026-04-12.md).
- Keep this file short and current. Additive historical detail belongs in `docs/internal/archive/`.
- Older entries moved to [`ai-status-archive-2026-04-13.md`](docs/internal/archive/ai-status-archive-2026-04-13.md).
- Older entries moved to [`ai-status-archive-2026-04-14.md`](docs/internal/archive/ai-status-archive-2026-04-14.md).
- Older entries moved to [`ai-status-archive-2026-04-15.md`](docs/internal/archive/ai-status-archive-2026-04-15.md).
- Older entries moved to [`ai-status-archive-2026-04-16.md`](docs/internal/archive/ai-status-archive-2026-04-16.md).
- Older entries moved to [`ai-status-archive-2026-04-17.md`](docs/internal/archive/ai-status-archive-2026-04-17.md).
- Older entries moved to [`ai-status-archive-2026-04-18.md`](docs/internal/archive/ai-status-archive-2026-04-18.md).
- Older entries moved to [`ai-status-archive-2026-04-19.md`](docs/internal/archive/ai-status-archive-2026-04-19.md).
- Older entries moved to [`ai-status-archive-2026-04-20.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-04-20.md).
- Older entries moved to [`ai-status-archive-2026-04-26.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-04-26.md).
- Older entries moved to [`ai-status-archive-2026-04-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-04-27.md).
- Older entries moved to [`ai-status-archive-2026-04-28.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-04-28.md).
- Older entries moved to [`ai-status-archive-2026-05-02.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-05-02.md).
- Older entries moved to [`ai-status-archive-2026-05-14.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-05-14.md).
- Older entries moved to [`ai-status-archive-2026-05-16.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-05-16.md).
- Older entries moved to [`ai-status-archive-2026-05-25.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-05-25.md).

## 2026-05-25 - Shared datasource artifact detail projection
- State: Done
- Scope: rust/src/commands/datasource/browse/support.rs; rust/src/commands/datasource/inspect/export.rs; rust/src/commands/snapshot/review/browser.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Added a shared datasource browser detail projection for local artifact records and routed datasource local inspect plus snapshot datasource review rows through it before appending review evidence.
- Result: Datasource local list and snapshot datasource review browser rows now share identity fact shaping for Name, UID, Type, Org, URL, Access, and Default while preserving existing shared review evidence projection and public CLI/doc surfaces.

## 2026-05-25 - Cleared dashboard browse helper drift
- State: Done
- Scope: rust/src/commands/dashboard/browse/render_detail.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Renamed the dashboard browse fact-line builder to describe its dashboard-specific filtering and live-details badge behavior instead of looking like a generic build_info_lines helper.
- Result: The manual TUI inventory report now has zero helper-drift candidates while dashboard browse fact rendering keeps the existing shared browser_detail_info_lines_with output path and behavior.

## 2026-05-25 - Shared datasource review empty lines
- State: Done
- Scope: rust/src/common/browser/session.rs; rust/src/commands/datasource/browse/render.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved datasource browse REVIEW empty-state line formatting into a shared browser helper and renamed the datasource review panel builder to avoid a generic helper-drift wrapper.
- Result: Datasource browse review empty states now share browser-level REVIEW message formatting while review evidence rendering and public CLI/doc surfaces remain unchanged.

## 2026-05-25 - Shared status overview detail facts
- State: Done
- Scope: rust/src/commands/status/overview/section_rows.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Routed status overview section detail strings through shared browser_detail_fact instead of a local detail_line formatter and added a regression against reintroducing the wrapper.
- Result: Status overview section item details now share the browser-level Label: value fact formatter, reducing helper drift candidates from three to two while preserving existing detail strings. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.

## 2026-05-25 - TUI inventory helper-drift report
- State: Done
- Scope: scripts/tui_inventory_report.py; scripts/test_tui_inventory_report.py; docs/internal/tui-architecture-roadmap.md
- Current Update: Extended the manual TUI inventory report with helper-drift candidate detection for remaining local TUI/detail/review helper functions and added focused unittest coverage.
- Result: Maintainers can now use the TUI inventory helper to see remaining local helper candidates alongside the surface inventory, making future completion audits evidence-based. Public CLI paths, help text, command contracts, generated docs, Rust runtime behavior, Python package behavior, and package metadata are unchanged.

## 2026-05-25 - Shared datasource browse fixed control lines
- State: Done
- Scope: rust/src/common/tui/shell.rs; rust/src/commands/datasource/browse/render.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Added a shared fixed-body-width shell control-line helper and routed datasource browse footer rows through it instead of a local control_line renderer.
- Result: Datasource browse footer rows now share shell-level fixed-width control-line rendering while preserving existing 14-column label spacing. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
