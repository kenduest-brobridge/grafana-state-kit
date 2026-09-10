# Grafana stable upstream review — 2026-09-10

## Scope and plan

- Review official Grafana GitHub released versions against local `94b0572f`.
- User clarification: Grafana 13 is the primary focus; 12.x is the regression floor.
- Preserve Grafana 12.x support; exclude beta APIs and unreleased main changes.
- Analysis only: change task records, not implementation, dependency pins or CI.
- [x] Verify stable releases and relevant upstream fixes.
- [x] Compare dashboard, datasource and compatibility-smoke code with upstream.
- [x] Record prioritized follow-ups with sources and verification boundaries.

## Acceptance

- Distinguish confirmed local gaps from upstream-only bugs and proposed work.
- Reuse the existing stable compatibility backlog instead of duplicating it.
- Do not claim runtime compatibility without executing the relevant live lane.

## Findings and proposed follow-ups

### 1. Prioritize Grafana 13.2.1 verification and test-image refresh

- GitHub releases API reports non-prerelease `v13.2.1`, published 2026-09-02;
  the release includes two security fixes and three bug fixes. Its publication
  predates the September 5 review, so this is newly identified here, not a
  claim that every change happened after the last review.
- `Makefile:24`, `.github/workflows/ci.yml:57,79`, and both smoke script defaults
  still select 13.2.0. The newest patch has no local acceptance evidence yet.
- Proposed work: run Rust full and Python datasource lanes on 13.2.1 first,
  then update maintained pins/docs/contracts consistently. Retain 12.x floor
  coverage. Security fixes are Grafana server changes, not evidence that this
  Rust client contains those vulnerabilities.
- Source: https://github.com/grafana/grafana/releases/tag/v13.2.1

### 2. Recheck org deletion before changing the pinned exception

- Issue #127386 remains open. Both proposed fixes #127404 and #128259 are open
  and unmerged at inspection time; do not present them as released fixes.
- `scripts/test-rust-live-grafana.sh:728` and the Python script at line 206
  intentionally accept only the known 13.2.0 HTTP 500 JSON error. A 13.2.1 run
  with that error would fail rather than silently skip it.
- Proposed work: exercise 13.2.1 explicitly; if fixed, restore all deletion
  assertions; if reproduced, add only the independently verified version and
  regression fixtures. Never replace the guard with a broad `13.*` exception.
- Sources: https://github.com/grafana/grafana/issues/127386
  and https://github.com/grafana/grafana/pull/128259

### 3. Add variable-reference compatibility fixtures, not a speculative UI patch

- Released PR #131819 resolves datasource placeholders for `adhoc` and
  `groupby` variables in Classic/v1 UI import. Its description explicitly
  states backend `POST /api/dashboards/import` is unaffected and v2 UI already
  handles the three variable types. The changed files are frontend inputs
  implementation and tests, not a new REST contract.
- Local import auto classification recognizes `__inputs`/`__requires` in
  `rust/src/commands/dashboard/import/auto.rs:135`; export includes templated
  datasource artifacts. These flows deserve fixtures for query/adhoc/groupby,
  but this review has not reproduced a CLI interpolation defect.
- Proposed acceptance: export/import/readback keeps all three datasource
  references correct for same-instance and explicitly mapped cross-instance
  cases. Do not promise automatic mapping for archival v2 imports.
- Source: https://github.com/grafana/grafana/pull/131819

### 4. Strengthen native schema-v2 round-trip coverage

