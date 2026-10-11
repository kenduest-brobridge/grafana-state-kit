# Dashboard Import Auto Detection and Server Capability Negotiation

## Status

- State: partially implemented; remaining capability diagnostics are separate
- Date: 2026-07-17
- Scope: maximize Grafana 12, Grafana 13, and later compatibility by separating
  dashboard input classification from server API capability negotiation

## Reconciliation — 2026-10-11

- Import now defaults to `auto`; source classification, native-v2 probing,
  independent Classic raw fallback on 404, and routing diagnostics are shipped.
- Explicit `resource-v2` retains its native-only, single-namespace contract.
  Auto never converts a native dashboard into Classic JSON.
- `--native` export is implemented. These completed items are not new work.
- Shared operator-facing capability diagnostics remain open. Resource-v1 write
  support remains deliberately unsupported and is not a default priority.
- The checklist below describes the original proposal; use
  [the current development inventory](2026-10-11-development-convergence.md)
  to choose a new slice rather than restarting this entire plan.

## Original findings — 2026-07-17

- `dashboard import` currently defaults to `--input-format raw`.
- `raw` and `provisioning` use the classic `/api/dashboards/db` workflow.
- `resource-v2` must be selected explicitly and uses only
  `/apis/dashboard.grafana.app/v2/namespaces/<namespace>/dashboards`.
- Resource-v2 already probes its collection endpoint and refuses classic
  fallback on `404`, authentication failure, malformed response, or another
  transport error.
- `status live` already reads `GET /api/health` and exposes the raw
  `database`, `version`, and `commit` values under `discovery.instance`.
- `status resource describe dashboards` describes the CLI's supported
  selectors; it does not prove which dashboard APIs the connected Grafana
  instance serves.

## Design principles

1. Route by verified API capability and source schema, not by the Grafana
   major version alone. Version is diagnostic metadata, not proof that an
   endpoint is enabled, authorized, or compatible.
2. Never down-convert a `dashboard.grafana.app/*` resource to classic JSON as
   an implicit fallback. That can lose schema, ownership, namespace, and
   history semantics.
3. Preserve existing scripts: explicit `raw`, `provisioning`, and
   `resource-v2` behavior must remain stable.
4. Complete discovery, classification, validation, and capability negotiation
   before the first mutation.
5. Treat `401` and `403` as authentication or authorization failures, not as
   evidence that an API is unavailable. Treat malformed successful responses
   and `5xx` results as errors, not fallback signals.
6. Keep automatic decisions visible in text, table, JSON, and interactive
   review output.

## Proposed public contract

### Input classification

Extend `--input-format` with `auto` and `resource-v1`:

- `auto`: classify the selected artifact root before any server request.
- `raw`: preserve the existing classic raw-export grammar.
- `provisioning`: preserve the existing provisioning-root grammar.
- `resource-v1`: accept only `dashboard.grafana.app/v1` Dashboard resources.
- `resource-v2`: preserve the strict v2 resource grammar.

Keep `raw` as the initial default for backward compatibility. Introduce auto
selection explicitly first; changing the default can be considered only after
one release with telemetry-free operator feedback and migration notes.

Classification order:

1. Use a trusted combined-export index or artifact metadata when present.
2. Recognize resource objects by exact `apiVersion`, exact `kind: Dashboard`,
   stable `metadata.name`, and object-valued `spec`.
3. Recognize provisioning roots from their provider configuration plus the
   resolved dashboards directory.
4. Recognize classic raw documents using the existing raw loader contract.
5. Reject ambiguous, unknown, or mixed-schema roots with per-file evidence.

Mixed classic/resource input stays rejected in the first implementation.
Supporting it safely would require independent lane plans, a complete
cross-lane preflight, deterministic ordering, and truthful partial-run output.

### Server discovery and capability model

Add one shared read-only `GrafanaServerDiscovery` model used by both import and
status:

- instance metadata from `GET /api/health`: database, version, commit, and raw
  non-secret fields;
- dashboard API capabilities: classic, resource-v1, and resource-v2;
- per-capability state: `available`, `unsupported`, `unauthorized`,
  `unavailable`, or `invalid-response`;
- probe endpoint, HTTP status when present, namespace, and concise redacted
  reason;
- probe time and tool version for machine-readable evidence.

Probe resource collections with a bounded read such as `limit=1`. Probe
classic capability through a non-mutating endpoint already required by the
classic import preflight instead of issuing a synthetic write. Cache the
result for the command run so planning and apply cannot disagree.

### Routing matrix

