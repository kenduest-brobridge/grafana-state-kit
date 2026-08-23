# Project Review Fixes

## Status

- State: complete
- Date: 2026-07-18
- Source review: [project review](2026-07-18-project-review.md)

## Requirements

- Fix every confirmed finding from the project review:
  Rust clippy gate failures, stale GitLab CI paths/artifacts, alert-rule API
  mode routing, text resource-list output, URL path-segment handling, and
  Python quality/test coverage.
- Close the lower-priority review gaps as well: Python interactive datasource
  and dashboard-summary paths, dashboard history identity checks, and strict
  App Platform discovery validation.
- Preserve public CLI contracts where they are already correct; add regression
  tests before production behavior changes.
- Keep the existing review tracker changes and do not inspect `rust/target`.

## Execution plan

- [x] Add failing Rust/Python regression tests for each behavior and contract
      fix.
- [x] Fix shared Rust URL encoding, App Platform validation, resource routing,
      text rendering, and dashboard history/export path handling.
- [x] Resolve the Rust clippy diagnostics without changing behavior.
- [x] Repair GitHub/GitLab Python quality/test and packaging lanes, including
      current project paths and the single Rust binary artifact.
- [x] Implement read-only Python local interactive review loops for datasource
      inventory and dashboard summaries.
- [x] Run focused red/green tests, then full Rust/Python suites and repository
      quality/documentation checks.
- [x] Update this tracker and the active index with evidence and any explicit
      boundary that remains.

## Acceptance checks

- New regression tests cover each repaired behavior and pass after the fixes;
  focused Rust slices were run red before their corresponding implementations.
- `make quality-rust`, `make quality-python`, `make test-python`, `cd rust &&
  cargo test --quiet`, and the supported docs/contract gates pass.
- GitHub and GitLab configuration references only maintained paths/artifacts
  and includes an enforced Python test/quality lane.
- No review finding remains undocumented or silently deferred.

## Completed changes

- Rust now shares RFC 3986 path-segment encoding across App Platform and native
  dashboard resource APIs, validates App Platform/dashboard collection
  identity and items, honors alert resource API modes, preserves resource-list
  text rows, and validates native history/export item identity.
- All seven clippy diagnostics are resolved; the strict Rust quality gate is
  green.
- GitHub and GitLab CI now use `python/`, run Python tests and required quality
  tools, validate the current version/binary surface, and parse as valid YAML.
- Python local datasource listing and dashboard summary `--interactive` modes
  now provide read-only filter/select JSON review loops.

## Verification evidence

- `make quality-rust`: 1,946 Rust library tests plus 7 and 31 auxiliary test
  suites passed, 1 ignored; fmt and clippy pass. Direct `cargo test --quiet`
  reports 1,984 passed, 1 ignored.
- `make quality-python`: 1,233 Python tests passed; compile, unittest, ruff,
  mypy, and black all pass.
- CI YAML parses successfully; GitLab `version-policy` succeeds on the current
  synchronized `0.13.2` dev checkout.
- Passed `quality-docs-surface`, `quality-ai-workflow`, `man-check`,
  `html-check`, `quality-output-contracts`, `quality-architecture` (two
  existing size warnings), `quality-rust-feature-matrix`, `schema-check`, and
  `git diff --check`.
- Live acceptance passed against Grafana `13.1.0` at `127.0.0.1:3000` with
  Basic auth: health, Rust/Python status, dashboard and datasource reads,
  native resource listing in automatic and explicit v2 modes, legacy alert
  resource listing, classic/v2 dashboard history, single-dashboard fetch, and
  dashboard export with history plus v2 resource artifacts. The live instance
  had no alert rules, and status correctly reported `partial-no-data`; these
  checks performed no Grafana mutations.
