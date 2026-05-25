# ai-changes-archive-2026-05-25

## 2026-05-02 - Reduce proven JSON clone hot spots
- Summary: removed avoidable `serde_json::Value`/object clones in owned Rust read and aggregation paths without changing public JSON or transport behavior.
- Tests: covered dashboard API response normalization, dashboard version timestamp lookup, sync live availability merge, request-backed contact-point availability extraction, live multi-org status aggregation, broader dashboard/sync_live/status filters, and full Rust validation.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard_resource_client`; `cargo test --manifest-path rust/Cargo.toml --quiet latest_dashboard_version_timestamp`; `cargo test --manifest-path rust/Cargo.toml --quiet fetch_live_availability_with_request_collects_contact_points_and_plugins`; `cargo test --manifest-path rust/Cargo.toml --quiet merge_availability_deduplicates_arrays_and_overwrites_scalar_fields`; `cargo test --manifest-path rust/Cargo.toml --quiet build_live_multi_org_domain_status`; `cargo test --manifest-path rust/Cargo.toml --quiet dashboard`; `cargo test --manifest-path rust/Cargo.toml --quiet sync_live`; `cargo test --manifest-path rust/Cargo.toml --quiet status`; `cargo test --manifest-path rust/Cargo.toml --quiet`; `make quality-ai-workflow`; `git diff --check`.
- Impact: `rust/src/grafana/api/dashboard.rs`, `rust/src/grafana/api/sync_live.rs`, `rust/src/grafana/api/sync_live_read.rs`, `rust/src/grafana/api/sync_live_read/availability.rs`, `rust/src/commands/status/live_multi_org.rs`, `rust/src/commands/sync/live_rust_tests.rs`, `todo.md`, and AI trace docs. Public JSON, CLI behavior, live transport semantics, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low behavior-preserving ownership cleanup. Rollback would restore cloning during response normalization and status aggregation; focused and broader Rust tests cover the unchanged output behavior.
- Follow-up: keep future `Value` clone cleanup evidence-led and avoid changing flexible JSON handling just to make data structures more static.

## 2026-05-02 - Re-audit mutation review envelope evidence
- Summary: refreshed the mutation review envelope inventory with worker-backed evidence across dashboard/workspace, access/datasource, and alert/sync review surfaces.
- Tests: no Rust tests were needed because this is maintainer-only evidence and TODO routing. Validation covered AI workflow rules and whitespace checks.
- Test Run: `make quality-ai-workflow`; `git diff --check`.
- Impact: `docs/internal/mutation-review-envelope-inventory.md`, `todo.md`, and AI trace docs. Runtime behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only checkpoint. Rollback would restore the previous ambiguous TODO state where blocked `ReviewRisk` / `ReviewRequest` work looked equally ready as mutation action adapter coverage.
- Follow-up: implement the next adapter pass for access import dry-run, datasource import dry-run, datasource live mutation preview, and alert plan rows only where the mapping is lossless and internal-only.

## 2026-05-02 - Extend mutation action adapters
- Summary: added internal `ReviewMutationAction` projections/envelopes for access import dry-run, datasource import dry-run, datasource live mutation preview, and alert plan rows without changing public JSON.
- Tests: followed TDD in worker slices for access, datasource import, datasource live mutation, and alert plan adapters; then ran focused adapter tests, broader access/datasource/alert filters, and full Rust validation.
- Test Run: `cargo fmt --manifest-path rust/Cargo.toml --all`; `cargo test --manifest-path rust/Cargo.toml --quiet access_import_dry_run_review_envelope_projects_ready_and_blocked_rows`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource_import_dry_run_review_projection_and_envelope_preserve_row_evidence`; `cargo test --manifest-path rust/Cargo.toml --quiet live_mutation_preview_review_projection_and_envelope_preserve_row_evidence`; `cargo test --manifest-path rust/Cargo.toml --quiet alert_rust_tests_runtime_plan::alert_plan_review_projection_maps_local_actions_without_changing_raw_rows`; `cargo test --manifest-path rust/Cargo.toml --quiet access`; `cargo test --manifest-path rust/Cargo.toml --quiet datasource`; `cargo test --manifest-path rust/Cargo.toml --quiet alert`; `cargo test --manifest-path rust/Cargo.toml --quiet`.
- Impact: `rust/src/commands/access/import_dry_run.rs`, `rust/src/commands/datasource/import/dry_run.rs`, `rust/src/commands/datasource/mutation/render.rs`, `rust/src/commands/alert/runtime_support.rs`, `rust/src/commands/alert/tests/runtime_plan.rs`, `todo.md`, and AI trace docs. Public JSON, CLI behavior, `ReviewRisk`, `ReviewRequest`, legacy dashboard import dry-run, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low internal-adapter change. Rollback would remove normalized internal review views while leaving existing public dry-run/plan outputs unchanged; focused tests cover raw row preservation and action/status/blocked-reason mappings.
- Follow-up: keep `ReviewRisk` and `ReviewRequest` blocked until their cross-domain evidence improves; handle legacy dashboard import dry-run separately because its skip/fail actions need careful mapping.

