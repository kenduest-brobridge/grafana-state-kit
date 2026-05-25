# ai-changes.md

Current AI change log only.

- Older detailed history moved to [`archive/ai-changes-archive-2026-03-24.md`](docs/internal/archive/ai-changes-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-changes-archive-2026-03-27.md`](docs/internal/archive/ai-changes-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-changes-archive-2026-03-28.md`](docs/internal/archive/ai-changes-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-changes-archive-2026-03-31.md`](docs/internal/archive/ai-changes-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-changes-archive-2026-04-12.md`](docs/internal/archive/ai-changes-archive-2026-04-12.md).
- Keep this file limited to the latest active architecture and maintenance changes.
- Older entries moved to [`ai-changes-archive-2026-04-13.md`](docs/internal/archive/ai-changes-archive-2026-04-13.md).
- Older entries moved to [`ai-changes-archive-2026-04-14.md`](docs/internal/archive/ai-changes-archive-2026-04-14.md).
- Older entries moved to [`ai-changes-archive-2026-04-15.md`](docs/internal/archive/ai-changes-archive-2026-04-15.md).
- Older entries moved to [`ai-changes-archive-2026-04-16.md`](docs/internal/archive/ai-changes-archive-2026-04-16.md).
- Older entries moved to [`ai-changes-archive-2026-04-17.md`](docs/internal/archive/ai-changes-archive-2026-04-17.md).
- Older entries moved to [`ai-changes-archive-2026-04-18.md`](docs/internal/archive/ai-changes-archive-2026-04-18.md).
- Older entries moved to [`ai-changes-archive-2026-04-19.md`](docs/internal/archive/ai-changes-archive-2026-04-19.md).
- Older entries moved to [`ai-changes-archive-2026-04-20.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-20.md).
- Older entries moved to [`ai-changes-archive-2026-04-26.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-26.md).
- Older entries moved to [`ai-changes-archive-2026-04-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-27.md).
- Older entries moved to [`ai-changes-archive-2026-04-28.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-28.md).
- Older entries moved to [`ai-changes-archive-2026-05-02.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-02.md).
- Older entries moved to [`ai-changes-archive-2026-05-14.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-14.md).
- Older entries moved to [`ai-changes-archive-2026-05-16.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-16.md).
- Older entries moved to [`ai-changes-archive-2026-05-25.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-25.md).

## 2026-05-25 - Shared status overview detail facts
- Summary: Routed status overview section detail strings through shared browser_detail_fact instead of a local detail_line formatter and added a regression against reintroducing the wrapper.
- Tests: cargo test --quiet status_overview_section_rows_do_not_wrap_shared_detail_facts; cargo test --quiet status_overview; cargo test --quiet status (outside sandbox for local mock-server coverage); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Status overview section item details now share the browser-level Label: value fact formatter, reducing helper drift candidates from three to two while preserving existing detail strings. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The shared helper produces the same Label: value string and focused plus broader status tests cover the overview path.
- Follow-up: Use the remaining helper-drift candidates to decide whether dashboard build_info_lines and datasource build_review_lines are domain-specific or worth extracting before declaring the TUI design work complete.

## 2026-05-25 - TUI inventory helper-drift report
- Summary: Extended the manual TUI inventory report with helper-drift candidate detection for remaining local TUI/detail/review helper functions and added focused unittest coverage.
- Tests: python3 -m unittest -v scripts.test_tui_inventory_report; python3 scripts/tui_inventory_report.py; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow; git diff --check
- Impact: Maintainers can now use the TUI inventory helper to see remaining local helper candidates alongside the surface inventory, making future completion audits evidence-based. Public CLI paths, help text, command contracts, generated docs, Rust runtime behavior, Python package behavior, and package metadata are unchanged.
- Rollback/Risk: Low. This is a manual maintainer report enhancement with focused unittest coverage and no CI gate change.
- Follow-up: Use the helper-drift candidate list to decide whether remaining build_info_lines/build_review_lines/detail_line helpers are domain-specific or worth extracting before declaring the TUI design work complete.

## 2026-05-25 - Shared datasource browse fixed control lines
- Summary: Added a shared fixed-body-width shell control-line helper and routed datasource browse footer rows through it instead of a local control_line renderer.
- Tests: cargo test --quiet datasource_browse_render_does_not_wrap_control_line_shell_rows; cargo test --quiet control_lines_surface_consistent_focus_cycle_and_exit_labels; cargo test --quiet datasource_browse; cargo test --quiet interactive_browser; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Datasource browse footer rows now share shell-level fixed-width control-line rendering while preserving existing 14-column label spacing. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The shared helper preserves the previous datasource footer layout and focused datasource browse tests cover the control rows.
- Follow-up: Continue auditing remaining TUI control/detail helpers and then run a completion audit against the roadmap before declaring the TUI design work complete.

