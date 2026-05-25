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

## 2026-05-25 - Shared access browser detail lines
- Summary: Added a shared read-only browser detail info-line helper and routed access user/team browse detail rows through it instead of duplicate local renderers.
- Tests: cargo test --quiet browser_detail_info_line_formats_label_value_with_fallback; cargo test --quiet access_user_browse; cargo test --quiet interactive_browser; escalated cargo test --quiet access after sandbox denied local mock-server binding; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Access user/team browse detail rows now share browser-level 18-column Label: value rendering and blank-value fallback while preserving existing detail output. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change replaces equivalent local detail-line renderers with a shared helper and focused tests cover fallback formatting plus access browse render paths.
- Follow-up: Continue scanning specialized TUI browsers for remaining local detail/review row renderers that match shared browser projection helpers.

## 2026-05-25 - Shared datasource browser review lines
- Summary: Added a shared read-only browser review info-line projection and routed datasource browse review-pane evidence rows through it instead of a local renderer.
- Tests: cargo test --quiet shared_browser_review_lines_format_datasource_review_rows; cargo test --quiet review_pane_formats_local_review_evidence_without_secret_values; cargo test --quiet datasource_browse; cargo test --quiet interactive_browser; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Datasource browse review panes now share browser-level 24-column Label: value evidence rendering and blocker/required highlighting while preserving existing review output. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change replaces an equivalent local renderer with a shared helper, and focused tests cover shared review row rendering plus existing datasource review evidence output.
- Follow-up: Look for other TUI review panes with the same review evidence label/value shape before adding new local renderers.

## 2026-05-25 - Shared dashboard browser info lines
- Summary: Extended the shared read-only browser info-line projection with filter and special-row hooks, then routed dashboard browse detail rendering through it while preserving dashboard-specific hidden action lines and the live-details badge.
- Tests: cargo test --quiet shared_browser_info_lines_preserve_dashboard_filters_and_live_badge; cargo test --quiet dashboard_browse; cargo test --quiet datasource_browse; cargo test --quiet interactive_browser; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Dashboard browse and datasource browse now share the same styled Label: value detail projection path while dashboard browse keeps its action filtering and LIVE DETAILS marker. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change replaces an equivalent local renderer with a shared helper variant, and focused tests cover dashboard filtering/badge behavior plus the datasource wrapper path.
- Follow-up: Continue migrating compatible TUI review/detail renderers onto shared browser projection helpers where their row filtering and special cases can stay explicit.

## 2026-05-25 - Shared datasource browser info lines
- Summary: Added a shared read-only browser styled info-line projection and routed datasource browse detail rendering through it instead of a local Label/value renderer.
- Tests: cargo test --quiet shared_browser_info_lines_format_datasource_detail_rows; cargo test --quiet datasource_browse; cargo test --quiet interactive_browser; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Datasource browse detail panes now share browser-level 18-column Label: value styling while preserving existing datasource detail output. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change replaces an equivalent local renderer with a shared helper, and focused render/browser tests cover the projected output path.
- Follow-up: Continue migrating compatible dashboard browse detail rendering onto the same shared info-line helper, preserving its dashboard-specific filters and live-details badge handling.

## 2026-05-25 - Shared datasource browser detail facts
- Summary: Added a shared read-only browser fallback fact formatter and routed datasource browse detail rows through shared browser fact helpers instead of local Label/value formatting.
- Tests: cargo test --quiet browser_detail_fallback_fact_trims_or_uses_fallback; cargo test --quiet detail_lines_use_shared_browser_fact_formatting; cargo test --quiet detail_lines_sort_json_data_and_secure_json_field_keys; cargo test --quiet datasource_browse; cargo test --quiet interactive_browser; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; make quality-ai-workflow; git diff --check
- Impact: Datasource browse detail rows now share browser-level Label: value formatting and blank-value fallback trimming while preserving existing detail output. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change replaces local detail string formatting with equivalent shared helpers and focused tests cover fallback trimming plus datasource detail output.
- Follow-up: Continue migrating compatible datasource/status/dashboard TUI detail projections onto shared browser fact and section helpers where the data shape already matches.

## 2026-05-25 - Shared browser aligned detail facts
- Summary: Added a shared read-only browser aligned detail fact formatter and routed dashboard inspect workbench full-detail rows through it instead of a local formatter.
- Tests: cargo test --quiet browser_detail_aligned_fact_formats_full_detail_rows; cargo test --quiet inspect_workbench; cargo test --quiet full_detail_viewer; cargo test --quiet interactive_browser; cargo test --quiet dashboard_inspect; cargo fmt --check; cargo check --quiet --no-default-features --all-targets; git diff --check
- Impact: Dashboard inspect workbench full-detail rows now share browser-level aligned Label: value formatting while existing viewer output remains stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change replaces a local formatter with an equivalent shared helper and focused tests cover both the helper and full-detail viewer path.
- Follow-up: Continue migrating compatible TUI detail sections and review surfaces onto shared browser/review helpers where the data shape already matches.

## 2026-05-25 - Shared browser detail facts
- Summary: Added a shared read-only browser detail fact formatter and routed dashboard inspect workbench item detail rows through it instead of a local formatter.
- Tests: cargo test --quiet browser_detail_fact_formats_label_value_rows; cargo test --quiet inspect_workbench; cargo test --quiet dashboard_inspect; cargo test --quiet interactive_browser; escalated cargo test --quiet dashboard after sandbox denied local mock server binding; cargo fmt --check; cargo check --quiet --no-default-features --all-targets; git diff --check
- Impact: Dashboard inspect workbench TUI detail rows now share browser-level Label: value formatting while existing item output remains stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change replaces a local formatter with an equivalent shared helper and is covered by focused browser and dashboard tests.
- Follow-up: Continue migrating compatible TUI detail sections and review surfaces onto shared browser/review helpers where the data shape already matches.

## 2026-05-25 - Shared browser detail sections
- Summary: Added a shared read-only browser detail-section helper for Heading none/body formatting and routed dashboard topology inbound/outbound edge summaries through it.
- Tests: cargo test --quiet append_browser_detail_section_formats_empty_and_populated_sections; cargo test --quiet build_topology_browser_items_sorts_by_kind_then_label_and_summarizes_edges; cargo test --quiet dashboard_topology; cargo test --quiet topology_tui; cargo test --quiet interactive_browser; cargo fmt --check; cargo check --quiet --no-default-features --all-targets; git diff --check
- Impact: Dashboard topology TUI detail rows now share browser-level detail section shaping while keeping existing edge summary output stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The helper preserves existing labels and body lines, with focused tests for empty and populated sections plus topology browser output.
- Follow-up: Continue migrating compatible TUI detail sections and review surfaces onto shared browser/review helpers where the data shape already matches.
