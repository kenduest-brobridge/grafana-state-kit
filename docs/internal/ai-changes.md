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

## 2026-05-25 - Browse search and review pane consistency
- Summary: aligned `access user browse`, `access team browse`, `dashboard browse`, `datasource browse`, shared read-only browser, status overview, and dashboard inspect workbench repeat-search behavior/copy with the other Rust TUI browsers so repeated `n` searches skip the current row and wrap to the first or last matching row at result-set boundaries; promoted datasource secret/provider/read-only evidence into a dedicated Review pane, extended that pane to recognize compatible plan/diff/import review fields already present in datasource details, reused those safe Review lines in local `datasource list --interactive` item details and snapshot review datasource rows, aligned dashboard inspect workbench plus access/datasource/dashboard dialog/search copy with compact `Esc/q`-style controls, and clamped status TUI detail scrolling to the selected content.
- Tests: focused Rust state tests cover forward wrap, backward wrap, no previous search, empty query, and no-match cases for user/team access browse, dashboard/datasource forward/backward boundary wrap cases, shared read-only browser forward/backward repeat-search wrap within the active filter, dashboard inspect workbench backward boundary wrap, status TUI detail-scroll clamping, full-detail status copy, compact full-detail close controls, compact access/datasource/dashboard confirmation dialog copy, compact search-prompt hints, datasource Review focus cycling, separated Facts/Review evidence, render output that hides resolved credential values, provider references without raw tokens, read-only blocker evidence, local action/status/target evidence, diff-style status rows, secret-like changed-field path redaction, local datasource list browser detail reuse, snapshot review datasource evidence preservation/projection, and dashboard inspect workbench footer exit copy.
- Impact: `rust/src/common/browser/session.rs`, `rust/src/commands/access/user_browse_dialog.rs`, `rust/src/commands/access/user_browse_state.rs`, `rust/src/commands/access/team_browse_dialog.rs`, `rust/src/commands/access/team_browse_state.rs`, `rust/src/commands/dashboard/browse/render.rs`, `rust/src/commands/dashboard/browse/state.rs`, `rust/src/commands/dashboard/inspect_workbench/modal_state.rs`, `rust/src/commands/dashboard/inspect_workbench/render.rs`, `rust/src/commands/dashboard/inspect_workbench/render_modal_sections.rs`, `rust/src/commands/dashboard/inspect_workbench/state.rs`, `rust/src/commands/datasource/browse/state.rs`, `rust/src/commands/datasource/browse/support.rs`, `rust/src/commands/datasource/browse/render.rs`, `rust/src/commands/datasource/inspect/export.rs`, `rust/src/commands/datasource/mod.rs`, `rust/src/commands/snapshot/review/browser.rs`, `rust/src/commands/snapshot/review/document.rs`, `rust/src/commands/status/overview/tui_render.rs`, `rust/src/commands/status/tui/mod.rs`, datasource/snapshot tests, and maintainer trace docs. Public CLI paths, help text, command contracts, generated man/html docs, Python behavior, and package metadata are intentionally unchanged.
- Rollback/Risk: low to moderate. Rollback would restore older boundary search behavior and move datasource review evidence back into general fact rows, while leaving non-interactive datasource outputs unchanged.

## 2026-05-16 - TUI search and shell consistency pass
- Summary: advanced the Rust TUI consistency work after the Phase 0 roadmap by adding shared read-only browser search, status overview current-view search, access browse shared-shell header/footer language, datasource browse selection/search context, consistent TUI feature-required errors, and unified `Esc/q` exit labels across remaining Rust TUI surfaces.
- Tests: focused Rust tests covered shared browser search matching/repeat behavior, status overview search and render copy, access browse shell summaries, datasource browse context summaries, feature-gate fallback messages, and the final combined exit label regression. GitHub Actions for the latest pushed `dev` state completed successfully.
- Impact: `rust/src/common/browser/session.rs`, `rust/src/common/error.rs`, access/datasource/status/sync/dashboard TUI render/state modules, and this maintainer trace. Public CLI paths, help output, command contracts, generated man/html docs, Python package behavior, and release metadata are intentionally unchanged.
- Rollback/Risk: low to moderate. The changes are isolated to interactive Rust TUI state/render behavior and copy; rollback would restore the older per-surface search/control behavior without affecting non-interactive outputs. The remaining follow-up at the time was repeat-search wrap consistency for `access user browse` and `access team browse`.

## 2026-05-14 - Phase 0 TUI inventory roadmap
- Summary: added `docs/internal/tui-architecture-roadmap.md` to inventory current TUI/interactive surfaces, maturity tiers, architecture debt, and the next approved implementation phases; added a read-only inventory report script; improved access browse filtering/selection summaries; and aligned datasource browse exit control copy.
- Tests: targeted Rust tests cover the new access browse state behavior and datasource browse control-label rendering. Script smoke checks and AI workflow validation cover the maintainer-only inventory artifacts.
- Impact: `docs/internal/tui-architecture-roadmap.md`, `scripts/tui_inventory_report.py`, `rust/src/commands/access/access_browse.rs`, `rust/src/commands/datasource/browse/render.rs`, and AI trace docs. Dashboard browse, shared TUI production code, public CLI behavior, generated docs, and Python package behavior are intentionally unchanged.
- Rollback/Risk: low to moderate. Access browse filtering is additive and local to the TUI loop; datasource browse copy is render-only; the roadmap and manual report script can be removed without changing runtime behavior.

