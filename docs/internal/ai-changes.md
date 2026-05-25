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

## 2026-05-25 - Shared browser detail sections
- Summary: Added a shared read-only browser detail-section helper for Heading none/body formatting and routed dashboard topology inbound/outbound edge summaries through it.
- Tests: cargo test --quiet append_browser_detail_section_formats_empty_and_populated_sections; cargo test --quiet build_topology_browser_items_sorts_by_kind_then_label_and_summarizes_edges; cargo test --quiet dashboard_topology; cargo test --quiet topology_tui; cargo test --quiet interactive_browser; cargo fmt --check; cargo check --quiet --no-default-features --all-targets; git diff --check
- Impact: Dashboard topology TUI detail rows now share browser-level detail section shaping while keeping existing edge summary output stable. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The helper preserves existing labels and body lines, with focused tests for empty and populated sections plus topology browser output.
- Follow-up: Continue migrating compatible TUI detail sections and review surfaces onto shared browser/review helpers where the data shape already matches.

## 2026-05-25 - Shared TUI review evidence sections
- Summary: Added a shared review-contract helper for appending Review evidence sections and routed access plan, datasource local, and snapshot datasource browser details through it.
- Tests: cargo test --quiet append_review_evidence_section_adds_heading_only_for_non_empty_lines; cargo test --quiet access_plan_interactive_browser_action_details_include_shared_review_evidence; cargo test --quiet datasource_inspect_export_browser_items_include_review_evidence; cargo test --quiet snapshot_review_browser_items_reuse_datasource_review_evidence_without_secret_paths; cargo test --quiet review_contract; cargo test --quiet access_plan; cargo test --quiet datasource_inspect_export; cargo test --quiet snapshot_review; cargo fmt --check; cargo check --quiet --no-default-features --all-targets; git diff --check
- Impact: TUI browser details now share the same Review evidence heading and empty-section behavior across access plan, datasource local, and snapshot datasource rows. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The helper preserves existing rendered lines and centralizes repeated section-shaping logic covered by focused tests.
- Follow-up: Continue migrating compatible review surfaces onto shared review diff/detail helpers when they expose safe live/desired evidence.

## 2026-05-25 - Shared datasource TUI review projection
- Summary: Added a datasource-details review projection helper and routed datasource local interactive rows plus snapshot datasource review rows through it instead of constructing dummy browse items.
- Tests: cargo test --quiet review_lines_surface_plan_action_evidence_from_details; cargo test --quiet datasource_inspect_export_browser_items_include_review_evidence; cargo test --quiet snapshot_review_browser_items_reuse_datasource_review_evidence_without_secret_paths; cargo test --quiet datasource_inspect_export; cargo test --quiet snapshot_review; cargo test --quiet datasource_browse; cargo fmt --check; cargo check --quiet --no-default-features --all-targets; git diff --check
- Impact: Datasource local list and snapshot review interactive browser rows now share the datasource Review projection directly from details while live datasource browse behavior stays unchanged. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change removes duplicate adapter objects and preserves existing review line output through focused equivalence coverage.
- Follow-up: Continue migrating compatible review surfaces onto shared review diff/detail helpers when they expose safe live/desired evidence.

## 2026-05-25 - Shared TUI review changed-field safety
- Summary: Moved secret-like changed-field filtering into the shared review diff helper and routed sync diff models, datasource/snapshot review evidence, and access plan review detail filtering through the shared predicate.
- Tests: cargo fmt --check; cargo test --quiet shared_review_changed_field_filter_hides_secret_like_paths; cargo test --quiet review_operation_diff_model_hides_secret_like_changed_fields; cargo test --quiet review_operation_preview_hides_secret_like_changed_fields; cargo test --quiet datasource_inspect_export_browser_items_include_review_evidence; cargo test --quiet datasource_inspect_export; cargo test --quiet snapshot_review_document_preserves_datasource_review_evidence_for_browser_projection; cargo test --quiet snapshot_review_browser_items_reuse_datasource_review_evidence_without_secret_paths; cargo test --quiet access_plan_interactive_shared_diff_preview_hides_secret_like_fields; cargo test --quiet review_diff; cargo test --quiet cli_review_tui_rust_tests; cargo test --quiet access_plan; cargo test --quiet snapshot_review; cargo check --quiet --no-default-features --all-targets; git diff --check
- Impact: TUI review panes now share the same changed-field redaction policy before rendering field names or side-by-side values. Public CLI paths, help text, command contracts, generated docs, Python behavior, and package metadata are unchanged.
- Rollback/Risk: Low. The change narrows review display output for secret-like fields and keeps existing safe fields visible; rollback would restore duplicated predicates and narrower datasource filtering.
- Follow-up: Continue migrating compatible review surfaces onto shared review diff/detail helpers when they already expose safe live/desired evidence.

