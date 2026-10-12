# Development follow-through — 2026-10-12

## Scope and constraints

- Start at `eb1e0c93` with a clean working tree. Rust-first; preserve unrelated
  work and compatibility. No push, deployment, or CodeGraph rebuild.
- Classify all 51 remaining dormant Dashboard tests individually. Restore
  meaningful cases in registered owners; delete only proven redundant or obsolete
  cases. Confirm registration and actual execution, including TUI feature gates.
- Implement the prior roadmap: Alert fresh-drift protection and partial execution
  results, bounded server capability diagnostics, opt-in portable native-v2 export,
  and Playlist read-only inventory. Reuse existing domain contracts and primitives.
- Keep archival export unchanged by default, preserve opaque native specs, keep
  destructive authorization/security boundaries, and avoid general frameworks.

## Plan

- [x] Classify and reconcile all 51 dormant tests with per-case evidence.
- [x] Add Alert pre-write baseline checks and truthful partial results.
- [x] Add bounded read-only capability diagnostics and operator documentation.
- [x] Add opt-in portable native-v2 export with a verified metadata policy.
- [x] Add Playlist list/get/describe using verified stable API contracts.
- [x] Verify combined changes and reconcile reports and generated documentation.
- [x] Have Luna commit exact verified scope and inspect final Git state.

## Verification

- Begin with affected test owners and test registration, then broaden for shared
  CLI/API/output changes. Run appropriate fmt, Clippy, Rust, docs/contract and
  architecture gates. Record actual commands and counts below.
- Prior runtime evidence is reusable only where unchanged. Real Grafana acceptance,
  physical interactive TUI and OS keyring evidence must be reported separately.

## Results and remaining work

- All 51 cases restore unique supported behavior; no case was deleted as stale
  or duplicated. Removed the obsolete browse router. The per-case owner and
  assertion map is in `2026-10-12-dashboard-test-reconciliation.md`.
- Focused restored cases: list 24, browse 27 passed. Without default features:
  list 24 and browse 5 passed; the other 22 are TUI-only. Final Cargo `--list`
  reconciliation confirms each of the 51 inventory names occurs exactly once.
- Alert: pre-fix reproducer wrongly applied two updates without checking drift;
  repair tests verify all-baseline reads before writes and zero writes on drift,
  missing/failed reads, exact partial partitions, schema/action/duplicate guards.
  Focused apply 20 and all Alert 94 passed. Real binary integration covers both
  first/second write failures, JSON stdout, exit 1 and raw-error suppression.
- Capabilities: three bounded GETs, explicit namespace, strict list/item identity,
  separate read availability/authentication/authorization/not-served/malformed/
  server/transport states. Preserve existing live/staged readiness output.
- Portable: opt-in `--portable-v2`, archival default and v1/classic unaffected,
  strict v2/namespace and served-lane checks, verified server-field allowlist,
  full opaque spec, unknown metadata and folder/ownership preserved. Destination
  import owns namespace/folder overrides and concurrency tokens. The older draft
  UI-folder-stripping requirement is superseded by this verified bounded policy.
- Playlist: stable `playlist.grafana.app/v1` list/get/describe under status resource;
  shared validation/pagination, encoded identity, empty-list and HTTP error tests.
  Only auto/v1 supported; no legacy fallback or mutation surface.
- Root integration also reproduced and repaired inherited `--color` loss in
  capability dispatch; kept an existing regression assertion. The first full
  suite identified missing Examples in the new leaf help; shared help text repairs
  this and the final full suite passes. Docs examples remain on their owning pages.

## Final checks and evidence

- `cargo test --locked --offline --quiet` with loopback access: 2,081 library
  plus 43 integration tests passed (2,124 total), one ignored macOS Keychain
  smoke. Log: `/tmp/codex/grafana-utils/follow-full-rust-final.log`.
- `cargo clippy --locked --offline --all-targets -- -D warnings` and
  `cargo fmt --all --check` passed on final Rust source.
- `make man html quality-docs-surface man-check html-check quality-ai-workflow
  quality-architecture` passed. Generated 88 manpages and 284 HTML pages;
  architecture has zero hard failures and the same five size-warning owners.
  Feature-matrix check passed. Log: `/tmp/codex/grafana-utils/follow-quality-final.log`.
- `git diff --check` passed. Trace archive rotation retains the configured limits.
- Actual isolated Grafana health versions 12.0.10 and 13.2.1 verified. Floor:
  classic available; v1/v2/Playlist/portable-v2 unserved errors reported explicitly.
  Current: all dashboard read probes available; Playlist list/get match direct API;
  portable native export/import preserves exact elements/layout/CustomVariable spec.
- Additional real 13.2.1 acceptance preserves a nonempty folder and repo ownership
  annotation; replacing a managed resource is rejected with unchanged server spec
  and resourceVersion. Disposable containers were removed by recorded-ID traps.
  Logs: `/tmp/codex/grafana-utils/follow-live-acceptance-validated.log` and
  `/tmp/codex/grafana-utils/follow-live-folder-ownership.log`.
- Live feature acceptance predates the root color/help-only repairs; current-source
  CLI/parser/help tests verify those repairs. No new live mutation behavior changed.
- No Python implementation changes or new Python/live-five-lane rerun. Reuse prior
  evidence only for unchanged surfaces; no production, physical TUI, OS-keyring or
  real Git Sync repository acceptance is claimed. Alert preflight cannot lock
  concurrent edits or automatically roll back earlier writes.
- Reviewed 173-path scope committed locally by Luna; no push or deployment.
