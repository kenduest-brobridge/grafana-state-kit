# Project Review: Bugs and Improvements

## Status

- State: complete
- Date: 2026-07-18
- Scope: evidence-backed review of the Rust-first CLI/TUI, Grafana API
  integrations, Python compatibility layer, tests, public contracts, and
  generated documentation after the current dashboard import work

## Requirements and constraints

- Review the live branch and current tracker before relying on historical
  findings.
- Treat prior completed review items as regression baselines, not as proof that
  the current branch is defect-free.
- Report only reproducible bugs or improvements with a concrete code/path
  reference and impact assessment.
- Preserve the clean worktree; this pass is review-only unless a later request
  explicitly authorizes fixes.
- Do not inspect Rust build outputs under `rust/target`.

## Review plan

- [x] Map the current Rust command/API architecture and changed surfaces.
- [x] Trace high-risk dashboard import/export/history, capability routing, and
      auth/error/output paths for correctness and contract drift.
- [x] Review secondary Python behavior, public CLI/docs contracts, generated
      artifacts, and test coverage for user-visible gaps.
- [x] Run focused static/contract checks and targeted tests needed to confirm
      suspected findings without treating a broad green suite as proof.
- [x] Record ranked findings, evidence, and follow-up recommendations; update
      this tracker and `todo/todo.md` with the honest boundary.

## Findings

### High: Rust quality gate is currently red

- `make quality-rust` fails on seven clippy diagnostics under `-D warnings`:
  `question_mark` in `rust/src/commands/access/service_account_workflows_sync.rs:219`,
  `some_filter` in `rust/src/commands/dashboard/browse/state.rs:516`, and five
  `useless_borrows_in_formatting` diagnostics in
  `rust/src/commands/status/overview/section_rows.rs:251-260`.
- `.github/workflows/ci.yml` makes `rust-quality` a prerequisite for live smoke
  and release jobs, so this is a delivery blocker even though the Rust tests
  pass.
- Next action: apply semantics-preserving clippy fixes and keep the strict gate.

### High: GitLab CI/CD no longer matches the repository layout or artifacts

- `.gitlab-ci.yml:52` reads a root `pyproject.toml`, but the maintained file is
  `python/pyproject.toml`; the static-analysis commands at lines 110-113 also
  target a missing root `grafana_utils` path.
- The Python package jobs run Poetry/build from the repository root at lines
  159-161 and 183-185, so they cannot discover the Python project.
- The Rust artifact paths at lines 213-214 and 234-235 request
  `grafana-access-utils`, but the current Cargo manifest exposes only the
  `grafana-util` binary.
- On the current `dev` branch, the policy additionally requires preview
  version suffixes while both manifests are `0.13.2`.
- Next action: either remove/deprecate this stale pipeline or update its working
  directories, binary artifact list, and version policy together, then validate
  the YAML in CI.

### Medium: `resource get alert-rules/...` ignores `--api-mode`

- `rust/src/commands/resource/runtime.rs:238-242` hardcodes
  `ResourceApiMode::Auto` and supplies no legacy closure, regardless of the
  caller's `api_mode`.
- Consequently `--api-mode v1`, `v2`, or `v1beta1` does not reject unsupported
  alert-rule modes as `resource list` does; it still calls the legacy alert-rule
  endpoint. This makes explicit mode selection misleading and inconsistent.
- Next action: call `ensure_legacy_only_mode` and resolve with the passed mode;
  add request/error tests for all unsupported modes.

### Medium: text resource lists discard every item

- `rust/src/commands/resource/render.rs:205-211` renders only the kind and
  count for `status resource list ... --output-format text`; `document.items`
  is never emitted.
- Table, JSON, and YAML retain the rows, so the same command silently loses
  inventory identities when text is selected.
- Next action: render one concise identity/title line per item, or explicitly
  document text as summary-only and add a contract test.

### Medium: dynamic App Platform path segments are not encoded

- `rust/src/grafana/api/app_platform.rs:51-61` interpolates `namespace` and
  `name` directly into URL paths. `status resource` accepts both from CLI
  input, while `rust/src/grafana/http.rs:102-118` parses the assembled string
  as a URL.
- Characters such as `?`, `#`, `%`, or `/` can change the route/query or make
  the request invalid instead of remaining one path-segment value. The newer
  dashboard resource helper already percent-encodes these segments, so the
  generic client is inconsistent.
- Next action: centralize RFC 3986 path-segment encoding (or strictly validate
  Kubernetes-style names) and add namespace/name edge-case request tests.

### Medium: Python quality enforcement can be falsely green

- `scripts/check-python-quality.sh:40-50` treats missing `ruff`, `mypy`, and
  `black` modules as successful skips. In this checkout, `make quality-python`
  passed while warning that all three checks were skipped.
- `.github/workflows/ci.yml` has no Python test or quality job, and the GitLab
  pipeline explicitly says it omits unit tests. The public Python entry point
  in `python/pyproject.toml:24-25` therefore has no consistently enforced
  static/test lane.
- Next action: make the maintained Poetry environment the canonical Python
  quality runner, fail when required tools are unavailable, and add a Python
  test/quality job to the supported CI pipeline.

## Lower-priority improvements

- The Python compatibility surface still rejects advertised interactive paths
  for datasource listing and dashboard summary at
  `python/grafana_utils/datasource/workflows.py:1768-1771` and
  `python/grafana_utils/dashboards/inspection_workflow.py:431-434`; either
  implement parity or make the Rust-only boundary explicit in docs.
- Native dashboard history/export discovery could validate returned item
  identity (`metadata.name`, namespace, apiVersion, kind) against the requested
  selector instead of relying only on server-side selectors.
- App Platform capability discovery should validate the collection's `items`
  shape and expected resource kind/version in one shared helper; current
  discovery accepts any JSON object before later list parsing.

## Verification evidence

- `cd rust && cargo test --quiet`: 1,972 passed, 1 ignored.
- Focused `cargo test --quiet resource`: 146 passed.
- `make test-python`: 1,230 passed, 2 skipped under the system Python.
- `poetry run ... unittest`: blocked by the existing Poetry environment missing
  `yaml`; `python/pyproject.toml` correctly declares `PyYAML>=6,<7`.
- Passed: `make quality-docs-surface`, `make quality-ai-workflow`, `make
  man-check`, `make html-check`, `make quality-output-contracts`, `make
  quality-architecture` (two size warnings), `make quality-rust-feature-matrix`,
  `make schema-check`, and `git diff --check`.
- No live Grafana server/runtime verification was performed in this review.
- No production code was changed; only this review tracker and its active index
  were updated.

## Acceptance

The review is complete when confirmed findings are ranked by severity, each has
reproduction/evidence and a proposed next action, unconfirmed risks are marked
as such, and the active index reflects the resulting follow-up state.