## 2026-05-25 - Access plan shared diff preview
- Summary: Made access plan action details build compact shared live/desired diff previews from bundle/live change rows while filtering secret-like change fields from both the generic review details and TUI preview output.
- Tests: cargo fmt --check; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo check --quiet --no-default-features --all-targets; cargo test --quiet access_plan; cargo test --quiet review_diff; cargo test --quiet access_plan_interactive_browser_items_follow_review_projection; cargo test --quiet access_plan_interactive_browser_action_details_include_shared_review_evidence; cargo test --quiet access_plan_interactive_shared_diff_preview_hides_secret_like_fields
- Impact: Access plan review rows now share the same ReviewDiffModel preview vocabulary as sync review and dashboard import review for compatible field-change evidence, reducing per-surface shaping while avoiding secret-like value leakage.
- Rollback/Risk: low. Rollback would remove the compact shared live/desired preview from access plan rows and restore the previous per-row Change lines only.
- Follow-up: Continue migrating compatible review surfaces that have safe live/desired evidence onto shared ReviewDiffModel preview or side-by-side rendering.

## 2026-05-25 - Dashboard import shared diff preview
- Summary: Made dashboard import interactive reviews build a shared ReviewDiffModel for changed live-vs-local title, folder UID, tag, and panel evidence, then render a compact shared live/desired diff preview in the review pane.
- Tests: cargo fmt --check; cargo check --quiet --no-default-features --all-targets; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo test --quiet import_interactive; cargo test --quiet dashboard_import; cargo test --quiet review_diff; cargo test --quiet import_review_panel_projects_shared_diff_preview_when_available; cargo test --quiet interactive_import_review_diff_model_uses_shared_changed_field_projection; cargo test --quiet shared_review_diff_view_helpers_cover_titles_scroll_and_text_windows
- Impact: Dashboard import review now consumes the same shared diff model path as sync review for compatible changed-field evidence, reducing per-surface review shaping while preserving existing summary/structural/raw diff lines.
- Rollback/Risk: low. Rollback would remove the compact shared live/desired preview from the import review pane while keeping existing import summary, structural, and raw diff lines.
- Follow-up: Continue migrating other compatible review surfaces that already expose live/desired/changed-field evidence onto the shared ReviewDiffModel projection.

## 2026-05-25 - No-default TUI helper warning cleanup
- Summary: Moved remaining TUI-only or TUI-test-only helper aliases and re-exports out of no-default production/test targets so no-default all-target builds can run with warnings denied.
- Tests: cargo fmt --check; RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo check --quiet --no-default-features --all-targets; cargo test --quiet --no-default-features tui_not_built_returns_shared_tui_feature_error; cargo test --quiet --no-default-features access_plan_interactive_browser; cargo test --quiet --no-default-features review_mutation_action_detail_lines_project_generic_review_evidence; cargo test --quiet --no-default-features workspace_roots_are_treated_as_local_browse_sources; cargo test --quiet access_plan_interactive_browser; cargo test --quiet resolve_report_column_ids; cargo test --quiet dispatch_query_analysis_matches_shared_analyzer_fixture_cases; cargo test --quiet build_topology_tui_groups_summarize_node_kinds; cargo test --quiet filter_topology_tui_items_limits_items_to_selected_group; cargo test --quiet snapshot_review_parses_all_supported_output_modes; cargo test --quiet root_command_entrypoints_use_grouped_help_for_bare_and_help_forms
- Impact: The remaining no-default TUI helper/alias warning surface is cleared while default-feature access plan, dashboard browse/report/topology, snapshot parser, and grouped-help tests remain behavior-compatible.
- Rollback/Risk: low. Rollback would restore no-default warning noise without changing normal default-feature interactive behavior.
- Follow-up: Continue higher-level TUI design work by migrating compatible review surfaces onto shared diff/detail visualization helpers instead of chasing no-default warning hygiene.