## 2026-05-02 - Split datasource import dry-run helpers
- Summary: split datasource import dry-run responsibilities so runtime collection remains in `dry_run.rs`, output rendering lives in `dry_run_output.rs`, review projection/tests live in `dry_run_review.rs`, and secret visibility lives in `dry_run_secret_visibility.rs`.
- Tests: main is running focused Rust validation for the refactor; this update is maintainer trace only.
- Impact: `rust/src/commands/datasource/import/dry_run.rs`, `rust/src/commands/datasource/import/dry_run_output.rs`, `rust/src/commands/datasource/import/dry_run_review.rs`, `rust/src/commands/datasource/import/dry_run_secret_visibility.rs`, `docs/internal/rust-architecture-guardrails.md`, and AI trace docs. Public CLI/JSON behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low behavior-preserving module split. Rollback would move the extracted helper code back into `dry_run.rs` without changing external output.

## 2026-05-02 - Split alert runtime support helpers
- Summary: split alert runtime support responsibilities so plan/delete/import/diff document construction lives in `runtime_plan_document.rs` and alert plan review projections live in `runtime_review.rs`, leaving `runtime_support.rs` as the narrower runtime assembly surface.
- Tests: main is running focused Rust validation for the refactor; this update is maintainer trace only.
- Impact: `rust/src/commands/alert/runtime_support.rs`, `rust/src/commands/alert/runtime_plan_document.rs`, `rust/src/commands/alert/runtime_review.rs`, and AI trace docs. Public CLI/JSON behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low behavior-preserving module split. Rollback would move the extracted helper code back into `runtime_support.rs` without changing external output.

## 2026-05-02 - Consume mutation review adapters
- Summary: added `build_review_mutation_summary_rows(&ReviewMutationEnvelope)` as a shared internal consumer for access import dry-run, datasource import dry-run, datasource live mutation, and alert plan review adapters.
- Tests: worker tests prove those adapters feed the shared summary consumer without public JSON drift.
- Impact: `rust/src/commands/review_contract.rs`, focused Rust tests, `todo.md`, and AI trace docs. Public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low internal-consumer change. Rollback would remove the shared summary projection while leaving public domain review and dry-run outputs unchanged.

## 2026-05-02 - Cleanup TODO trace after mutation adapter pass
- Summary: refreshed the active AI trace entry after the mutation adapter pass and kept this as docs/TODO cleanup only.
- Tests: no Rust tests were needed for this documentation-only update. Validation should be `make quality-ai-workflow` and `git diff --check`.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, and the current AI trace only. Rust behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only change. Rollback would remove the refreshed TODO trace note and restore the previous active trace state.

## 2026-05-02 - Close remaining P3 TODO guardrail
- Summary: closed the remaining P3 TODO guardrail in the active AI trace and added a concrete backlog item to consume the review-adapter output on the next pass.
- Tests: no Rust tests were needed because this is docs/TODO cleanup only. Validation should be `make quality-ai-workflow` and `git diff --check`.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, and the current AI trace only. Rust behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only change. Rollback would remove the guardrail closure note and the new review-adapter consumption backlog entry.

## 2026-05-02 - Extend mutation action adapters
- Summary: added internal `ReviewMutationAction` projections/envelopes for access import dry-run, datasource import dry-run, datasource live mutation preview, and alert plan rows without changing public JSON.
- Tests: followed TDD in worker slices for access, datasource import, datasource live mutation, and alert plan adapters; then ran focused adapter tests, broader access/datasource/alert filters, and full Rust validation.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet access_import_dry_run_review_envelope_projects_ready_and_blocked_rows`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_import_dry_run_review_projection_and_envelope_preserve_row_evidence`; `cargo test --manifest-path rust/Cargo.toml --quiet live_mutation_preview_review_projection_and_envelope_preserve_row_evidence`; `cargo test --manifest-path rust/Cargo.toml --quiet alert_rust_tests_runtime_plan::alert_plan_review_projection_maps_local_actions_without_changing_raw_rows`; `cargo test --manifest-path rust/Cargo.toml --quiet access`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource`; `cargo test --manifest-path rust/Cargo.toml --quiet alert`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/access/import_dry_run.rs`, `rust/src/commands/datasource/import/dry_run.rs`, `rust/src/commands/datasource/mutation/render.rs`, `rust/src/commands/alert/runtime_support.rs`, `rust/src/commands/alert/tests/runtime_plan.rs`, `todo.md`, and AI trace docs. Public JSON, CLI behavior, `ReviewRisk`, `ReviewRequest`, legacy dashboard import dry-run, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low internal-adapter change. Rollback would remove normalized internal review views while leaving existing public dry-run/plan outputs unchanged; focused tests cover raw row preservation and action/status/blocked-reason mappings.
- Follow-up: keep `ReviewRisk` and `ReviewRequest` blocked until their cross-domain evidence improves; handle legacy dashboard import dry-run separately because its skip/fail actions need careful mapping.

## 2026-05-02 - Re-audit mutation review envelope evidence
- Summary: refreshed the mutation review envelope inventory with worker-backed evidence across dashboard/workspace, access/datasource, and alert/sync review surfaces.
- Tests: no Rust tests were needed because this is maintainer-only evidence and TODO routing. Validation covered AI workflow rules and whitespace checks.
- Test Run: `make quality-ai-workflow`; `git diff --check`.
- Impact: `docs/internal/mutation-review-envelope-inventory.md`, `todo.md`, and AI trace docs. Runtime behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only checkpoint. Rollback would restore the previous ambiguous TODO state where blocked `ReviewRisk` / `ReviewRequest` work looked equally ready as mutation action adapter coverage.
- Follow-up: implement the next adapter pass for access import dry-run, datasource import dry-run, datasource live mutation preview, and alert plan rows only where the mapping is lossless and internal-only.