## 2026-05-25 - Shared browse boxed shell spans
- Summary: Added shared tui_shell::boxed and routed dashboard/datasource browse boxed helper labels through it instead of local plain_boxed helpers.
- Tests: cargo test --quiet dashboard_browse_render_detail_does_not_wrap_boxed_shell_span; cargo test --quiet datasource_browse_render_does_not_wrap_boxed_shell_span; cargo test --quiet dashboard_browse; cargo test --quiet datasource_browse; cargo test --quiet interactive_browser; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Dashboard and datasource browse boxed helper labels now share the shell-level boxed span primitive while preserving existing fallback behavior and rendered detail output. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change moves equivalent white-on-background span rendering into shared shell code; datasource keeps blank fallback at the call site and focused browse tests cover both render paths.
- Follow-up: Continue auditing remaining TUI control-line adapters and detail/review projection helpers before broader workbench abstraction.

## 2026-05-25 - Shared browse muted shell spans
- Summary: Added shared tui_shell::muted and routed dashboard/datasource browse muted labels through it instead of identical local helpers.
- Tests: cargo test --quiet dashboard_browse_render_detail_does_not_wrap_muted_shell_span; cargo test --quiet datasource_browse_render_does_not_wrap_muted_shell_span; cargo test --quiet dashboard_browse; cargo test --quiet datasource_browse; cargo test --quiet interactive_browser; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Dashboard and datasource browse muted labels now share the shell-level muted span primitive while existing detail/review output remains stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change moves equivalent gray-span helpers into shared shell code and focused browse tests cover both render paths.
- Follow-up: Continue auditing remaining TUI control-line adapters and detail/review projection helpers before broader workbench abstraction.

## 2026-05-25 - Shared access browse detail info lines
- Summary: Routed access user/team browse fact rows directly through shared browser_detail_info_line and added regressions against reintroducing local detail_line delegate wrappers.
- Tests: cargo test --quiet user_browse_render_does_not_wrap_shared_detail_info_lines; cargo test --quiet team_browse_render_does_not_wrap_shared_detail_info_lines; cargo test --quiet user_browse; cargo test --quiet team_browse; cargo test --quiet access (rerun outside sandbox after local mock-server permission denial); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Access user/team browse fact rows now share browser-level detail info-line rendering directly while existing rendered detail output remains stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change removes equivalent local delegate wrappers, keeps the same fallback value, and focused plus broader access tests cover the render/state paths.
- Follow-up: Continue auditing remaining TUI detail/review projection helpers and datasource browse control-line adapters before broader workbench abstraction.

## 2026-05-25 - Shared inspect workbench shell controls
- Summary: Routed dashboard inspect workbench shell control/key/plain spans directly through shared tui_shell helpers and added a regression against reintroducing local delegate wrappers.
- Tests: cargo test --quiet inspect_workbench_render_helpers_do_not_wrap_shared_shell_spans; cargo test --quiet inspect_workbench; cargo test --quiet full_detail_viewer; cargo test --quiet dashboard_inspect; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Dashboard inspect workbench shell controls now share tui_shell primitives directly while existing render output remains stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change removes equivalent local delegate wrappers and focused inspect workbench tests cover the render paths.
- Follow-up: Continue auditing remaining TUI control-line adapters and detail/review projection helpers before broader workbench abstraction.

## 2026-05-25 - Shared access browse shell spans
- Summary: Routed access user/team browse action-row key-chip/plain spans directly through shared tui_shell helpers instead of local delegate wrappers.
- Tests: cargo test --quiet user_browse; cargo test --quiet team_browse; cargo test --quiet access; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Access user/team browse action rows now share shell-level key-chip/plain rendering without changing public CLI paths, help text, command contracts, generated docs, Python behavior, or package metadata.
- Rollback/Risk: Low. The change replaces local delegate wrappers with direct calls to equivalent shared shell helpers and focused access browse tests cover the render/state paths.
- Follow-up: Continue auditing remaining TUI render wrappers and control-line adapters before considering broader workbench abstraction.

## 2026-05-25 - Shared status tui shell spans
- Summary: Routed status TUI key-chip/plain spans through shared tui_shell helpers instead of local duplicates.
- Tests: cargo test --quiet status_tui; cargo test --quiet --test project_status_tui_rust_tests; cargo test --quiet project_status (rerun outside sandbox after local mock-server permission denial); cargo test --quiet status (outside sandbox for local mock-server coverage); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Status TUI header rows now share shell-level key-chip/plain span rendering while existing status output remains stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change replaces equivalent local span helpers with shared shell helpers and focused status tests cover the render paths.
- Follow-up: Continue auditing remaining TUI render helpers for shared-shell primitives before broader workbench abstraction.

## 2026-05-25 - Shared inspect viewer wrapped detail lines
- Summary: Added a shared read-only browser wrapped labeled-detail helper and routed dashboard inspect workbench full-detail viewer label/value wrapping through it while keeping logical row mapping local.
- Tests: cargo test --quiet browser_wrapped_labeled_detail_lines_preserve_prefix_width; cargo test --quiet full_detail_viewer; cargo test --quiet inspect_workbench; cargo test --quiet interactive_browser; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Dashboard inspect workbench full-detail viewer metadata rows now share browser-level aligned label/value wrapping while existing viewer output and logical row mapping remain stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change moves an equivalent wrapped label/value renderer into the shared browser helper layer and focused tests cover the helper plus viewer/workbench paths.
- Follow-up: Continue auditing remaining TUI render helpers for shared-shell primitives before introducing broader workbench abstractions.
