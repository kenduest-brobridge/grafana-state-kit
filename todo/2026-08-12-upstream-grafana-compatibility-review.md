# Upstream Grafana Compatibility Review

## Status

- State: complete
- Date: 2026-08-12
- Scope type: upstream investigation and compatibility planning only
- Upstream: official `grafana/grafana` GitHub repository and official release documentation
- Project boundary: Rust-first implementation; Python only where maintained parity or packaging is affected

## Goal

Determine what Grafana has changed upstream since this project's most recent
Grafana 13.1 compatibility work, then identify which changes `grafana-utils`
must accommodate, should support as an improvement, or can intentionally defer.

## Constraints

- Do not change implementation, tests, generated docs, dependencies, or public CLI behavior in this review.
- Use current official upstream sources rather than relying on the July 2026 review as proof of current behavior.
- Inspect `rust/src/` first and avoid `rust/target`.
- Preserve all pre-existing worktree changes.
- Separate released behavior, preview/experimental behavior, deprecation notices, and proposals that have not shipped.
- Exclude beta APIs and unreleased upstream `main` behavior from implementation planning.
- Do not recommend work merely because an upstream feature is new; require a concrete fit with this project's CLI, API, resource, sync, access, status, or export/import responsibilities.

## Questions

1. What is the latest released Grafana line and what changed after the project's last verified Grafana 13.1 baseline?
2. Which official API, App Platform resource, dashboard schema, provisioning/Git Sync, authentication, RBAC, alerting, datasource, or plugin changes overlap this repository?
3. Does the current project already support each relevant change, partially support it, or lack it?
4. What compatibility work is required now, what is a worthwhile optional feature, and what should remain deferred?

## Evidence Plan

- [x] Capture current branch/worktree and existing compatibility trackers.
- [x] Map the current Grafana-facing Rust architecture and public contracts with CodeGraph.
- [x] Inventory literal API versions, endpoints, feature gates, resource kinds, and version assumptions in source/tests/contracts.
- [x] Review official GitHub releases, changelog/migration notes, API definitions, and relevant merged upstream changes.
- [x] Build a finding matrix with upstream evidence, current-project evidence, impact, recommendation, priority, and acceptance boundary.
- [x] Record investigation limitations and distinguish source-level evidence from live Grafana verification.

## Classification

- **Must adapt:** an upstream released change can break or misrepresent an existing supported workflow.
- **Should support:** a released stable capability materially improves an existing project workflow without being required for compatibility.
- **Watch/defer:** experimental, preview, edition-specific, unrelated, or insufficiently stable for this project's support contract.
- **Already covered:** current implementation and regression evidence already satisfy the relevant upstream behavior.

## Findings

### Upstream baseline

