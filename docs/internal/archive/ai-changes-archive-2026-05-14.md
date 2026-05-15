# ai-changes-archive-2026-05-14

## 2026-04-26 - Prove provisioning remains derived dashboard projection
- Summary: added regression coverage that provisioning dashboard files normalize to the same classic dashboard compare payload as raw export wrappers, and that sync bundle rejects explicit dual raw/provisioning dashboard inputs instead of treating provisioning as an alternate source of truth.
- Tests: covered raw wrapper vs provisioning compare normalization, provisioning classic payload shape, direct provisioning source loading, import dry-run provisioning roots, and sync bundle dual dashboard source rejection.
- Test Run: `cargo test --manifest-path rust/Cargo.toml --quiet compare_local_document_`; `cargo test --manifest-path rust/Cargo.toml --quiet source_loader_contract_resolves_direct_provisioning_root`; `cargo test --manifest-path rust/Cargo.toml --quiet collect_import_dry_run_report_accepts_provisioning_root_variant_metadata`; `cargo test --manifest-path rust/Cargo.toml --quiet run_sync_cli_bundle_rejects_dual_dashboard_sources`; `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
- Impact: `rust/src/commands/dashboard/import/compare.rs`, `rust/src/commands/sync/bundle_exec_sources_rust_tests.rs`, `todo.md`, and AI trace docs. Public JSON, generated docs, Python implementation, and dashboard v2 support are intentionally unchanged.
- Rollback/Risk: low test-only boundary hardening. Rollback would remove the regression coverage while leaving existing provisioning behavior unchanged.
