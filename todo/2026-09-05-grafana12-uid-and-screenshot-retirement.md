# Grafana 12 UID Mutations and Screenshot Retirement

## Status

- State: complete
- Date: 2026-09-05
- Parent plan: [stable Grafana compatibility](2026-08-12-stable-grafana-compatibility-plan.md)
- Scope: establish the Grafana 12.0+ compatibility contract, remove unhelpful
  screenshot functionality, and move datasource mutations to UID-only routes.

## User decision

- Screenshot capture is retired because it is not useful enough to keep as a
  supported CLI workflow.
- Grafana 12.x and 13.x remain the supported target lines; no beta API or
  Grafana-13-only requirement may replace a common Grafana 12 workflow.

## Requirements

### SCR-001 - Retire screenshot capture completely

- Remove screenshot commands, parser routes, runtime dispatch, browser/render
  dependencies, tests, command contracts, and operator documentation from both
  maintained Rust and Python CLI surfaces.
- Do not leave a hidden, unsupported screenshot command or a compatibility
  alias that silently performs browser automation.
- Preserve unrelated dashboard analysis, export, inspect, and local review
  workflows.
- Regenerate man and HTML documentation instead of hand-editing generated
  artifacts.
- Add regression tests proving the retired public command path is rejected and
  absent from CLI help/command contracts.

### SGC-000 - Declare the Grafana 12.0+ contract

- State Grafana 12.0 as the supported floor in one maintainer source of truth
  and link operator-facing compatibility guidance to it.
- Describe capability discovery as the mechanism for optional stable resources;
  only `404` means unavailable, never authentication, authorization, malformed
  success, or server errors.
- Ensure documentation does not imply that a Grafana 13-only route is required
  for standard workflows.

### SGC-001 - Make datasource mutations UID-only

- Use `/api/datasources/uid/{uid}` for default Rust and maintained Python
  update/delete/import/sync/browse mutation flows.
- On a UID route error, return it directly and do not retry a numeric-ID route.
- Preserve intentional legacy read-only modes separately; they are not part of
  this mutation migration.
- Add Rust and Python request-path regressions before production changes.

### SGC-002 - Released Grafana compatibility matrix

- Add explicitly pinned floor-12, current-12, and current-13 local/CI lanes
  only after the shared UID mutation contract is covered by unit tests.
- Record image tag and `/api/health.version` in live evidence.
- If containers or credentials are unavailable, keep live acceptance unchecked
  and report the boundary rather than inferring it from static tests.

## Execution checklist

- [x] Audit Rust, Python, public docs/contracts, generated artifacts, and live
      test scripts for screenshot and numeric datasource mutations.
- [x] Write failing retirement and UID-route regressions; verify each fails for
      the expected pre-change behavior.
- [x] Remove screenshot functionality and its now-unused dependencies.
- [x] Implement UID-only mutation handling in Rust and Python.
- [x] Update the Grafana 12.0+ compatibility contract and released-version
      matrix scripts/docs.
- [x] Run focused tests, full quality gates, generated-doc checks, and the
      released Grafana 12/13 matrix where available.
- [x] Independently review, fix confirmed findings, and commit by concern.

## Audited implementation map

- Rust retirement removes the dashboard screenshot parser/dispatch path,
  `screenshot*.rs` modules, the browser Cargo feature and optional Chromium
  dependencies, plus browser-flavored release/install/matrix lanes. Preserve
  dashboard variable parsing by moving its shared query parser out of the
  screenshot module.
- Python retirement removes `dashboards/screenshot.py`, its dashboard/unified
  CLI wiring and focused tests, then removes the now-unused Pillow dependency
  through Poetry-managed metadata/lock updates.
- Source command pages and their EN/zh-TW links are deleted. The man/HTML
  generators own removal of checked-in generated pages with prune enabled.
- UID mutation work is limited to create/update/delete/import/sync/browse
  writes. Read-only legacy datasource selection remains intentionally separate.

## Acceptance

- [x] `grafana-util dashboard screenshot` is absent from help and rejected as
      an unknown command in maintained Rust and Python CLI entry points.
- [x] Screenshot runtime/browser code and its package/feature dependencies have
      no maintained call path.
- [x] Default datasource update/delete/mutation workflows make only UID route
      requests and propagate UID errors without numeric fallback.
- [x] Docs, command contract, generated man/HTML, Rust/Python quality checks,
      and relevant focused tests pass.
- [x] Live Grafana 12.x/13.x results are recorded explicitly, including any
      unavailable environment boundary.

## Evidence

- The [subsequent review](2026-09-05-compatibility-retirement-review.md)
  supersedes the initial quality and live evidence below: it repaired Cargo
  artifact selection and cleanup, narrowed the org-delete exception further,
  restored independent org checks, and reran the full quality/live matrix.

- Rust live smoke passed against `grafana/grafana:12.0.10`, `12.4.9`, and
  `13.2.0`; Python datasource smoke passed against `12.0.10` and `13.2.0`.
  Each run reports the corresponding `/api/health.version`.
- Grafana 13.2.0 reproduces its upstream organization-delete `500`; only the
  dependent missing-org and org-replay fixture paths are skipped after that
  exact response is observed. Other server errors remain fatal.
- Independent final review found the original broad Grafana-13 skip and it was
  narrowed to the observed regression before re-running the Grafana 13 lane.
