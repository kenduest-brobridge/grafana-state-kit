# Datasource TUI Secret-Redaction Coverage

## Status

- State: complete
- Date: 2026-07-12
- Priority: root `todo.md` P0 TUI completion
- Scope: Rust TUI safety tests and registry evidence only

## Goal

Prove datasource detail and review projections never render direct credential
values when a live response, local artifact, or snapshot contains hostile
`password`, `token`, `apiKey`, or nested `secureJsonData.*` fields.

## Requirements

- Keep the existing operator-safe behavior: render stable identity and
  non-secret configuration facts, allow secret field names where the existing
  review contract requires them, and never render credential values.
- Cover the live `datasource browse` detail and review projections, the local
  `datasource list --interactive` detail projection, and the
  `snapshot review --interactive` browser projection.
- Keep review-diff field filtering evidence explicit for direct password,
  token, API-key, and nested secure-JSON paths.
- Update the TUI registry with the focused datasource test path when the
  registry's existing test evidence does not name it.

## Constraints

- Treat `rust/src/` as the only implementation/test surface for this slice.
- Do not add a new CLI flag, change output wording, or update generated docs.
- Do not expose the test sentinel values in failure messages or tracker notes.
- Preserve the user's uncommitted full-project analysis roadmap without
  changing its analysis-only scope.
- Do not inspect `rust/target`.

## Assumptions

- The user's direction to begin coding selects the first incomplete P0 slice
  in the root tracker; the separate full-project roadmap remains an active,
  read-only assessment task.
- Existing production projections are expected to be safe. If a new test
  demonstrates a leak, add the smallest shared redaction fix in the same
  layer and retain the test as the regression guard.

## Acceptance Checks

- A live browse item containing direct and nested secret sentinels renders
  safety evidence but none of the sentinel values in either detail or review
  lines.
- A local interactive-list datasource projection excludes all direct and nested
  secret sentinel values.
- A snapshot review datasource item excludes all direct and nested secret
  sentinel values while retaining the review-required safety evidence.
- Changed-field review output excludes direct password, token, API-key, and
  `secureJsonData.*` paths.
- Datasource delete confirmation carries only name, UID, and numeric ID; the
  edit dialog carries only name, URL, access mode, and default status. Neither
  confirmation model receives the datasource details map.
- `cargo test --quiet datasource` and the focused snapshot-review test pass.
- `python3 -m unittest -v scripts.test_tui_inventory_report` and
  `python3 scripts/tui_inventory_report.py --registry-check` pass if the
  registry changes.

## Execution Plan

- [x] Add the focused live and local pure-projection tests in
      `rust/src/commands/datasource/browse/support.rs`; use distinct sentinel
      values and assert only their absence from rendered lines.
- [x] Extend the changed-field review test in
      `rust/src/commands/datasource/browse/support.rs` to cover direct
      password, token, and API-key paths as well as `secureJsonData.*`.
- [x] Add a snapshot browser-item test in
      `rust/src/commands/snapshot/tests_review_browser_rust_tests.rs` that
      exercises the shared datasource detail/review projection with hostile
      direct and nested secret values.
- [x] Add `rust/src/commands/datasource/browse/support.rs` to the relevant
      datasource entries in `scripts/contracts/tui-registry.json` if needed
      for concrete test-path evidence.
- [x] Verify the existing delete/edit confirmation models and render tests do
      not carry or project raw datasource detail fields.
- [x] Run the focused Rust tests. If a sentinel appears, keep the test red,
      implement the minimal shared projection fix, and rerun the focused test
      green before continuing.
- [x] Run the applicable datasource, snapshot, TUI-registry, formatting, and
      diff checks; record exact evidence here and update the concise index.

## Evidence Log

- 2026-07-12: CodeGraph located the shared detail/review boundary at
  `datasource_browse_support::{detail_lines, review_lines,
  datasource_browser_detail_lines_from_details}`. Snapshot review calls the
  local-detail and review functions directly, so coverage should stay at these
  shared projections plus one snapshot browser integration test.
- 2026-07-12: Baseline `cargo test --quiet` completed with 1,924 passed and
  one ignored test before the safety coverage changed.
- 2026-07-12: Added focused live/local/detail/review/diff and snapshot
  assertions for direct and nested credential values. The existing shared
  projections required no production behavior change.
- 2026-07-12: Focused filters for the new local detail, live review,
  changed-field review, and snapshot browser tests each passed one test.
- 2026-07-12: Final `cargo test --quiet` completed with 1,925 passed and one
  ignored test. `cargo fmt --all --check`, the 14-test TUI registry suite, the
  registry check, and `make quality-docs-surface` passed.
- 2026-07-15: Closeout inspection confirmed `PendingDelete` carries only name,
  UID, and numeric ID, while `EditDialogState` copies only name, URL, access,
  and default status. Existing confirmation rendering and input tests exercise
  the applicable safe control paths without passing the raw details map.
- 2026-07-15: Fresh closeout verification passed 49 secret-focused Rust tests
  with one ignored, all 1,925 Rust tests with one ignored, Rust formatting, 14
  TUI registry tests, the live registry check, docs-surface, architecture, AI
  workflow, and diff checks. Architecture reported only two pre-existing
  dashboard file-size warnings.
