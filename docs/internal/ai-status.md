# ai-status.md

Current AI-maintained status only.

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

## 2026-06-11 - Review contract test split
- State: Done
- Scope: rust/src/commands/review_contract.rs; rust/src/commands/review_contract_tests.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved shared review contract inline tests into a file-backed test module while preserving the same private-helper coverage.
- Result: `review_contract.rs` dropped below the architecture warning threshold; the only remaining architecture warning is shared browser session.

## 2026-06-11 - Datasource browse support review split
- State: Done
- Scope: rust/src/commands/datasource/browse/support.rs; rust/src/commands/datasource/browse/support_review.rs; rust/src/commands/datasource/mod.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved datasource browse review-evidence projection and safe changed-field filtering into a dedicated support review module while preserving existing detail/review tests.
- Result: `datasource/browse/support.rs` dropped below the architecture warning threshold; the remaining architecture warnings are review_contract and shared browser session.

## 2026-06-11 - Datasource browse TUI chrome split
- State: Done
- Scope: rust/src/commands/datasource/browse/render.rs; rust/src/commands/datasource/browse/render_chrome.rs; rust/src/commands/datasource/mod.rs; docs/internal/tui-architecture-roadmap.md
- Current Update: Moved datasource browse footer controls and search prompt rendering into a dedicated chrome module while preserving the existing renderer tests.
- Result: `datasource/browse/render.rs` dropped below the architecture warning threshold; datasource browse rendering remains behavior-compatible and the remaining architecture warnings are datasource browse support, review_contract, and shared browser session.