| Classified input | Required capability | Write surface | Fallback |
|---|---|---|---|
| classic raw | classic | `/api/dashboards/db` | none needed |
| provisioning | classic | `/api/dashboards/db` | none needed |
| resource-v1 | resource-v1 | `/apis/dashboard.grafana.app/v1/...` | never to classic |
| resource-v2 | resource-v2 | `/apis/dashboard.grafana.app/v2/...` | never to v1/classic |

Grafana 13 may still receive classic JSON through the classic API. Grafana 12
may use resource APIs only when the live probe proves they exist. A future
Grafana release is supported without a hard-coded major-version branch when
it continues to serve one of the verified contracts.

### Operator overrides

- `--input-format ...` remains the source-schema override.
- Add `--api-mode auto|classic|resource-v1|resource-v2` only if implementation
  reveals a real need to override transport independently. Reject unsafe
  source/API combinations rather than converting silently.
- Add `--explain-routing` to render classification evidence, capability
  evidence, selected endpoint family, and rejected alternatives without
  applying changes.
- Dry-run and interactive review must always display the selected source and
  API lanes.

## Status command integration

Preserve the existing command:

```text
grafana-util status live --profile prod --output-format json
```

It continues to expose `discovery.instance.health.version`, `database`, and
`commit`. Extend its machine-readable `discovery` section with the shared
dashboard capability matrix.

Also add a focused read-only leaf command:

```text
grafana-util status capabilities --profile prod \
  --resource-namespace default --output-format table
```

The focused command reports instance metadata and classic/resource-v1/
resource-v2 availability without running the complete project-domain status
aggregation. JSON/YAML output must use the same `GrafanaServerDiscovery`
contract as `status live` and dashboard import routing.

## Implementation plan

### Phase 1 - shared discovery contract

- [ ] Define the discovery/capability states and redacted output contract.
- [ ] Move the existing `/api/health` reader behind the shared model without
      changing current `status live` fields.
- [ ] Add bounded classic, resource-v1, and resource-v2 probes with explicit
      HTTP error classification and run-local caching.
- [ ] Add focused unit tests for `200`, `401`, `403`, `404`, `405`, `5xx`,
      empty body, wrong kind/version, and malformed JSON.

### Phase 2 - focused status surface

- [ ] Add `status capabilities` text, table, JSON, and YAML rendering.
- [ ] Extend `status live` with the same capability document while preserving
      its existing readiness semantics when optional discovery fails.
- [ ] Update command contracts, bilingual docs, generated man/HTML, and parser
      tests.

### Phase 3 - source auto detection

- [ ] Extract pure classifiers around the existing raw, provisioning, combined
      export, and resource-v2 loaders.
- [ ] Add strict resource-v1 classification and fixtures before implementing
      its write lane.
- [ ] Return deterministic per-file ambiguity and mixed-lane errors.
- [ ] Add `--input-format auto` without changing the current default.

### Phase 4 - negotiated import routing

- [ ] Build one immutable import plan containing source lane, namespace,
      capability evidence, selected endpoint family, and dashboard actions.
- [ ] Route resource-v1 only after its native response and reconciliation
      contract has focused tests and live evidence.
- [ ] Reuse the existing classic and resource-v2 apply engines; do not create
      an independent second decision path for production-client execution.
- [ ] Add `--explain-routing` plus JSON routing evidence.

### Phase 5 - compatibility acceptance

- [ ] Verify classic raw and provisioning import against a Grafana 12 target.
- [ ] Verify classic and resource-v2 import against Grafana 13.1 or newer.
- [ ] Verify resource-v1 only where a live target proves the contract.
- [ ] Verify unsupported-resource, unauthorized-resource, malformed-response,
      mixed-input, no-change, update, and partial-run behavior.
- [ ] Run focused tests, default/TUI Rust suites, clippy for touched targets,
      docs/contracts checks, generated output checks, and `git diff --check`.

## Acceptance criteria

- A user can inspect server version/build and dashboard API capabilities with
  one read-only command.
- Import auto mode chooses a lane from deterministic source evidence, then
  verifies the matching server capability before any write.
- Explicit existing input modes remain backward compatible.
- Resource objects are never silently converted or sent to classic APIs.
- Auth failures, server failures, invalid successful responses, and true
  unsupported endpoints are distinguishable in human and machine output.
- Import dry-run, interactive review, status output, and production apply all
  use the same cached discovery and routing decision.

## Risks and deferred decisions

- Defaulting to `auto` is deferred because it changes behavior for scripts
  that currently rely on implicit `raw`.
- Mixed-lane mutation is deferred until an all-lanes preflight and partial-run
  contract exists.
- Semantic conversion between classic, resource-v1, and resource-v2 is out of
  scope; conversion must remain an explicit command/workflow.
- Capability probing must stay bounded and avoid leaking auth material or raw
  server error bodies.
