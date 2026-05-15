# ai-status-archive-2026-05-14

## 2026-05-02 - Extend mutation action adapters
- State: Done
- Scope: Rust internal review projections/envelopes for access import dry-run, datasource import dry-run, datasource live mutation preview, and alert plan rows; focused domain tests; full Rust validation; TODO trace. Public JSON, CLI behavior, `ReviewRisk`, `ReviewRequest`, legacy dashboard import dry-run, generated docs, and Python implementation are out of scope.
- Baseline: Dashboard plan, datasource plan, access plan, and workspace preview already projected into `ReviewMutationAction`, but selected dry-run/import rows still only had domain-local review evidence.
- Current Update: Added internal-only adapters that normalize proven action/status/blocked-reason fields into `ReviewMutationAction` while preserving original domain rows as `raw`.
- Result: Focused access/datasource/alert tests and full Rust validation pass.
