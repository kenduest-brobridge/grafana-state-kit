# Deep project review and fixes — 2026-10-11

## Scope and constraints

- Review Rust-first implementation: dashboard mutation/export, alert/access/
  datasource, sync/snapshot/status, shared transport/auth/files, CLI and delivery.
- Start from clean working tree at `eea06314`; preserve unrelated later changes.
- Fix confirmed defects with regression coverage; preserve public compatibility,
  secret handling, ownership guards and review/apply boundaries.
- Local commit authorized; no push or deployment. Luna owns Git commit work.

## Plan

- [x] Review independent domains and shared infrastructure; verify findings.
- [x] Repair confirmed defects and update affected contracts/documentation.
- [x] Run focused regression checks, then appropriate combined quality checks.
- [x] Reconcile results, remaining boundaries, and commit exact reviewed scope.

## Evidence and remaining work

- CodeGraph index available; current source remains authoritative.
- Initial Git status was clean. Combined verification passed; reviewed scope
  committed locally by Luna.

## Confirmed findings and repairs

| Priority | Failure and repair | Owning source |
| --- | --- | --- |
| P1 | Live flat-export folder guard differed from preview; load folder inventory for guarded imports and keep existing-target identity when its folder is unknown. | `dashboard/import/{apply_prepare,apply_live,dry_run}.rs` |
| P1 | History restore overwrote source-owned dashboards; apply ownership checks in preview and immediately before restore. | `dashboard/history_{restore,live}.rs` |
| P1 | Reserved UID/name delimiters changed HTTP paths; encode dynamic segments across domain API and request adapters. Reject URL dot segments before normalization. | `grafana/api/{dashboard,datasource,alerting,alert_live}.rs`, domain adapters, `grafana/http.rs` |
| P1 | Sync dropped live dashboard folder and source-management evidence; preserve metadata through normalization and planning, and honor every blocking ownership note. | `grafana/api/sync_live_read/dashboard.rs`, `sync/{bundle_inputs_dashboard,plan_builder}.rs`, `dashboard/import/target.rs` |
| P1 | Team member `permission: 4` was ignored, losing admins in exports/bulk updates; preserve Grafana permission semantics and legacy boolean fallback. | `access/team_runtime.rs` |
| P1 | Wrong passphrases/replaced keys could rewrite encrypted credentials; authenticate existing entries before changes. | `config/profile/secret_store.rs` |
| P1 | Linux OS storage silently used a nonpersistent mock; explicitly select Secret Service and compile its Linux backend guard. | `rust/Cargo.toml`, `config/profile/secret_store.rs` |
| P1 | Controlled alert apply trusted declared changed fields and validated rows after earlier writes; prebuild effective payloads, verify identities and actual changes, then execute. | `alert/runtime_support.rs`, `grafana/api/alert_live.rs` |
| P1 | Text-mode export-org datasource dry-run performed real writes; forward dry-run through scoped execution and preview missing organizations. | `datasource/import_export.rs` |
| P2 | UID-less update-only import created dashboards; skip missing targets consistently. | `dashboard/import/lookup_cache.rs` |
| P2 | New-org dry-run queried the source org ID or omitted user previews; simulate an empty target membership without remote reads. | `access/org_workflows_sync.rs` |
| P2 | Generated prune plans contain `desired: null`, which apply intent could not deserialize; normalize delete payloads at the adapter boundary. | `sync/apply_builder.rs` |
| P2 | Discovered inputs overrode explicit package/provisioning/preview datasource selections; honor explicit input precedence. | `sync/{cli,input_normalization}.rs` |
| P2 | Workspace review, output and live-availability flags were ignored; reuse review generation, persist scan/test output, and merge fetched availability. | `sync/task_first.rs`, status overview aggregation |
| P2 | Datasource secret placeholders stripped significant credential whitespace; preserve resolved values while rejecting whitespace-only inputs. | `datasource/secret/mod.rs` |
| P2 | Environment Token overrode explicit Basic credentials; keep environment auth as fallback. | `common/auth.rs` |
| P2 | Unresolved redirects/304 were treated as successful empty responses; accept only HTTP success statuses. | `grafana/http.rs` |
| P2 | No-overwrite JSON output followed dangling symlinks and had a creation race; use exclusive file creation. | `common/io.rs` |
| P2 | Existing encrypted-store reads recreated missing keys; malformed nonces panicked and versions/salts were unchecked. Require existing keys and validate format before use. | `config/profile/secret_store.rs` |

