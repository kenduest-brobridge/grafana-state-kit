# Dashboard Schema v2 Resource API Import

## Status

- State: complete
- Date: 2026-07-17
- Priority: High
- Scope: Rust-first dashboard import, App Platform v2 resource API, tests,
  contracts, and operator documentation
- Tracking ID prefix: `V2I`

## Goal

Allow a dashboard exported as a native `dashboard.grafana.app/v2` resource to
round-trip through `grafana-util dashboard import` without converting it into
classic dashboard JSON or sending it to the legacy `/api/dashboards/db`
endpoint.

The import must preserve the resource schema, select the App Platform
namespace explicitly, distinguish create from update safely, expose a useful
dry-run review, and retain the existing ownership protections for Git Sync and
file-provisioned dashboards.

## Current Gap

- `DashboardImportInputFormat` supports only `raw` and `provisioning`.
- `build_import_payload` classifies `dashboard.grafana.app/*` envelopes as
  resource documents and rejects them before transport.
- The current live import path posts only to `/api/dashboards/db`.
- Dashboard resource v1/v2 export, discovery, list, get, and history paths
  exist, but no dashboard resource create/update import lane exists.
- `status resource` is intentionally read-only and must not become a generic
  mutation surface as part of this work.

## Product Decision

Extend `grafana-util dashboard import` instead of creating a new top-level
command.

Proposed command shape:

```bash
grafana-util dashboard import \
  --profile prod \
  --input-dir ./dashboards/resource-v2 \
  --input-format resource-v2 \
  --resource-namespace default \
  --dry-run --table
```

Live create/update remains an explicit second step:

```bash
grafana-util dashboard import \
  --profile prod \
  --input-dir ./dashboards/resource-v2 \
  --input-format resource-v2 \
  --resource-namespace default \
  --replace-existing
```

## Requirements

### R1 - Explicit input and API lane

- Add `resource-v2` to `DashboardImportInputFormat`.
- Add `--resource-namespace <NAMESPACE>` for resource-v2 import, defaulting to
  `default` only when no explicit namespace or export metadata supplies one.
- Accept native JSON resource envelopes with exactly:
  - `apiVersion: dashboard.grafana.app/v2`;
  - `kind: Dashboard`;
  - a non-empty `metadata.name`;
  - an object-valued `spec`.
- Reject `metadata.generateName` in the first release because deterministic
  dry-run, duplicate detection, no-change comparison, and update routing all
  require a stable resource identity.
- Do not auto-convert schema v2 `spec.elements`, layouts, variables, or query
  kinds into classic dashboard JSON.
- Do not fallback to `/api/dashboards/db` when the v2 API is unavailable.
- Keep `status resource` read-only.

### R2 - Source discovery and export round-trip

- Accept either the exported `resource-v2/` directory or a combined dashboard
  export root whose index points to native v2 resource files.
- Ignore `_folder.json`, root indexes, resource list envelopes, history files,
  and non-dashboard resources.
- Reject mixed v1/v2 resource inputs instead of partially importing them.
- Reject duplicate `metadata.name` values before any live write.
- Preserve deterministic file and action ordering.
- Keep the existing raw and provisioning source resolution unchanged.

### R3 - Resource payload normalization

- Treat the selected CLI namespace as authoritative; do not trust a source
  document to route itself into another namespace.
- Preserve `apiVersion`, `kind`, `metadata.name`, supported labels and
  annotations, and the complete v2 `spec`.
- Remove server-owned metadata on create, including response UID, generation,
  creation/update timestamps, managed fields, and stale resource versions.
- For update, fetch the current resource and use the live concurrency token or
  resource version required by the served API contract.
- Map `--import-folder-uid` to the `grafana.app/folder` metadata annotation.
- Map `--import-message` to the resource history message annotation supported
  by the API.
- Never copy source namespace, created-by, updated-by, or server timestamps
  into a mutation request.

