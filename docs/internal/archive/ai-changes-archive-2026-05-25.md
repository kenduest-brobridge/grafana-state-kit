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
