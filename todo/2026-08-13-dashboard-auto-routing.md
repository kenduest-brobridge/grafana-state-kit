# Dashboard Auto Routing and Native Export Shortcut

## Status

- State: completed (static and request-level verification)
- Date: 2026-08-13
- Scope: Rust dashboard export/import CLI, tests, contracts, and bilingual docs
- Compatibility floor: Grafana 12.0+

## Goal

Make the common dashboard backup and restore workflow simple without hiding
schema or transport decisions. Preserve existing explicit modes and avoid any
lossy conversion between Classic, resource-v1, and resource-v2 dashboards.

## Public contract

### Export

- Keep the existing default export unchanged: Classic `raw/`, `prompt/`, and
  `provisioning/` artifacts remain the no-flag behavior on Grafana 12 and 13.
- Add `--native` as a shortcut for requesting every stable dashboard resource
  lane served by the target, equivalent to `--resource-format all`.
- Keep `--resource-format none|v1|v2|all` and `--resource-namespace` as expert
  overrides.
- `--native` and an explicit `--resource-format` are mutually exclusive.
- Do not make optional resource permissions a prerequisite for the existing
  default Classic export.

### Import

- Add `auto` to `--input-format` and make it the `dashboard import` default.
- Preserve explicit `raw`, `provisioning`, and `resource-v2` behavior exactly.
- Classify input before mutation from trusted export indexes, recognized lane
  directories, and exact JSON resource identity.
- For a combined export containing both resource-v2 and Classic raw artifacts:
  - prefer resource-v2 when its collection endpoint is served;
  - choose the independent raw artifact only when the v2 probe returns `404`;
  - never treat `401`, `403`, malformed success, or `5xx` as unsupported.
- For resource-v2-only input, a missing v2 endpoint remains an error and never
  falls back or converts the document.
- Infer a single non-empty source namespace for auto-selected resource-v2 when
  `--resource-namespace` is omitted; require an override if source namespaces
  conflict. Default to `default` only when documents omit namespace.
- Emit the auto-routing decision to stderr so JSON stdout remains valid.
- Keep multi-org/classic-only routing on Classic artifacts; resource-v2 remains
  single-namespace.

## Non-goals

- Resource-v1 mutation/import. Export remains supported, but write support
  requires a separate stable reconciliation and live-acceptance slice.
- Semantic conversion among Classic, resource-v1, and resource-v2.
- Mixed-lane mutation in one run.
- Beta APIs, upstream-main behavior, or version-number-only routing.
- Changing the existing default export directory layout.

## Implementation plan

### DAR-001 - Parser and compatibility contract

- [x] Add the `auto` import value and make it the import-only default.
- [x] Add `--native` export shortcut with conflict validation.
- [x] Preserve defaults for non-import commands that reuse the input-format
      type.
- [x] Add parser/help regressions before production changes.

### DAR-002 - Pure source inventory

- [x] Add a deterministic, filesystem-only inventory for raw, provisioning,
      resource-v1, and resource-v2 candidates.
- [x] Prefer trusted root index paths when present and reject escaping paths.
- [x] Reject ambiguous direct directories and resource-v1-only inputs with an
      actionable message.
- [x] Add fixtures for direct lanes, combined roots, malformed JSON, symlinks,
      and conflicting resource namespaces.

### DAR-003 - Capability-safe auto routing

- [x] Probe resource-v2 with a bounded collection read and validate exact
      `DashboardList` identity.
- [x] Classify only `404` as unsupported; propagate auth/server/contract errors.
- [x] Cache the successful v2 preflight for the import run so apply does not
      issue a second capability probe.
- [x] Select Classic raw only when the combined source actually contains its
      own raw artifact.
- [x] Render one stable decision summary without contaminating JSON stdout.

### DAR-004 - Documentation and compatibility evidence

- [x] Update English and Traditional Chinese command docs and help examples.
- [x] Update command-surface contracts and generated man/HTML artifacts.
- [x] Run focused tests, full Rust tests, Rust quality, docs/contracts checks,
      and generated-output checks.
- [x] Review the complete scoped diff independently and fix confirmed issues.

## Acceptance

- [x] `dashboard export` without `--native` is byte-for-byte behavior compatible
      with the prior Classic selection and layout.
- [x] `dashboard export --native` selects the existing stable resource discovery
      behavior without requiring API-version knowledge.
- [x] `dashboard import --input-dir <raw>` works without `--input-format` in
      automated routing tests and chooses the Grafana 12-compatible Classic
      endpoint.
- [x] `dashboard import --input-dir <resource-v2>` works without
      `--input-format` when v2 is served.
- [x] A combined Classic/resource-v2 root chooses v2 when available and its own
      raw lane only on a true v2 `404`.
- [x] No resource wrapper is posted to `/api/dashboards/db`.
- [x] Explicit legacy command lines retain their existing behavior.
- [x] All scoped automated checks pass; any unavailable live Grafana 12/13 boundary is
      reported explicitly rather than described as verified.
- [ ] Live acceptance against released Grafana 12.x and 13.x images remains
      open under the stable compatibility matrix plan.

## Evidence

- Baseline before implementation: `cargo test --quiet` passed 1,984 tests with
  1 ignored on the pre-existing dirty worktree.
- Existing unrelated worktree changes are preserved. Implementation is being
  performed in place because uncommitted dashboard API contract hardening is a
  required dependency and would be absent from a fresh worktree at `HEAD`.
- Final focused and full Rust verification: `cargo test --quiet` passed 1,971
  tests in the main Rust test binary, plus the auxiliary test binaries (7 and
  31 passed), with 1 ignored and 0 failures; `make quality-rust` passed.
- Final documentation and contract verification passed:
  `make quality-docs-surface man-check html-check quality-ai-workflow
  quality-output-contracts quality-rust-feature-matrix`.
- `make quality-rust-feature-matrix-full` still reports the pre-existing
  `--no-default-features probe unexpectedly succeeded` policy mismatch; neither
  `rust/Cargo.toml` nor `scripts/check_rust_feature_matrix.py` is changed by
  this work.
- Independent review found and fixed three issues: omitted namespaces now
  participate in conflict detection as `default`, malformed v2 documents stay
  in the resource-v2 error path, and known lane directories reject symlinks.
- No live Grafana 12.x/13.x server was available in this run; compatibility is
  covered by the request/inventory tests and static contracts, not live
  acceptance.