### R4 - API availability and transport

- Probe
  `/apis/dashboard.grafana.app/v2/namespaces/{namespace}/dashboards?limit=1`
  before the first mutation.
- Treat `404` as an unsupported v2 API and return an actionable error without
  legacy fallback.
- Preserve `401` and `403` as authentication or authorization failures.
- Create a missing dashboard with `POST` to the namespace collection.
- Update an existing dashboard with `PUT` to
  `/apis/dashboard.grafana.app/v2/namespaces/{namespace}/dashboards/{name}`.
- Percent-encode namespace and resource identity path segments through a
  shared path builder.
- Treat `409` as a concurrency conflict; report it and do not retry with a
  blind overwrite.
- Require an object-valued resource response and validate the returned kind,
  API version, and identity.

### R5 - Dry-run and reconciliation

- Fetch the live resource by `metadata.name` before deciding the action.
- Produce stable actions: `create`, `update`, `no_change`, `skip`, and
  `blocked`.
- `--dry-run` must never send `POST`, `PUT`, `PATCH`, or `DELETE`.
- Normalize server-owned metadata before diffing so timestamps, generation,
  managed fields, and resource versions do not cause false updates.
- Compare the complete v2 `spec` and operator-controlled metadata.
- `--replace-existing` permits update; without it, an existing resource is
  blocked or skipped with an explicit reason.
- `--update-existing-only` skips missing resources and never creates them.
- No delete/prune behavior is introduced in this slice.
- Dry-run JSON/table/text output identifies API version, namespace, resource
  name, source path, action, folder annotation, blocker, and changed fields.

### R6 - Ownership and mutation safety

- Reuse the existing dashboard ownership/provenance classifier where its
  evidence applies to resource responses.
- Block direct updates to Git Sync-managed or file-provisioned dashboards and
  tell the operator to update the owning source.
- Do not offer an override flag that bypasses managed-source ownership in the
  first release.
- Validate every input and complete the API/permission preflight before the
  first mutation, preventing validation-time partial writes.
- Process mutations sequentially in deterministic order.
- On a transport or conflict failure, stop the live run and report completed,
  failed, and not-attempted resources; do not claim transactionality or
  rollback.
- Never render authorization headers, tokens, or unexpected secret-bearing
  metadata values in review output.

### R7 - First-release compatibility boundary

The initial resource-v2 lane is single-namespace only. At runtime, reject
resource-v2 import combined with classic org/folder-routing options whose
semantics are not yet proven:

- `--org-id`;
- `--use-export-org`;
- `--only-org-id`;
- `--create-missing-orgs`;
- `--require-matching-export-org`;
- `--require-matching-folder-path`;
- `--ensure-folders`;
- `--strict-schema` and `--target-schema-version` until a v2-aware validator
  exists.

Continue to support `--import-folder-uid`, `--import-message`,
`--replace-existing`, `--update-existing-only`, `--interactive`, `--dry-run`,
output selection, and progress reporting where their behavior is covered by
resource-v2 tests.

Cross-org and automatic folder creation become separate follow-up work after
the single-namespace lane has live acceptance evidence.

### R8 - Documentation and public contracts

- Update English and zh-TW dashboard import documentation with the new input
  format, namespace rules, examples, incompatibilities, and failure guidance.
- Explain that resource-v2 import requires the new `/apis` endpoint and never
  falls back to the classic API.
- Update command help and parser tests.
- Update `scripts/contracts/command-surface.json` for the new enum value and
  public flag.
- Regenerate man and HTML documentation through their generators rather than
  editing generated output directly.
- Keep Foundation SDK, Git Sync, file provisioning, and API import ownership
  boundaries explicit.

## Non-Goals

