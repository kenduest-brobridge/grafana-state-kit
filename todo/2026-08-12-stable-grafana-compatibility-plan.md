# Stable Grafana Compatibility Plan

## Status

- State: in progress (SGC-000 through SGC-002 complete; SGC-003 and SGC-004 remain separate slices)
- Date: 2026-08-12
- Source review: `todo/2026-08-12-upstream-grafana-compatibility-review.md`
- Scope: Grafana 12.x and 13.x using released, stable APIs and officially
  released Grafana images only

## Objective

Maintain `grafana-utils` compatibility from Grafana 12.0 onward, including the
released Grafana 12.x and 13.x lines, while keeping the compatibility contract
explicit, testable, and safe for operators.

## Non-goals

- Do not implement or track beta APIs, including notification
  `v1beta1` mutation support.
- Do not add CI or compatibility claims for unreleased upstream `main` images.
- Do not migrate unrelated legacy APIs without a released stable replacement.
- Do not change existing archival dashboard exports implicitly.
- Do not make a Grafana 13-only API a prerequisite for workflows supported on
  Grafana 12.x.

## Requirements

### `SGC-000` - Grafana 12.x compatibility floor

- [x] Declare Grafana 12.0 as the minimum supported major/minor floor in the
  maintainer compatibility contract and operator documentation.
- [x] Keep workflows shared by Grafana 12.x and 13.x on stable APIs available
  in both lines.
- [x] Use capability discovery for stable features that are not present across
  every supported 12.x and 13.x release.
- [x] Return an explicit unsupported-capability result when a stable resource
  is unavailable; do not treat `401`, `403`, malformed success, or `5xx` as an
  absence signal.
- [x] Preserve the existing Grafana 12 dashboard, datasource, alert, access,
  status, import/export, and sync paths while adding Grafana 13 behavior.
- [x] Document feature availability separately from the minimum server version
  so a 13-only feature does not make the whole CLI 13-only.

Acceptance:

- [x] Core Rust and maintained Python workflows pass against a released
  Grafana 12.x floor image and the current Grafana 13.x image.
- [x] Feature-specific commands explain when a stable API is unavailable on a
  particular supported server.
- [x] No common workflow unconditionally selects a Grafana 13-only route.

### `SGC-001` - Datasource UID-only mutation

- [x] Change Rust datasource update/delete behavior to use
  `/api/datasources/uid/{uid}` without automatic numeric-ID fallback.
- [x] Change maintained Python modify, browse edit/delete, import, and sync
  mutation paths to use datasource UIDs.
- [x] Return the original UID not-found/error result without issuing a second
  numeric-ID request.
- [x] Verify the UID mutation route against both Grafana 12.x and 13.x; retain
  a numeric-ID path only if a supported 12.x server actually requires it, and
  then only behind an explicit compatibility option rather than auto mode.
- [x] Add Rust and Python regression tests covering request paths and the
  no-fallback contract.

Acceptance:

- [x] Rust focused datasource tests pass.
- [x] Python focused datasource and sync tests pass.
- [x] Live add/update/delete succeeds against the Grafana 12.x floor and
  Grafana 13.2.0 using UID routes (see the 2026-09-05 execution evidence).
- [x] Captured requests show no `/api/datasources/{numeric-id}` mutation in the
  default path.

### `SGC-002` - Released-version compatibility matrix

- [x] Define Grafana 12.0 as the compatibility floor in one maintainer-facing
  source of truth.
- [x] Add a blocking floor lane using the latest released patch in the oldest
  supported Grafana 12.0 minor line.
- [x] Add a blocking current-12 lane using the latest reviewed Grafana 12.x
  release.
- [x] Pin the current stable live-smoke lane to the reviewed released image
  `grafana/grafana:13.2.0`.
- [x] Run the maintained Rust live suite in the floor-12, current-12, and
  current-13 lanes.
