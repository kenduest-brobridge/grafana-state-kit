# Todo

## Active

- [ ] [2026-10-11 deep project review and fixes](2026-10-11-deep-project-review.md) -
  Review maintained Rust behavior, shared contracts and delivery surfaces;
  reproduce and repair confirmed defects, validate, and commit locally.

- [x] [2026-09-10 stable upstream review and execution](2026-09-10-upstream-stable-review.md) -
  Verify Grafana 13.2.1, fix confirmed compatibility issues, add regression
  coverage, review and commit; retain Grafana 12.x compatibility.

- [x] [2026-09-05 compatibility and screenshot retirement review](2026-09-05-compatibility-retirement-review.md) -
  Fix smoke artifact selection, cleanup and exception boundaries; verify code,
  documentation, and all five live compatibility lanes.

- [x] [2026-09-05 Grafana 12 UID mutations and screenshot retirement](2026-09-05-grafana12-uid-and-screenshot-retirement.md) -
  Retire screenshot capture, establish the Grafana 12.0+ compatibility contract,
  use UID-only datasource mutations, and add released-version test lanes.

- [x] [2026-08-23 worktree review, fix, and commit](2026-08-23-worktree-review-fix-commit.md) -
  Review every current change, fix confirmed defects, verify with external
  build caches, and commit the completed concerns without mixing open roadmap
  work.

- [x] [2026-08-13 dashboard auto routing and native export shortcut](2026-08-13-dashboard-auto-routing.md) -
  Make import auto-detect Classic versus resource-v2 safely, add one native
  export shortcut, preserve Grafana 12 defaults, then review and verify.

- [ ] [2026-08-12 stable Grafana compatibility plan](2026-08-12-stable-grafana-compatibility-plan.md) -
  Preserve a Grafana 12.0+ compatibility floor, migrate datasource mutation to
  UID routes, test released 12.x/13.x lanes, and capability-gate new features.

- [x] [2026-08-12 upstream Grafana compatibility review](2026-08-12-upstream-grafana-compatibility-review.md) -
  Found required datasource UID migration and Grafana 13.1.3 test-matrix work;
  beta APIs and unreleased upstream-main behavior are excluded from the plan.

- [x] [2026-07-18 project review fixes](2026-07-18-project-review-fixes.md) -
  Fixed the confirmed runtime, URL, output, lint, CI/CD, Python quality, and
  interactive-review findings; full Rust/Python/docs verification passes.

- [x] [2026-07-18 project review: bugs and improvements](2026-07-18-project-review.md) -
  Review complete: Rust quality and GitLab delivery blockers, resource API-mode
  and text-output bugs, path encoding, and Python quality/coverage gaps are
  ranked with evidence; fixes remain separate follow-up work.

- [ ] [2026-07-17 dashboard import auto detection and server capability negotiation](2026-07-17-dashboard-import-auto-capability-negotiation.md) -
  Plan backward-compatible source classification, shared server discovery,
  explainable endpoint routing, and status reporting across Grafana 12, 13,
  and later compatible API surfaces.

- [x] [2026-07-17 dashboard resource-v2 import review and fix](2026-07-17-dashboard-v2-resource-import-review-fix.md) -
  Fixed production dispatch, strict response identity, safe discovery,
  idempotency, output and interactive contracts, then reverified unit, full
  Rust, generated-doc, contract, registry, and live Grafana 13.1 behavior.

- [x] [2026-07-17 dashboard schema v2 resource API import](2026-07-17-dashboard-v2-resource-import.md) -
  Complete namespace-aware native v2 import with guarded reconciliation,
  searchable interactive review, truthful partial-run reporting, bilingual
  generated docs, and positive/negative Grafana 13.1 live evidence.

- [x] [2026-07-12 full project analysis and improvement roadmap](2026-07-12-full-project-analysis-roadmap.md) -
  Produce one complete evidence-backed project assessment and atomic backlog
  through phased Rust-first architecture, correctness, security, delivery,
  compatibility, documentation, and operational-readiness review.

- [x] [2026-07-12 datasource TUI secret-redaction coverage](2026-07-12-datasource-tui-secret-redaction-coverage.md) -
  Prove live browse, local list, and snapshot-review datasource detail and
  review panes never render direct or nested credential values.

- [x] [2026-07-12 Grafana 13 review follow-up fixes](2026-07-12-grafana13-review-followup-fixes.md) -
  Correct resource history semantics, native-only export routing, Cloud
  namespace selection, and Git Sync symlink traversal found after merge.

- [x] [2026-07-10 Grafana 13 all improvements](2026-07-10-grafana13-all-improvements.md) -
  Implement the confirmed dashboard resource, Python packaging, datasource,
  TUI governance, App Platform history/folder/notification/secret metadata,
  and Git Sync 13.1 improvements with compatibility-first TDD slices.

- [x] [2026-07-10 project-wide Grafana 13 review](2026-07-10-project-wide-grafana13-review.md) -
  Re-review the Rust-first backend, CLI/TUI and browser-facing surfaces,
  secondary Python compatibility layer, tests, public contracts, and Grafana
  13 integration opportunities; record only evidence-backed bugs and ranked
  improvements.
