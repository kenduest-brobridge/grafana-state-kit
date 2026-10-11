# ai-status.md

Current AI-maintained status only.
- Older entries moved to [ai-status-archive-2026-10-11.md](archive/ai-status-archive-2026-10-11.md).

## 2026-10-11 - Deep project review and maintainability convergence

- State: done; committed locally
- Scope: Rust shared auth/HTTP/files, dashboard folder and ownership guards,
  alert/datasource route encoding, access role/import behavior, sync artifacts,
  and profile secret integrity plus Linux backend selection.
- Convergence: Simplify import lookup adapters and duplicate auth resolution;
  remove 16 exact test copies and two obsolete prototype self-tests, preserving
  the real Alert assessment assertion; restore three unique dormant interactive
  cases to active coverage. Development priorities are recorded in
  `todo/2026-10-11-development-convergence.md`.
- Current Update: Reproduced guarded import mismatches, lost team-admin roles,
  new-org preview failure, prune deserialization failure, credential corruption
  and authentication precedence defects; regression repairs are integrated.
- Review Result: Full Rust tests (2,052 passed, one ignored), Python (1,213 tests,
  two skipped), fmt/Clippy, documentation contracts, and the five-lane Grafana
  matrix passed. Linux Secret Service cross-target compilation passed.
- Convergence Result: Full Rust gate after removals passed 2,034 tests (one
  ignored); three restored cases each passed exact execution and registration.
  Final fmt/Clippy, AI workflow and architecture checks passed.
  The remaining 51 dormant Dashboard cases need individual reconciliation.
- Boundary: Evidence and known Grafana 13 org-delete skips are recorded in
  `todo/2026-10-11-deep-project-review.md`; real OS-keyring sessions, interactive
  TUI review, and production acceptance remain outside this validation.

- Older entries moved to [`ai-status-archive-2026-09-05.md`](archive/ai-status-archive-2026-09-05.md).
- Older entries moved to [`ai-status-archive-2026-09-10.md`](archive/ai-status-archive-2026-09-10.md).

## 2026-09-10 - Grafana 13.2.1 compatibility verification
- State: done
- Scope: Refresh the maintained 13.x image to 13.2.1 while retaining 12.0.10 and 12.4.9 regression lanes.
- Current Update: Both live clients reproduced the org-delete HTTP 500; only 13.2.0 and 13.2.1 qualify for the exact-response fixture exception. Added dashboard JSON-preservation and permission-error regression coverage.
- Result: Full Rust/Python quality, generated-doc checks, the five-lane live matrix, and independent diff review passed.
- Boundary: Native-v2 preservation tests exercise client payloads, not server validation of unknown fields. Prompt exports still require Grafana UI datasource selection; beta and optional feature expansion remain out of scope.

## 2026-09-05 - Grafana 12 compatibility and screenshot retirement
- State: done
- Scope: retire dashboard screenshot/browser artifacts, use UID-only datasource writes, and pin released Grafana 12.0/12.x/13.x live lanes.
- Current Update: Screenshot commands and dependencies are removed from both maintained CLIs; datasource mutation flows use UID routes without numeric write fallback; the live matrix records health versions for 12.0.10, 12.4.9, and 13.2.0.
- Review Update: Smoke uses Cargo's reported executable, cleans owned containers by recorded ID even after startup failure, and validates HTTP status plus JSON before allowing the pinned 13.2.0 org-delete exception. Independent org export/diff/update coverage remains enabled.
- Boundary: Grafana 13.2.0 has an upstream organization-delete regression, so only the dependent missing-org/replay fixtures are skipped after the exact server response is observed; ordinary datasource and dashboard workflows remain covered.

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
- Older entries moved to [`ai-status-archive-2026-05-28.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-05-28.md).
- Older entries moved to [`ai-status-archive-2026-06-11.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-status-archive-2026-06-11.md).
- Older entries moved to [`ai-status-archive-2026-07-12.md`](/Users/ken/work/grafana-utils/.worktrees/grafana13-all-improvements/docs/internal/archive/ai-status-archive-2026-07-12.md).
- Older entries moved to [`ai-status-archive-2026-07-17.md`](archive/ai-status-archive-2026-07-17.md).

## 2026-07-17 - Dashboard schema v2 resource import
- State: Done
- Scope: explicit resource-v2 dashboard import lane, single-namespace `/apis` mutation contract, operator docs, and command-surface contract
- Current Update: Review/fix wired the production client entrypoint to the native lane, added strict collection and item/write response validation, lane-specific output columns, structured dry-run JSON, contained source discovery, terminal cancel semantics, secret-safe rendering and partial failures, guarded unknown ownership, stripped server-owned labels, and enforced dry-run-only output selectors.
- Result: The public CLI now reaches only the namespaced v2 API for resource input, rejects malformed or escaped inputs before mutation, and reports an unchanged live replay as `no_change` on Grafana 13.1.

## 2026-07-12 - Grafana 13 App Platform integration
- State: Done
- Scope: shared App Platform client; dashboard v2 export/history; status resource v1/v2/v1beta1; notification/secret safety; Grafana 13.1 Git Sync discovery
- Current Update: Added 404-only discovery and pagination, native v2 resource lanes, stable resource history, explicit preview inspection, secret redaction, and root-level Git Sync CRD discovery.
- Result: Classic behavior stays default while Grafana 13 resources are available through explicit, tested read-only/export lanes.

## 2026-06-11 - Shared browser detail and test split
- State: Done
- Scope: rust/src/common/browser/session.rs; rust/src/common/browser/detail.rs; rust/src/common/browser/session_tests.rs; rust/src/lib.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved shared browser detail projection helpers and session behavior tests out of session.rs while preserving the crate::interactive_browser public-in-crate helper surface.
- Result: `session.rs` dropped below the architecture warning threshold and `make quality-architecture` is clean.