- [x] Run the Python datasource live suite in at least the floor-12 and
  current-13 lanes.
- [x] Record the container image and `/api/health.version` in test evidence.
- [x] Keep release gating independent of unpinned `latest` images.

Acceptance:

- [x] CI and local scripts select versions explicitly.
- [x] All three released Rust lanes pass.
- [x] Both required Python datasource lanes pass.
- [x] Failure output identifies the exact Grafana version and failing workflow.

### `SGC-003` - Stable playlist read support

- [ ] Extend the shared resource taxonomy with
  `playlist.grafana.app/v1`.
- [ ] Add read-only playlist list, get, and describe behavior to
  `grafana-util status resource`.
- [ ] Reuse the shared App Platform collection/item, namespace, pagination,
  response-identity, and history mechanics.
- [ ] Capability-gate playlist resource access so Grafana 12.x servers without
  the stable resource report it as unavailable without affecting other status
  commands.
- [ ] Add parser, response-kind, pagination, text/JSON output, and error tests.
- [ ] Update bilingual command documentation and command-surface contracts.
- [ ] Keep playlist create/update/delete/import/sync out of this slice.

Acceptance:

- [ ] `PlaylistList` and `Playlist` responses are validated strictly.
- [ ] Empty collections, pagination, invalid identity, `401`, `403`, `404`, and
  `5xx` retain truthful behavior.
- [ ] Read-only live acceptance passes where the stable resource is available,
  and the Grafana 12.x floor produces the documented capability result.
- [ ] Generated docs and contract checks pass.

### `SGC-004` - Explicit portable dashboard v2 export

- [ ] Preserve the current archival/round-trip export behavior and folder
  annotations as the default.
- [ ] Add an explicit portable/share export choice for schema-v2 resources.
- [ ] Keep Grafana 12.x archival/classic export behavior available when schema
  v2 resource export is not supported by the server.
- [ ] Strip only documented environment/server-owned metadata in portable mode,
  including folder annotations removed by Grafana's UI export.
- [ ] Preserve the complete dashboard v2 `spec` without interpreting panel or
  visualization fields.
- [ ] Make output mode visible in CLI help, result summaries, and generated
  documentation.
- [ ] Add golden/fixture coverage proving archival and portable outputs differ
  only in the intended metadata.

Acceptance:

- [ ] Existing archival export fixtures remain unchanged.
- [ ] Portable output omits `grafana.app/folder`,
  `grafana.app/folderTitle`, and `grafana.app/folderUrl`.
- [ ] Portable output remains valid for the documented import/share workflow.
- [ ] Grafana 12.x export remains functional without selecting a schema-v2
  resource route.
- [ ] CLI, generated docs, and command-surface checks pass.

## Dependencies and sequencing

1. Establish `SGC-000` first so all following work preserves the Grafana 12.0+
   compatibility floor.
2. Implement `SGC-001` because Grafana 13 disables numeric datasource mutation
   by default, while proving the same UID workflow still works on Grafana 12.x.
3. Implement `SGC-002` immediately after `SGC-001` so the compatibility fix is
   proven across the released 12.x and 13.x lanes.
4. Complete the existing dashboard import capability-negotiation plan before
   adding another dashboard export routing choice.
5. Implement `SGC-003` and `SGC-004` as separate capability-gated feature
   slices that do not raise the minimum server version.

## Verification plan

- Start with focused Rust/Python tests for each slice.
- Run `cd rust && cargo test --quiet` for Rust-crossing changes.
- Run `cd python && PYTHONPATH=. poetry run python -m unittest -v tests` for
  maintained Python changes.
- Run `make quality-docs-surface`, `make man-check`, and `make html-check` for
  CLI or documentation changes.
- Run the released-version live lanes defined by `SGC-002` before declaring
  Grafana compatibility complete.
- Record actual test counts, image tags, Grafana health versions, and any
  intentionally unverified boundary in this file as work proceeds.
