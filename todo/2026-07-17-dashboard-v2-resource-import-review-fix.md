# Dashboard Resource-v2 Import Review and Fix

## Status

- State: complete
- Date: 2026-07-17
- Scope: review the uncommitted resource-v2 import implementation and fix only
  confirmed correctness, safety, CLI-contract, documentation, or test gaps

## Requirements and constraints

- Review the current working tree against
  `2026-07-17-dashboard-v2-resource-import.md`.
- Preserve the classic raw/provisioning import behavior.
- Treat unexpected v2 API responses, mutation confirmation, ownership
  detection, changed-field output, partial-run reporting, and secret redaction
  as high-risk paths.
- Add regression tests before or with every confirmed behavior fix.
- Keep generated documentation derived from its source files.

## Review plan

- [x] Independently review the complete working-tree diff against the plan.
- [x] Trace resource-v2 discovery, reconciliation, interactive selection, and
      sequential apply for unsafe or inconsistent edge cases.
- [x] Verify CLI output-format and output-column behavior remains compatible.
- [x] Patch confirmed findings and add focused regression coverage.
- [x] Run focused tests, the full Rust TUI suite, docs/contracts checks,
      formatting, and whitespace validation.
- [x] Record findings, fixes, and any remaining boundary honestly.

## Acceptance

The review is complete when all confirmed critical and important findings are
fixed, focused regression coverage passes, the full relevant verification
stack is green, and this tracker plus `todo/todo.md` reflect the live state.

## Evidence

Confirmed and fixed findings:

- Production `import_dashboards_with_client` did not route resource-v2 input to
  the native implementation; a real local HTTP regression now proves the
  public client sends probe/get/create only to `/apis/.../v2`.
- The active collection probe accepted the wrong API version or kind, while
  item GET, POST, and PUT accepted weak or empty resource identities. The
  probe now validates list API version/kind, and item responses additionally
  validate namespace/name.
- The union output-column parser could panic classic rendering or silently
  erase v2 table columns; lane-specific validation now fails before discovery
  or network access.
- Combined-root traversal and standalone symbolic links could escape the
  selected input root; canonical containment and symlink rejection now close
  both paths.
- Ownership detection substring-matched arbitrary metadata text; it now uses
  explicit managed/provisioning fields and known Grafana annotation keys.
- Interactive confirmation discarded terminal cancellation and dry-run omitted
  non-mutating rows; cancel now exits and dry-run retains every reviewed row.
- Partial-write errors could echo credentials; the rendered error is redacted
  while preserving its HTTP status and truthful operation partition.
- Resource-v2 JSON was a bare array despite the documented single-document
  contract; it now includes kind, mode, dashboards, and action summary.
- Live Grafana 13.1 added schema defaults after create, causing false updates;
  comparison now normalizes those exact v2 defaults and a live repeat reports
  `no_change`.
- A final pre-commit review found that unknown `managedBy` values could be
  overwritten; they now classify as `managed-unknown` and remain blocked.
- Mutation normalization now removes Grafana's server-owned
  `grafana.app/deprecatedInternalID` label instead of replaying a stale value.
- `--output-format` is now enforced as dry-run-only by both Clap and runtime
  validation, preventing a silently ignored output selector on live mutation.
- Resource-v2 interactive list and status surfaces now redact credential-shaped
  namespace, name, and search-status text as well as detail fields.

The initial independent reviewer verdict was not ready to merge, with two
critical and seven important findings. A separate final pre-commit review found
four additional important findings. All code findings above are addressed; the
final verification results are:

- Focused resource-v2 suites: import `22`, API `6`, interactive `10`, renderer
  `4`, and apply `5` tests passed.
- Fresh full Rust default and TUI-enabled runs each completed with `1,972`
  tests passed and `1` ignored across five suites.
- `cargo fmt --all --check`, output contracts, docs surface, AI workflow,
  generated man/HTML checks, TUI registry inventory, and `git diff --check`
  passed.
- Global `cargo clippy --all-targets --features tui -- -D warnings` has no
  resource-v2 findings. It remains non-zero on seven pre-existing findings in
  access service-account sync, dashboard browse state, and status overview
  formatting; those unrelated files were not changed in this review slice.
- Live Grafana OSS 13.1 production-client acceptance passed create,
  schema-default-aware `no_change`, blocked update, permitted update, history
  message, DELETE `200`, and follow-up `404` cleanup.
