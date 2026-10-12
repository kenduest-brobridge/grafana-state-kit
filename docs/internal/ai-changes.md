# ai-changes.md

Current AI change log only.
- Older entry moved to [ai-changes-archive-2026-10-12.md](archive/ai-changes-archive-2026-10-12.md).
- Older entries moved to [ai-changes-archive-2026-10-11.md](archive/ai-changes-archive-2026-10-11.md).

## 2026-10-12 - Development follow-through

- Summary: Restore all 51 useful dormant Dashboard cases under current domain
  owners; remove their obsolete router and adapt/strengthen fixtures and assertions.
- Runtime: Add fresh baseline reads before any controlled Alert write, truthful
  completed/failed/not-attempted results with failure exit status, three bounded
  dashboard API read probes, opt-in native-v2 metadata portability and Playlist
  v1 list/get/describe through existing read-only resource primitives.
- Contracts: Preserve default archival export, opaque `spec`, ownership guards,
  explicit namespace and destructive approval. Keep unsupported/unauthorized,
  malformed-response and transport/server failures distinct.
- Verification: `todo/2026-10-12-development-follow-through.md` records the final
  combined checks and live evidence. Physical TUI and OS keyring sessions are
  separate acceptance boundaries.
- Risk: Alert drift checks reduce stale-review writes but cannot supply atomic
  server transactions. Portable export does not remap datasource/folder identity
  or bypass managed-resource protections. No push or deployment.

## 2026-10-11 - Deep project review and maintainability convergence

- Summary: Restore fail-closed dashboard folder/ownership guards, explicit Basic
  auth precedence, strict HTTP success and JSON overwrite checks; encode dynamic
  Grafana routes and preserve team-admin roles, new-org dry-runs, prune intents,
  explicit package inputs, and dashboard folder/ownership evidence through sync.
- Workflow: Preflight all effective controlled alert mutations; honor routed
  datasource dry-runs and workspace review/output/live-availability options,
  and preserve significant whitespace in resolved datasource credentials.
- Convergence: Replace unused import lookup traits/adapters with existing calls,
  reuse domain connection builders, retire the unwired test-only root preflight,
  and remove 16 exact duplicate tests while retaining contract coverage.
  Restore three unique dormant Classic interactive review tests to active
  execution; record the remaining 51 dormant cases rather than deleting them.
- Secrets: Authenticate existing encrypted entries before writes; reject missing
  or wrong keys and malformed versions/nonces/salts; enable Linux Secret Service
  instead of the default mock backend. No encrypted format migration.
- Tests: Regression and integration evidence is tracked in
  `todo/2026-10-11-deep-project-review.md`.
- Impact/Risk: Tightens existing safety contracts without new CLI paths; Linux
  OS storage requires a real Secret Service session, and corrupted encrypted
  stores must be restored rather than overwritten with another key.

- Older entries moved to [`ai-changes-archive-2026-09-05.md`](archive/ai-changes-archive-2026-09-05.md).
- Older entries moved to [`ai-changes-archive-2026-09-10.md`](archive/ai-changes-archive-2026-09-10.md).

## 2026-09-10 - Grafana 13.2.1 compatibility verification
- Summary: Pin maintained 13.x Make/CI/smoke lanes to 13.2.1 and allow the independently reproduced org-delete fixture exception for that exact patch.
- Tests: Add strict version/status/JSON exception cases, query/adhoc/groupby export/import payload coverage, nested native-v2 spec preservation, and 403 propagation without retry or provenance bypass.
- Impact: Grafana 12 compatibility and production API routing remain unchanged; no new beta or prompt-import capability is introduced.
- Rollback/Risk: Low; test scripts, regression coverage and documentation only. Unknown versions and nonmatching failures remain fatal. See `todo/2026-09-10-upstream-stable-review.md` for current verification evidence.

## 2026-09-05 - Grafana 12 compatibility and screenshot retirement
- Summary: Removed the dashboard screenshot command and browser/Pillow release dependencies from Rust and Python, migrated maintained datasource writes to `/api/datasources/uid/{uid}`, and added pinned Grafana 12.0.10, 12.4.9, and 13.2.0 smoke lanes in Make and CI.
- Review fixes: Select the executable from Cargo build artifacts; clean temporary resources on exit using Docker's recorded container ID even after startup failure; require the recorded 13.2.0 HTTP 500 JSON error before skipping only deletion-dependent org steps; fix Python quality formatting and document the verification boundary.
- Tests: targeted and full Rust/Python suites; generated man/HTML and docs-surface checks; live Rust 12.0.10/12.4.9/13.2.0 and Python datasource 12.0.10/13.2.0 smoke lanes.
- Impact: The ordinary CLI contract is Grafana 12.0+; optional capabilities remain 404-only discovery. Screenshot automation is no longer a supported public surface, and datasource update/delete/import/sync/browse writes have no numeric-ID fallback.
- Rollback/Risk: Medium; screenshot command removal is intentionally breaking, while UID routing targets Grafana's stable datasource API. Grafana 13.2.0 org deletion is skipped only for affected test fixtures because the upstream server returns 500.
- Follow-up: Re-enable the Grafana 13 missing-org and access-org-replay smoke cases when the upstream organization deletion regression is fixed.

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
- Older entries moved to [`ai-changes-archive-2026-05-28.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-28.md).
- Older entries moved to [`ai-changes-archive-2026-06-11.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-06-11.md).
- Older entries moved to [`ai-changes-archive-2026-07-12.md`](/Users/ken/work/grafana-utils/.worktrees/grafana13-all-improvements/docs/internal/archive/ai-changes-archive-2026-07-12.md).
- Older entries moved to [`ai-changes-archive-2026-07-17.md`](archive/ai-changes-archive-2026-07-17.md).