## 2026-05-25 - Dashboard no-default TUI cfg cleanup
- Summary: Gated topology/impact test interactive browser branches and dashboard import/inspect test-only re-exports on the tui feature so no-default all-target builds no longer compile those TUI-only paths or emit their unreachable/unused warnings.
- Tests: cargo fmt --check; cargo check --quiet --no-default-features --all-targets; cargo test --quiet --no-default-features tui_not_built_returns_shared_tui_feature_error; cargo test --quiet dashboard_topology; cargo test --quiet topology_impact; cargo test --quiet routed_import_scope_identity_matches_table_json_and_progress_surfaces; cargo test --quiet export_focus_report_query
- Impact: The no-default warning surface is narrower and now leaves only smaller helper/alias dead-code follow-ups; default-feature dashboard topology, routed import, and query report tests remain behavior-compatible.
- Rollback/Risk: low. Rollback would restore the no-default unreachable/unused warning noise without changing normal default-feature interactive behavior.
- Follow-up: Continue trimming the remaining no-default helper/alias warnings around access plan aliases, dashboard browse source helpers, review-contract detail projection, snapshot root command support, report column helpers, and dashboard test fixtures.

## 2026-05-25 - No-default TUI warning cleanup
- Summary: reduced `--no-default-features` TUI warning noise by removing unused fallback imports and allowing dead code only on TUI-heavy helper modules that are intentionally compiled for tests or feature-enabled interactive paths.
- Tests: `cargo fmt --check`; `cargo test --quiet --no-default-features tui_not_built_returns_shared_tui_feature_error`; `cargo test --quiet access_plan`; `cargo test --quiet snapshot_review`; `cargo test --quiet datasource_inspect_export_browser_items`; `cargo test --quiet build_governance_gate_tui_groups_summarizes_findings`.
- Impact: `rust/src/commands/access/access_browse.rs`, `rust/src/commands/access/access_plan_tui.rs`, `rust/src/common/browser/session.rs`, `rust/src/commands/review_diff.rs`, `rust/src/commands/sync/review_tui.rs`, `rust/src/commands/snapshot/review/browser.rs`, `rust/src/commands/dashboard/governance_gate.rs`, `rust/src/commands/dashboard/governance_gate/items.rs`, `rust/src/commands/datasource/browse/support.rs`, and maintainer trace docs. Public CLI paths, help output, command contracts, generated docs, Python behavior, and package metadata are intentionally unchanged.
- Rollback/Risk: low. Rollback would restore broader no-default warning output without changing normal default-feature interactive behavior.

## 2026-05-25 - Shared review diff visualization helpers
- Summary: moved the sync review TUI's generic side-by-side diff view helpers into `review_diff.rs`, including pane focus/control state, scroll limits, pane titles, text wrapping/clipping, and TUI list-item/control rendering, while keeping sync review's operation collection and safe changed-field preview logic local.
- Tests: `cargo fmt --check`; `cargo test --quiet review_diff`; `cargo test --quiet review_operation`; `cargo test --quiet review_diff_scroll_max_uses_longer_side`; `cargo test --quiet wrap_text_chunks_splits_long_diff_lines_for_pane_width`; `cargo test --quiet clip_text_window_slices_nowrap_diff_content`; `cargo test --quiet --no-default-features tui_not_built_returns_shared_tui_feature_error`; escalated `cargo test --quiet sync` because the sandbox denied local TCP mock-server binding.
- Impact: `rust/src/commands/review_diff.rs`, `rust/src/commands/review_diff_rust_tests.rs`, `rust/src/commands/sync/review_tui_helpers.rs`, and maintainer trace docs. Public CLI paths, help output, command contracts, generated docs, Python behavior, and package metadata are intentionally unchanged.
- Rollback/Risk: low. Rollback would move the generic helpers back into sync review, reducing reuse for other compatible TUI review surfaces without changing sync review behavior.