## 2026-05-02 - Close remaining P3 TODO guardrail
- Summary: closed the remaining P3 TODO guardrail in the active AI trace and added a concrete backlog item to consume the review-adapter output on the next pass.
- Tests: no Rust tests were needed because this is docs/TODO cleanup only. Validation should be `make quality-ai-workflow` and `git diff --check`.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, and the current AI trace only. Rust behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only change. Rollback would remove the guardrail closure note and the new review-adapter consumption backlog entry.

## 2026-05-02 - Cleanup TODO trace after mutation adapter pass
- Summary: refreshed the active AI trace entry after the mutation adapter pass and kept this as docs/TODO cleanup only.
- Tests: no Rust tests were needed for this documentation-only update. Validation should be `make quality-ai-workflow` and `git diff --check`.
- Impact: `docs/internal/ai-status.md`, `docs/internal/ai-changes.md`, and the current AI trace only. Rust behavior, public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low documentation-only change. Rollback would remove the refreshed TODO trace note and restore the previous active trace state.

## 2026-05-02 - Consume mutation review adapters
- Summary: added `build_review_mutation_summary_rows(&ReviewMutationEnvelope)` as a shared internal consumer for access import dry-run, datasource import dry-run, datasource live mutation, and alert plan review adapters.
- Tests: worker tests prove those adapters feed the shared summary consumer without public JSON drift.
- Impact: `rust/src/commands/review_contract.rs`, focused Rust tests, `todo.md`, and AI trace docs. Public JSON, CLI behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low internal-consumer change. Rollback would remove the shared summary projection while leaving public domain review and dry-run outputs unchanged.

## 2026-05-02 - Split alert runtime support helpers
- Summary: split alert runtime support responsibilities so plan/delete/import/diff document construction lives in `runtime_plan_document.rs` and alert plan review projections live in `runtime_review.rs`, leaving `runtime_support.rs` as the narrower runtime assembly surface.
- Tests: main is running focused Rust validation for the refactor; this update is maintainer trace only.
- Impact: `rust/src/commands/alert/runtime_support.rs`, `rust/src/commands/alert/runtime_plan_document.rs`, `rust/src/commands/alert/runtime_review.rs`, and AI trace docs. Public CLI/JSON behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low behavior-preserving module split. Rollback would move the extracted helper code back into `runtime_support.rs` without changing external output.

## 2026-05-02 - Split datasource import dry-run helpers
- Summary: split datasource import dry-run responsibilities so runtime collection remains in `dry_run.rs`, output rendering lives in `dry_run_output.rs`, review projection/tests live in `dry_run_review.rs`, and secret visibility lives in `dry_run_secret_visibility.rs`.
- Tests: main is running focused Rust validation for the refactor; this update is maintainer trace only.
- Impact: `rust/src/commands/datasource/import/dry_run.rs`, `rust/src/commands/datasource/import/dry_run_output.rs`, `rust/src/commands/datasource/import/dry_run_review.rs`, `rust/src/commands/datasource/import/dry_run_secret_visibility.rs`, `docs/internal/rust-architecture-guardrails.md`, and AI trace docs. Public CLI/JSON behavior, generated docs, and Python implementation are intentionally unchanged.
- Rollback/Risk: low behavior-preserving module split. Rollback would move the extracted helper code back into `dry_run.rs` without changing external output.