- The latest official release on 2026-08-12 is
  [`v13.1.3`](https://github.com/grafana/grafana/releases/tag/v13.1.3),
  published on 2026-08-07. There is no official Grafana 13.2 release yet.
- The project's prior live compatibility evidence stopped at Grafana 13.1.0,
  while its checked-in live-smoke default still uses `grafana/grafana:13.0.1`.
- The official
  [`v13.1.0...v13.1.3` comparison](https://github.com/grafana/grafana/compare/v13.1.0...v13.1.3)
  contains 103 commits. The patch line is mostly fixes and dependency/security
  maintenance, but it also includes Grafana-facing behavior relevant to this
  repository: configurable Git Sync per-resource timeout, idempotent GitHub
  webhooks, and v2 dashboard export metadata cleanup.
- Grafana 13 deprecates the broad legacy `/api` family but does not disable it
  generally yet. The important exception is numeric-ID datasource APIs, which
  are disabled by default. Official migration guidance also states that many
  legacy families do not yet have exact `/apis` replacements.

### Recommendation matrix

| ID | Classification | Priority | Upstream change | Current project state | Recommendation |
|---|---|---:|---|---|---|
| `UGC-001` | Must adapt | P1 | Grafana 13 disables datasource endpoints that address a datasource by numeric ID by default. | Python modify, browse edit/delete, import, and sync apply still write to `/api/datasources/{id}`. Rust is UID-first but automatically falls back to the numeric route after a UID 404. | Make UID update/delete the normal contract in both implementations. Remove automatic numeric fallback from default/auto behavior; retain it only behind an explicit legacy compatibility choice if old Grafana support requires it. |
| `UGC-002` | Must adapt | P1 | Current official release is 13.1.3; 13.1.2 included a security fix and 13.1.3 contains additional dependency maintenance. | Rust and Python live-smoke scripts default to 13.0.1, and CI runs one Rust live target without a Grafana-version matrix. | Declare Grafana 12.0+ support and test an oldest-supported 12.x floor, current 12.x, and current 13.x lane. Run Python datasource live coverage on both the 12.x floor and current 13.x. Test only released Grafana versions. |
| `UGC-004` | Should support | P2 | `playlist.grafana.app/v1` is a stable namespaced API with list/get/create/update/delete support. | The generic App Platform client already supports stable v1 collection, item, pagination, and history reads, but `ResourceKind` does not include playlists. | Add read-only playlist list/get first, plus generic stable-resource history where useful. Treat playlist mutation as a later review-first workflow rather than expanding the generic status command into an unreviewed write surface. |
| `UGC-005` | Should support | P2 | Grafana 13.1.2 strips `grafana.app/folder`, `grafana.app/folderTitle`, and `grafana.app/folderUrl` from UI v2 resource exports. | Native API export intentionally preserves the server resource unchanged and uses `grafana.app/folder` to reconstruct folder paths and round-trip placement. | Keep archival/round-trip export unchanged. Add an explicit portable/share export mode only if users need Grafana-UI-equivalent artifacts; do not silently strip folder ownership from existing exports. |
| `UGC-006` | Already covered | - | Grafana 13.1 made Git Sync root-level sync GA and added README rendering and signed commits. | Workspace discovery accepts root-level v2 resources, `.git` directories or worktree files, `_folder.json`, and `README.md`; direct API mutation of Git Sync-owned dashboards is blocked and operators are routed back to repository/PR ownership. | No server-side Git Sync implementation is required. Keep the current source-ownership boundary; optionally document upstream sync timeout/repository limits in troubleshooting. |
| `UGC-007` | Already covered | - | Stable dashboard/folder App Platform APIs use namespaced v1/v2 resources, continuation tokens, `metadata.name`, optimistic concurrency, and generic history selectors. | The shared client validates collection identity, paginates with repeated-token protection, uses encoded `metadata.name` paths, supports resource history, and preserves schema-v2 `spec` opaquely during export/import. | Continue capability probing rather than version-string gating. Complete the existing dashboard import auto-detection/capability-negotiation plan instead of adding another router. |

### Detailed evidence and acceptance boundaries

#### `UGC-001` - datasource UID migration

Official Grafana 13 upgrade guidance says numeric-ID datasource APIs are
disabled by default and calls must use datasource UIDs. Current project
evidence includes:

- `python/grafana_utils/datasource/workflows.py`: live modify, browse
  edit/delete, import, and related replay paths still construct
  `/api/datasources/{id}`.
- `python/grafana_utils/sync_cli.py`: datasource sync update/delete requires a
  live numeric ID and writes to `/api/datasources/{id}`.
- `rust/src/grafana/api/datasource.rs`: UID update/delete exists, but a 404
  automatically invokes `/api/datasources/{id}` when the caller supplies a
  fallback ID.

Acceptance for the fix:

- all supported Grafana 12.x and 13.x update/delete paths use
  `/api/datasources/uid/{uid}`;
- a missing UID returns the original not-found result without a second numeric
  request;
- any retained numeric compatibility path is explicit and tested only against
  an older supported Grafana lane;
- Python and Rust live smoke both demonstrate add, update, and delete on the
  Grafana 12.x floor and Grafana 13.1.3.

#### `UGC-002` - released-version test matrix

`scripts/test-rust-live-grafana.sh`,
`scripts/test-python-datasource-live-grafana.sh`, and the Python access live
script default to Grafana 13.0.1. `.github/workflows/ci.yml` runs the Rust live
smoke once without setting an image matrix.

Acceptance for the matrix:

- one blocking floor lane uses the latest patch in the oldest supported
  Grafana 12.0 minor line;
- one blocking current-12.x lane detects drift within the supported major;
- one blocking latest-stable lane is pinned to 13.1.3 or a newer verified
  13.1 patch;
- the image tag and observed `/api/health.version` are recorded in evidence;
- release jobs depend on the intended released-version lanes, not an
  unpinned `latest` image.

#### `UGC-004` - playlist read support

This is a small extension because `AppPlatformResourceClient` already owns the
path, pagination, identity, and history mechanics. The first slice should add
`playlists` to read-only `status resource` list/get/describe, response-kind
tests for `PlaylistList` and `Playlist`, bilingual docs, command contracts, and
live read acceptance. Playlist import/sync/delete needs its own safety design.

#### `UGC-005` - portable v2 export

Grafana's 13.1.2 change applies to the UI share/export path, not to the raw
resource GET contract. This project is correct to preserve full live metadata
for round-trip evidence. If portability is added, define two explicit products:

- **round-trip/archival:** retain folder annotation and index-derived folder
  path;
- **portable/share:** strip environment-specific folder annotations and
  server-owned metadata while preserving the entire v2 `spec`.

### No-action upstream items

- Grafana 13.1.1's configurable Git Sync per-resource timeout and idempotent
  GitHub webhook fix are server behavior. `grafana-utils` does not configure
  server repository connections, so only troubleshooting/version guidance is
  relevant.
- Dashboard visual/editor features such as quick filters, section variables,
  panel styles, and revised query editing do not require code changes because
  the resource-v2 lane treats the complete `spec` as opaque data.
- Grafana 13.1.3's BarChart, snapshot-key, and dependency fixes do not change
  this project's API contract, but they reinforce updating the live image.
- Grafana Assistant and PDC feature additions are edition/plugin capabilities,
  not responsibilities of this CLI.

### Suggested execution order

1. Establish Grafana 12.0+ as the compatibility floor and require capability
   negotiation for stable features that are not universal across 12.x/13.x.
2. `UGC-001`: remove default numeric datasource mutation and add regressions
   for both Grafana 12.x and 13.x.
3. `UGC-002`: update the live compatibility matrix and prove the UID fix on the
   oldest supported 12.x, current 12.x, and current 13.x lanes.
4. Complete the existing dashboard import capability-negotiation plan so new
   `/apis` lanes share one discovery contract.
5. `UGC-004` and `UGC-005`: add playlist reads and portable export as bounded,
   capability-gated features that do not raise the minimum server version.

Beta APIs and unreleased upstream-main behavior are intentionally excluded and
must not create implementation tasks. See the stable-only execution plan in
`todo/2026-08-12-stable-grafana-compatibility-plan.md`.

## Verification Evidence

- Repository baseline: `dev...origin/dev [ahead 21]` with extensive
  pre-existing modified files. This planning pass changed only this dated
  tracker, the stable compatibility plan, and `todo/todo.md`.
- Structural inventory: CodeGraph identified the shared
  `AppPlatformResourceClient`, versioned path construction, pagination,
  history helpers, dashboard resource clients, alerting, datasource, access,
  and status boundaries.
- Literal inventory: current endpoints, resource groups, API versions, test
  images, Git Sync fixtures, and legacy mutation routes were checked without
  inspecting `rust/target`.
- Official release evidence:
  - [Grafana v13.1.3 release](https://github.com/grafana/grafana/releases/tag/v13.1.3)
  - [Grafana v13.1.0 to v13.1.3 comparison](https://github.com/grafana/grafana/compare/v13.1.0...v13.1.3)
  - [Grafana 13.1 changelog](https://github.com/grafana/grafana/blob/v13.1.3/CHANGELOG.md)
  - [Grafana 13.1.1 configurable Git Sync timeout](https://github.com/grafana/grafana/commit/8dfd7cd1fd2618f382da6a1b71bb6460ed4a7de2)
  - [Grafana 13.1.2 portable v2 export metadata fix](https://github.com/grafana/grafana/commit/4beb73858401c4ca9980f9b8118fef56a94c53c7)
- Official API/upgrade evidence:
  - [Grafana 13 upgrade guide](https://grafana.com/docs/grafana/latest/upgrade-guide/upgrade-v13.0/)
  - [Grafana 13.1 features](https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v13-1/)
  - [API migration guide](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/apis-migration/)
  - [Available HTTP APIs](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/)
  - [Playlist API](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/playlist/)
  - [Resource history API](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/resource-history/)
- No unit, integration, Docker, or live Grafana tests were run because this was
  an investigation-only pass. Compatibility conclusions are source- and
  documentation-backed; implementation findings still require the acceptance
  runs listed above.