### Review coverage and decisions

- Dashboard: Classic/native resource lanes, mutation selectors, folder ordering,
  history, export readers, source containment and resource-version protection.
- Alert/access/datasource: request routing, replay/import/update boundaries,
  team roles, organization dry-runs, secret payload redaction, controlled apply.
- Workflow: package/preview/plan/apply, ownership, staged/live status, snapshot
  and resource discovery; confirmed ignored workspace flags are repaired.
- Shared infrastructure and delivery: auth/profile resolution, HTTP status/URL
  handling, overwrite behavior, CLI dispatch, feature/dependency configuration,
  CI/install/doc generation entrypoints. Python stays unchanged.
- Intentional behavior retained: native resource import has no Classic fallback;
  unknown ownership remains distinct from confirmed source-managed ownership;
  headless Linux requires an available Secret Service session.

### Verification results

- Original failures reproduced for folder guards, UID-less update-only,
  source-owned restore, reserved UID wire routing, team-admin preservation,
  new-org dry-run, prune intent, auth precedence, HTTP redirects, dangling
  output symlinks, URL dot segments, routed datasource dry-run writes,
  credential whitespace, and five encrypted-store regressions.
- Focused checks: common 22, dashboard 14, encrypted credentials 12, team 15,
  org import 8, controlled alert apply 12, resource paths 3, datasource secrets 7,
  and sync initial phase 226 passed. Wire fixtures ran with real loopback.
- Final `cargo test --locked --offline --quiet`: 2,010 library tests plus 42
  integration tests passed; one macOS Keychain smoke intentionally ignored.
  Two earlier full attempts stopped on a new workspace test's output-shape
  assertion; it now checks package preflight counts/inputs and passes.
- `cargo fmt --all --check` and `cargo clippy --locked --offline --all-targets
  -- -D warnings` passed. Clippy fixes were confined to new test helpers.
- Linux cross-target test compilation passed, including actual DBus/Secret
  Service dependencies. Real Linux session persistence was not exercised.
- Python: initial system interpreter failed because PyYAML was absent; existing
  Poetry environment passed 1,213 tests (2 skipped). No Python source fix needed.
- Five-lane `make test-grafana-compatibility-matrix` passed: Rust dashboard,
  alert, sync, datasource and access on 12.0.10/12.4.9/13.2.1, plus Python
  datasource on 12.0.10/13.2.1. Health versions matched. Grafana 13.2.1's known
  exact org-delete HTTP 500 exception skips dependent recreation/replay cases.
  Disposable containers were cleaned by their owned scripts.
- Documentation surface/output contracts and generated HTML/man checks passed;
  AI trace size repaired by archiving one existing entry from each trace file.
  Final handbook availability example explicitly supplies package and target
  inventory, matching the staged-only context requirement.
- Architecture check: no hard failures; five existing size warnings in
  untouched modules, without unrelated refactoring.

### Acceptance boundaries

- Loopback client tests and disposable Grafana smokes do not establish installed
  customer/production acceptance. No deployment or push is authorized.
- Real macOS Keychain smoke remains intentionally ignored; Linux Secret Service
  was compiled, not tested against a logged-in desktop session.
- Alert preflight protects artifact consistency; remote drift/CAS, multi-write
  rollback, process/disk interruption during profile persistence, and concurrent
  profile edits are outside these repairs.
- Interactive TUI review was wired to existing helpers but not manually driven.