## 2026-07-17 - Dashboard schema v2 resource import
- Summary: Added explicit `dashboard import --input-format resource-v2` support, then review-fixed production dispatch, response identity validation, output-column isolation, structured JSON summaries, path containment, interactive cancellation and redaction, unknown ownership protection, server-owned label stripping, dry-run output-selector enforcement, failure redaction, and Grafana-added schema-default comparison.
- Tests: focused resource-v2 import/API/interactive/render/apply suites; production-client HTTP regression; live Grafana 13.1 create/no-change/update/cleanup; full Rust TUI suite; docs/contracts/generated-output checks.
- Impact: Resource-v2 import uses only the namespaced `/apis` v2 endpoint, never converts or falls back to classic import, rejects malformed identities and escaped sources, supports review-first create/update reconciliation, and retains Git Sync/file-provisioning ownership protection.
- Rollback/Risk: Medium; the lane is opt-in and single-namespace, rejects incompatible classic org/folder/schema flags, and requires explicit update permission through `--replace-existing`.
- Follow-up: Add multi-namespace routing or automatic folder-resource creation only after live acceptance evidence establishes their contracts.

## 2026-06-11 - Shared browser detail and test split
- Summary: Moved shared browser detail projection helpers into common/browser/detail.rs and session behavior tests into common/browser/session_tests.rs while preserving the crate::interactive_browser helper surface.
- Tests: cargo test --quiet browser_ --features tui; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `common/browser/session.rs` is now below the architecture warning threshold and `make quality-architecture` reports no warnings. Public CLI paths, help text, generated docs, and shared browser behavior are unchanged.
- Rollback/Risk: Low; this is a structural move of existing helper/test code with re-exports preserving downstream call sites.
- Follow-up: Keep future shared browser work split between detail projection, tests, and runtime/session responsibilities.

## 2026-06-11 - Review contract test split
- Summary: Moved shared review contract inline tests into a file-backed test module while keeping production contract constants, action projection helpers, and envelope builders in review_contract.rs.
- Tests: cargo test --quiet review_contract; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `review_contract.rs` is now below the architecture warning threshold, reducing the warning list from two files to one without changing public CLI paths, help text, generated docs, or review projection behavior.
- Rollback/Risk: Low; this is a test-module move with the same private-helper coverage and no production behavior changes.
- Follow-up: Continue reducing the remaining shared warning-threshold file: shared browser session.

## 2026-07-12 - Grafana 13 App Platform integration
- Summary: Added the shared namespaced App Platform transport and integrated dashboard v2 export/history, generic resource inspection, public-preview notification/secret metadata, and Grafana 13.1 Git Sync root discovery.
- Tests: focused App Platform, dashboard, resource, parser, redaction, source-loader, and workspace-discovery suites; broad verification recorded in the owning todo.
- Impact: Operators can preserve native v1/v2 dashboard envelopes, inspect stable and preview resources with explicit maturity boundaries, and review current Git Sync repository layouts without bypassing source ownership.
- Rollback/Risk: Medium; new paths are opt-in or read-only, auto fallback is restricted to 404, and classic export/history remain the defaults.
- Follow-up: Keep beta resources explicit until Grafana promotes their API versions.

## 2026-06-11 - Datasource browse support review split
- Summary: Moved datasource browse review-evidence projection and safe changed-field filtering into a dedicated support review module while keeping support.rs focused on browse documents, item rows, and detail facts.
- Tests: cargo test --quiet datasource_browse --features tui; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `datasource/browse/support.rs` is now below the architecture warning threshold, reducing the warning list from three files to two without changing public CLI paths, help text, generated docs, or interactive behavior.
- Rollback/Risk: Low; this is a structural move of existing review projection code covered by focused datasource browse detail/review tests and downstream datasource inspect/snapshot re-export checks.
- Follow-up: Continue reducing the remaining shared warning-threshold files: review_contract and shared browser session.

## 2026-06-11 - Datasource browse TUI chrome split
- Summary: Moved datasource browse footer controls and search prompt rendering into a dedicated TUI chrome module while keeping the main renderer focused on frame, list, and detail layout.
- Tests: cargo test --quiet datasource_browse --features tui; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `datasource/browse/render.rs` is now below the architecture warning threshold, reducing the warning list from four files to three without changing public CLI paths, help text, generated docs, or interactive behavior.
- Rollback/Risk: Low; this is a structural move of existing footer/search rendering covered by focused datasource browse render tests.
- Follow-up: Continue reducing the remaining TUI/shared warning-threshold files: review_contract and shared browser session.