- Converting dashboard schema v2 into classic schema.
- Importing `dashboard.grafana.app/v1` resource envelopes.
- Generic mutation commands under `status resource`.
- Resource deletion, pruning, or bulk rollback.
- Automatic folder resource creation.
- Multi-org routing or namespace derivation across a combined export.
- Bypassing Git Sync or file-provisioning ownership.
- YAML input in the first implementation slice.
- Python parity unless a maintained packaging or shared-contract requirement
  is discovered during implementation.

## Atomic TODO List

### V2I-001 - Lock CLI and compatibility contracts

- [x] Add failing parser tests for `--input-format resource-v2` and
      `--resource-namespace`.
- [x] Add runtime validation tests for every incompatible classic routing or
      validation flag.
- [x] Extend `DashboardImportInputFormat` and `ImportArgs` only after the tests
      demonstrate the intended public contract.
- [x] Keep raw and provisioning parser/help snapshots unchanged except for the
      intentional new values.

**Acceptance:** CLI parsing selects one explicit v2 lane, incompatible flags
fail before file discovery or network access, and existing import invocations
retain their current defaults.

### V2I-002 - Add resource-v2 source discovery

- [x] Add fixtures for a standalone resource-v2 directory and a combined
      export root with `resource_v2_path` index entries.
- [x] Discover only native v2 dashboard resources in deterministic order.
- [x] Reject duplicate identities and mixed API versions, ignore list envelopes, and reject
      malformed resource documents before transport.
- [x] Keep export-root temporary ownership alive for the full import run.

**Acceptance:** a native v2 export can be loaded without conversion, while
ambiguous or mixed input fails before any Grafana request.

### V2I-003 - Build safe resource mutation payloads

- [x] Add pure create/update normalization helpers.
- [x] Strip server-owned metadata and source namespace fields.
- [x] Preserve the full schema v2 `spec` and permitted operator metadata.
- [x] Apply folder and history-message annotations from CLI options.
- [x] Add golden tests proving no v2 elements, layouts, variables, or queries
      are lost or rewritten.

**Acceptance:** normalized create and update payloads are deterministic,
schema-preserving, and free of stale server-owned metadata.

### V2I-004 - Add v2 API client operations

- [x] Reuse or extract the existing resource-lane path builder.
- [x] Add availability probe, get-if-exists, create, and update operations.
- [x] Validate object responses and returned resource identity.
- [x] Add request-capture tests for method, encoded path, namespace, payload,
      `404`, `401`, `403`, malformed response, and `409` behavior.

**Acceptance:** create uses the v2 collection endpoint, update uses the named
resource endpoint, and no v2 operation reaches `/api/dashboards/db`.

### V2I-005 - Implement dry-run reconciliation

- [x] Build create/update/no-change/skip/blocked actions from desired and live
      normalized resources.
- [x] Ignore server-owned metadata noise while detecting actual spec and
      operator-metadata differences.
- [x] Reuse ownership/provenance evidence and block source-managed updates.
- [x] Extend text, table, and JSON review output with v2 identity and namespace
      fields without breaking classic output contracts.
- [x] Prove dry-run sends read requests only.

**Acceptance:** dry-run accurately previews every resource action and cannot
mutate Grafana under any output or interactive mode.

### V2I-006 - Implement guarded live apply

- [x] Run complete input, compatibility, API, permission, identity, and
      ownership preflight before the first write.
- [x] Apply creates and permitted updates sequentially.
- [x] Honor `--replace-existing` and `--update-existing-only` exactly.
- [x] Stop on the first write failure and emit a truthful partial-run summary.
- [x] Preserve the classic import path unchanged behind its existing input
      formats.

**Acceptance:** successful runs create/update only the reviewed resources;
blocked, conflicting, or failed runs never fall back to the legacy API or
overstate completion.

### V2I-007 - Complete interactive and operator evidence

- [x] Show v2 source path, namespace, API version, resource name, folder,
      ownership, action, blocker, and changed fields in interactive review.
- [x] Keep confirmation and cancel controls distinct for mutation-capable
      flows.