- Released 13.2.0 includes four-level nesting and nested tabs (#129174).
  The 13.2.1 tagged dashboard API directory contains stable `v1` and `v2`.
- Local resource export already uses stable v1/v2 routes with 404-only optional
  discovery (`export_resource.rs:30-90`). Native v2 import preserves the whole
  spec (`import/resource_v2.rs:299-318`); a mandatory Classic conversion would
  undermine this preservation. No missing v2 implementation was found here.
- Proposed acceptance: nested tabs/rows, conditional layouts, variables and
  unknown spec fields survive export/import/readback without flattening.
- Portable/share export remains the existing SGC-004 backlog item; preserve
  archival output by default and avoid introducing another required flag for
  ordinary automatic imports.
- Sources: https://github.com/grafana/grafana/releases/tag/v13.2.0
  and https://github.com/grafana/grafana/tree/v13.2.1/apps/dashboard/pkg/apis/dashboard

### 5. Target 13.2 permission/error coverage; defer optional feature expansion

- 13.2.0 changes contact-point provenance mismatch from 500 to 403 (#127699)
  and changes legacy folder access checks (#125642). Add negative acceptance
  cases for permission-denied behavior rather than assuming endpoint absence
  or introducing a retry/bypass. No local regression is established here.
- Stable playlist v1 is present in the 13.2.1 tagged source, while local
  `commands/resource/catalog.rs:29-43` does not include playlists. Reuse
  SGC-003 for optional read-only support after compatibility verification.
- Notifications v1beta1 and other beta expansion remain excluded. Plugin
  packaging, panel editor sizing, and scripted-dashboard UI changes are not
  grounds to rewrite this CLI or restore the retired screenshot feature.
- Sources: https://github.com/grafana/grafana/releases/tag/v13.2.0
  and https://github.com/grafana/grafana/tree/v13.2.1/apps/playlist/pkg/apis/playlist

## Verification and boundary

- Read official release metadata, tagged API directories, PR descriptions and
  patches, and current issue/PR state through GitHub API; compared current code.
- CodeGraph context was insufficient for this cross-cutting review; current
  source and literal endpoint searches supplied the missing evidence.
- No implementation, dependency, CI pin, or generated-doc changes. Only this
  record and the todo index were changed. No commit or push requested.
- No runtime tests or live Grafana writes were performed. Existing 13.2.0
  evidence is not relabeled as 13.2.1 acceptance. Proposed tasks remain unimplemented.

## Authorized execution — test, review, fix, commit

- State: complete. User authorized implementation after the read-only review.
- Preserve the review above as the pre-execution assessment; this section
  records new evidence and supersedes its analysis-only boundary for this phase.
- [x] Run existing Rust/Python baseline and reproduce 13.2.1 behavior.
- [x] Update 13.x test pins and only evidence-backed org-delete handling.
- [x] Add variable-reference and nested native-v2 regression coverage.
- [x] Verify permission/provenance errors remain fatal without bypass/fallback.
- [x] Run full quality/docs and live 13.2.1 plus retained 12.x lanes.
- [x] Independently review, fix confirmed issues, and commit scoped changes.
- Do not implement beta, playlist, portable-export, or unrelated refactoring.

### Changes and evidence

- The unmodified Rust and Python live scripts both failed on 13.2.1 org
  deletion with HTTP 500/curl 22. A disposable diagnostic run printed exactly
  `{"message":"Failed to delete organization"}`. Regression tests failed for
  the new patch and stale pins before the script/Make/CI changes, then passed.
- The exception now enumerates only 13.2.0 and 13.2.1, preserving the existing
  exact status/JSON checks. Added rejection cases for 13.2.1 403/503/multiple
  JSON responses and the unverified 13.2.2 patch. No production error is hidden.
- Five new Rust test methods cover Classic query/adhoc/groupby references,
  shared prompt input placeholders and CLI prompt rejection, native-v2
  Tabs/Rows/Tabs/Grid layouts with conditions/variables/unknown fields, and
  contact-point/folder 403 propagation without retry or provenance bypass.
  These exposed no production-code defect; no Rust production behavior changed.
- Scope clarification: the CLI intentionally rejects prompt artifacts requiring
  Grafana UI datasource selection. No mapped prompt importer was added.
- Native-v2 fixture shapes were checked against the tagged 13.2.1 schema:
  https://github.com/grafana/grafana/blob/v13.2.1/packages/grafana-schema/src/schema/dashboard/v2/types.spec.gen.ts
  Its preservation assertion is a client export/discovery/payload-normalization
  test, not live server validation or rendered-dashboard acceptance.
- `make quality-rust`: 1,962 library tests plus 38 integration tests passed,
  one existing ignored test; Rustfmt and Clippy passed.
- `make quality-rust-feature-matrix-full`: default and no-default Cargo checks
  passed. Maintained Rust builds used the existing project-specific `/tmp` target.
- `cd python && poetry run bash ../scripts/check-python-quality.sh`: 1,213
  tests run, two optional-httpx cases skipped; Ruff, mypy (109 sources), Black
  (172 files) and bytecode checks passed.
- `make test-grafana-compatibility-matrix`: all five local Docker lanes passed
  with matching health versions: Rust 12.0.10/12.4.9/13.2.1 and Python datasource
  12.0.10/13.2.1. The exact known 13.2.1 org-delete failure skips only dependent
  fixtures. Test containers were removed; unrelated containers were preserved.
- `make man html man-check html-check quality-ai-workflow` and
  `make quality-docs-surface`: regenerated docs and checks passed. One old entry
  from each AI trace was archived using the repo tool to meet its size limits;
  archive links remain relative. No history content was discarded.
- No remote CI run, production deployment, browser rendering, merge, or push.
- Independent final review found no remaining important issues in this diff;
  the reviewer also ran shell guard tests, syntax checks, and diff checks.
  Main reran full Rust quality after the final native-v2 fixture expansion.