- [x] Cover search, repeat search, detail scrolling, footer accuracy, and
      no-TTY fallback where the existing dashboard import TUI supports them.
- [x] Ensure review and failure output never renders credentials or request
      authorization data.

**Acceptance:** interactive review exposes the same safety decision as dry-run
JSON without adding an unreviewed mutation path.

### V2I-008 - Update docs, contracts, and generated artifacts

- [x] Update English and zh-TW dashboard import source documentation.
- [x] Update CLI help examples and the command-surface contract.
- [x] Regenerate man pages and HTML documentation.
- [x] Update `docs/internal/ai-status.md` and `docs/internal/ai-changes.md`
      because this adds a meaningful API and architecture lane.
- [x] Keep the active todo index synchronized with implementation status.

**Acceptance:** public documentation clearly separates classic and v2 import,
all generated outputs match their sources, and contract checks report no
drift.

### V2I-009 - Run live Grafana acceptance

- [x] Confirm the target Grafana serves
      `dashboard.grafana.app/v2` in the intended namespace.
- [x] Dry-run and create a fixture dashboard in a disposable namespace/folder.
- [x] Re-run dry-run and verify `no_change`.
- [x] Change a v2 element and verify update is blocked without
      `--replace-existing`, then succeeds with it.
- [x] Verify folder annotation and import message behavior.
- [x] Verify unsupported API, insufficient permission, ownership blocker, and
      `409` conflict evidence.
- [x] Delete the disposable fixture through an explicitly approved cleanup
      path outside this import feature.

**Acceptance:** live evidence proves create, no-change, update, folder,
history, permissions, unsupported-instance, ownership, and conflict behavior
without touching production-managed dashboards.

## Suggested Implementation Order

1. `V2I-001` CLI contract.
2. `V2I-002` source discovery.
3. `V2I-003` payload normalization.
4. `V2I-004` API operations.
5. `V2I-005` dry-run reconciliation.
6. `V2I-006` guarded live apply.
7. `V2I-007` interactive evidence.
8. `V2I-008` docs and generated artifacts.
9. `V2I-009` live acceptance.

Do not start live mutation work before the CLI, source, payload, transport, and
dry-run contracts are green.

## Suggested Commit Groups

1. `feature:` CLI contract, source discovery, payload normalization, and their
   focused tests.
2. `feature:` v2 API primitives, dry-run reconciliation, ownership checks,
   guarded live apply, and request-capture tests.
3. `test:` interactive, failure, and output-contract regression coverage.
4. `docs:` English/zh-TW sources, contracts, generated docs, and tracker
   closeout.

Use red-green TDD within each implementation group, but commit only after its
focused tests are green. Do not mix unrelated dashboard TUI or
full-project-analysis changes into these commits.

## Validation Gates

Run the smallest relevant target first, then broaden:

```bash
cd rust && cargo test --quiet dashboard_v2_resource
cd rust && cargo test --quiet dashboard_import
cd rust && cargo fmt --all --check
cd rust && cargo clippy --all-targets -- -D warnings
cd rust && cargo test --quiet
python3 scripts/tui_inventory_report.py --registry-check
make quality-output-contracts
make quality-docs-surface
make quality-ai-workflow
make man-check
make html-check
```

For live acceptance, record the Grafana version/edition, served API versions,
namespace, authentication type, exact underlying command, response status,
created fixture identity, and cleanup result. Redact tokens and credentials.

## Documentation References

- Grafana Dashboard HTTP API:
  <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/dashboard/>
- Grafana provisioning and dashboard v2 resource format:
  <https://grafana.com/docs/grafana/latest/administration/provisioning/>

## Completion Boundary

This plan is complete when `V2I-001` through `V2I-009` are satisfied, the
classic import lane remains regression-green, resource-v2 export/import has
live round-trip evidence, generated documentation is current, and
`todo/todo.md` marks this item complete.

## 2026-07-17 Implementation Evidence

Completed in this pass:

- `V2I-001` CLI enum, namespace flag, and classic-option compatibility gates.
- `V2I-002` deterministic standalone/combined-root discovery with duplicate,
  mixed-version, generateName, and malformed-document rejection.
- `V2I-003` schema-preserving create/update normalization and golden fixtures.
- `V2I-004` encoded v2 probe/get/create/update transport with strict response
  identity checks and 401/403/404/409 coverage.
- Core `V2I-005` and `V2I-006` behavior: preflight reads, create/update/
  no-change/skip/blocked reconciliation, read-only dry-run, ownership blocker,
  sequential writes, and no classic fallback.
- `V2I-008` bilingual source docs, command contract, generated man/HTML output,
  and maintainer AI status/change records.
- Live Grafana 13.1 evidence: create, no-change, blocked update, permitted update,
  import history message, Grafana 12 unsupported response, and exact disposable
  fixture cleanup confirmed by a follow-up 404.

Completion evidence added in the final pass:

- `V2I-005` now has typed text/table/JSON output, selectable table columns,
  ownership/blocker fields, deterministic changed-field paths, and redaction.
- `V2I-006` now reports completed, failed, and not-attempted identities on the
  first failed write while preserving the original HTTP status and stating
  that no rollback was attempted.
- `V2I-007` now integrates a resource-v2 TUI review with search/repeat,
  scrolling, accurate footer states, explicit confirmation/cancel, TTY
  enforcement, and credential-redaction coverage.
- Live negative evidence covers Viewer-token `403`, repository ownership
  blocking, folder/message annotations, a forced stale-resourceVersion `409`,
  and follow-up `404` confirmation after disposable resource cleanup.
- Final review verification: five focused resource-v2 suites pass 47 tests;
  fresh full default and TUI-enabled Rust runs each pass 1,972 tests with 1
  ignored across five suites. Output-contract, docs-surface, AI-workflow,
  TUI registry, man-page, HTML-generation, formatting, and whitespace checks
  pass. Global clippy has seven known unrelated findings outside this slice and
  no resource-v2 findings.

### Auditable live acceptance record

- Target: Grafana OSS `13.1.0` (`Homebrew` build),
  `http://127.0.0.1:3000`, served API `dashboard.grafana.app/v2`, namespace
  `default`, Basic authentication. Credentials were supplied locally and are
  redacted here.
- Fixture: `codex-v2-review-20260717`, source directory
  `/private/tmp/grafana-utils-v2-review-live`.
- Create preview command:
  `grafana-util dashboard import --url http://127.0.0.1:3000 --basic-user admin --basic-password <redacted> --input-dir /private/tmp/grafana-utils-v2-review-live --input-format resource-v2 --resource-namespace default --dry-run --json`.
  Exit `0`; structured JSON reported `create=1`.
- Create command: the same command without `--dry-run --json`, with
  `--import-message review-fix-live-acceptance`. Exit `0`; the strict v2
  response identity check accepted name `codex-v2-review-20260717` in namespace
  `default`.
- Idempotency preview: repeating the preview after create exited `0` and
  reported `noChange=1` after Grafana-added schema defaults were normalized.
- Update preview: changing only `spec.title` exited `0` and reported one
  `blocked` row with `changedFields=[spec.title]` until `--replace-existing`
  was supplied.
- Update command: the live command with `--replace-existing --import-message
  review-fix-update` exited `0`; follow-up GET returned the updated title and
  annotation `grafana.app/message=review-fix-update`.
- Cleanup: direct DELETE of the disposable v2 resource returned HTTP `200`;
  follow-up GET returned HTTP `404`. The local fixture directory was removed.
- Earlier negative acceptance used a disposable Viewer service-account token
  and returned HTTP `403`; a stale `metadata.resourceVersion` PUT returned
  HTTP `409`; repository ownership was blocked before CLI mutation. Tokens and
  service-account artifacts were deleted after the checks.
