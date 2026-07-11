# Grafana 13 All Improvements Implementation Plan

> **For agentic workers:** Execute inline with the `executing-plans` skill.
> Every behavior change follows red-green-refactor and each slice is verified
> independently before the broad repository gates run.

## Status

- State: active
- Branch: `codex/grafana13-all-improvements`
- Worktree: `.worktrees/grafana13-all-improvements`
- Baseline: Rust 1,882 tests passed with one ignored test; Poetry reproduced
  one failure and seven errors caused by the missing YAML runtime dependency.

## Goal

Implement every actionable item from the 2026-07-10 project-wide review:
confirmed Python, datasource, dashboard resource, TUI governance, and tracker
defects plus safe Grafana 13 resource API, history, notification, Git Sync, and
secret-metadata integrations.

## Architecture

The implementation extends the shared Rust Grafana client with one App
Platform resource layer. It owns group/version discovery, standardized
collection paths, pagination, single-resource reads, and resource history.
Dashboard export and `status resource` reuse that layer instead of duplicating
version routing.

Compatibility is negotiated. Stable resource APIs are preferred by `auto`
only when discovery proves they are served; legacy routes remain available.
Beta notification and secret resources require `--api-mode v1beta1`. Secret
integration is metadata-only and strips any unexpected `spec.value` before
rendering. Existing alert provisioning mutations remain unchanged.

Grafana 13.1 Git Sync support is local and read-only: recognize repository-root
dashboard CRDs, `_folder.json`, and `README.md`, record provenance, and preserve
the existing block on direct mutation of Git Sync-managed resources.

## Requirements and Constraints

- Preserve classic dashboard import/apply semantics; resource v2 is export,
  inspect, and history only in this slice.
- `dashboard export --resource-format v2` must use
  `dashboard.grafana.app/v2`; `all` must export every discovered supported lane
  without failing because an older Grafana lacks one lane.
- Discovery must treat only HTTP 404 as unsupported. Authentication,
  permission, transport, and invalid-payload failures must remain visible.
- `status resource --api-mode` accepts `auto`, `legacy`, `v1`, `v2`, and
  `v1beta1`; invalid kind/version combinations fail with concrete guidance.
- Folder v1 list/get uses continuation-token pagination.
- Dashboard resource history supports v1/v2 list and historical get while
  classic history remains the default.
- Notification resource integration covers receivers, routing trees, template
  groups, time intervals, and inhibition rules as read-only values.
- Secret integration covers secure-value list/get metadata only. It never
  accepts, stores, logs, exports, or renders a secret value.
- Python package metadata declares PyYAML and the Poetry test target performs
  real unittest discovery.
- TUI registry entries name exact parser and test files; the checker validates
  their existence and exact public command evidence.
- Public CLI/help/docs changes update contracts and regenerate man/HTML output.
- Default Rust builds stay lean; no new default feature is introduced.

## Acceptance Checks

- [x] Poetry installs PyYAML and all 1,230 Python tests pass with two skips.
- [x] Nonnumeric datasource UID 404 makes no numeric-ID fallback request.
- [x] TUI registry includes access user/team browse and exact parser/test paths.
- [ ] Dashboard v2 and `all` export tests pass, including unsupported-lane skip.
- [ ] Shared discovery, folder pagination, and resource history tests pass.
- [ ] Notification v1beta1 resource kinds are available only through explicit
      beta mode and preserve raw resource documents.
- [ ] Secure-value outputs are metadata-only even for a hostile response that
      unexpectedly contains `spec.value`.
- [ ] Git Sync root-level CRDs, `_folder.json`, README provenance, and linked
      worktree `.git` files are covered.
- [ ] English/zh-TW docs, command contracts, man pages, and HTML are regenerated.
- [ ] Rust fmt, Clippy, default/browser builds, Rust/Python suites, architecture,
      output, TUI, docs, generated-doc, and diff checks pass.

## Execution Plan

### Task 1: Repair the Python YAML installation contract

**Files**

- Modify: `python/pyproject.toml`
- Modify mechanically: `python/poetry.lock`
- Modify: `python/grafana_utils/yaml_compat.py`
- Create: `python/tests/test_python_yaml_compat.py`
- Modify: `Makefile`
- Modify: `docs/internal/maintainer-quickstart.md`
- Modify: `docs/internal/maintainer-role-map.md`

- [x] Write a failing test that loads `root:\n  lane: dashboards` and asserts
      `safe_dump` emits `lane: dashboards` rather than JSON braces.
- [x] Run
      `cd python && PYTHONPATH=. poetry run python -m unittest -v tests.test_python_yaml_compat`
      and confirm the current JSON fallback fails.
- [x] Add `PyYAML>=6,<7`, import it directly in `yaml_compat.py`, update the lock,
      install, and rerun the focused test green.
- [x] Add `types-PyYAML` to the development environment and verify MyPy accepts
      both the adapter and its test.
- [x] Change `poetry-test` and maintainer commands to
      `python -m unittest discover -s tests -v`.
- [x] Run `make poetry-test` and record 1,227 passing tests with two skips.

### Task 1B: Restore the Python formatter quality gate

**Files**

- Format mechanically: `python/grafana_utils/**/*.py`
- Format mechanically: `python/tests/**/*.py`

- [x] Record the pre-existing Black 25.11.0 baseline and confirm Task 1 files
      pass in isolation.
- [x] Run the locked Black formatter over `grafana_utils` and `tests` as a
      separate mechanical change with no behavior edits.
- [x] Rerun `make poetry-test` and the project Black check. The full quality
      target now reaches Ruff/MyPy and exposes a separate pre-existing lint and
      type backlog tracked in Task 1C.
- [x] Commit the formatter-only change separately from functional work.

### Task 1C: Restore the Python Ruff and MyPy quality gates

**Files**

- Modify/test: the 17 Python modules reported by MyPy
- Modify/test: the Python modules/tests reported by Ruff

- [x] Capture and group the current 55 Ruff and 56 MyPy errors by root cause.
- [x] Apply Ruff safe fixes, review every changed import, and rerun Ruff.
- [x] Fix remaining lint findings without behavior changes.
- [x] Fix MyPy findings by correcting real API/name defects first, then adding
      precise annotations and narrow casts where runtime validation exists.
- [x] Run `make poetry-test` and `make poetry-quality-python` green.
- [x] Commit lint/type repairs separately from formatter and Grafana features.

### Task 2: Stop invalid datasource numeric-ID fallback

**Files**

- Modify/test: `rust/src/commands/resource/runtime.rs`

- [x] Add a request-server regression test for `datasources/prom-main`: UID GET
      returns 404, the command returns that error, and only one request is made.
- [x] Run the focused test and confirm it loses the original 404 after the
      current fallback attempt.
- [x] Supply the legacy closure in auto mode only when
      `selector.identity.parse::<u64>().is_ok()`; explicit legacy mode rejects a
      nonnumeric selector before transport.
- [x] Rerun the focused resource runtime tests green.

### Task 3: Make TUI registry coverage exact

**Files**

- Modify: `scripts/contracts/tui-registry.json`
- Modify: `scripts/tui_inventory_report.py`
- Modify/test: `scripts/test_tui_inventory_report.py`

- [x] Add failing tests that require `parser_paths` and `test_paths`, reject a
      missing path, and assert both access browse routes appear.
- [x] Run the script tests and confirm red.
- [x] Add `access user browse` and `access team browse`, mark consolidated
      `access browse` read-only, and replace `has_tests` with concrete paths.
- [x] Validate each path exists and each command token sequence occurs in at
      least one parser path; report `missing-parser-evidence` or
      `missing-test-path` otherwise.
- [x] Add 60x10 terminal tests for access browse detail scrolling and modal
      bounds using the existing `TestBackend` pattern.
- [x] Run registry, checker, and focused Rust TUI tests green.

### Task 4: Add the shared App Platform resource client

**Files**

- Create/test: `rust/src/grafana/api/app_platform.rs`
- Modify: `rust/src/grafana/api/mod.rs`
- Modify: `rust/src/grafana/api/client.rs`

- [x] Write failing pure/request tests for discovery, collection paths,
      404-only unsupported handling, continue pagination, get, history
      selectors, and historical `resourceVersion` reads.
- [x] Introduce these core types and methods:

```rust
pub(crate) enum AppPlatformApiVersion { V1, V2, V1Beta1 }
pub(crate) struct AppPlatformResource {
    pub(crate) group: &'static str,
    pub(crate) version: AppPlatformApiVersion,
    pub(crate) plural: &'static str,
}

impl AppPlatformResourceClient<'_> {
    pub(crate) fn is_served(&self, resource: AppPlatformResource) -> Result<bool>;
    pub(crate) fn list_all(&self, resource: AppPlatformResource, namespace: &str) -> Result<Vec<Map<String, Value>>>;
    pub(crate) fn get(&self, resource: AppPlatformResource, namespace: &str, name: &str) -> Result<Value>;
    pub(crate) fn list_history(&self, resource: AppPlatformResource, namespace: &str, name: &str, limit: usize) -> Result<Vec<Map<String, Value>>>;
    pub(crate) fn get_at_resource_version(&self, resource: AppPlatformResource, namespace: &str, name: &str, resource_version: &str) -> Result<Value>;
}
```

- [x] Rerun the focused API tests green.

### Task 5: Enable dashboard v2 export and resource history

**Files**

- Modify/test: `rust/src/grafana/api/dashboard.rs`
- Modify/test: `rust/src/commands/dashboard/export_resource.rs`
- Modify/test: `rust/src/commands/dashboard/export_scope.rs`
- Modify/test: `rust/src/commands/dashboard/export_focus_report_path_top_rust_tests.rs`
- Modify: `rust/src/commands/dashboard/models.rs`
- Modify: `rust/src/commands/dashboard/export_root_bundle.rs`
- Modify/test: `rust/src/commands/dashboard/cli_defs_command_history.rs`
- Modify/test: `rust/src/commands/dashboard/history_live.rs`

- [x] Add failing tests for v2 lane metadata/path, v2 GET, `all` with only v1
      served, and resource history pagination/version retrieval.
- [x] Make v2 map to `dashboard.grafana.app/v2` and `resource-v2/`.
- [x] Probe lane group/version discovery before export. Explicit unavailable
      versions error; `all` retains every served lane and skips only 404 lanes.
- [x] Add `--history-source classic|v1|v2` to list/export/get-compatible history
      commands with `classic` as the default.
- [x] Preserve v2 resource objects without converting `spec.elements` to
      classic dashboard JSON.
- [x] Run focused dashboard API/export/history tests green.

### Task 6: Expand the read-only resource surface for Grafana 13

**Files**

- Modify/test: `rust/src/commands/resource/cli_defs.rs`
- Modify/test: `rust/src/commands/resource/catalog.rs`
- Modify/test: `rust/src/commands/resource/runtime.rs`
- Modify/test: `rust/src/commands/resource/render.rs`
- Modify: `rust/src/commands/resource/resource_help.rs`

- [x] Add failing parser/catalog/runtime tests for API modes `v1`, `v2`, and
      `v1beta1`; folder v1 auto preference; and invalid mode guidance.
- [x] Add resource kinds `receivers`, `routing-trees`, `template-groups`,
      `time-intervals`, `inhibition-rules`, and `secure-values`.
- [x] Implement descriptor-driven routing through the App Platform client:
      dashboard auto order v2, v1, legacy; folder auto order v1, legacy; beta
      resources require explicit v1beta1.
- [x] Teach table rendering to read `metadata.name`, `spec.title`,
      `spec.description`, and resource-specific names while JSON/YAML preserve
      the full resource documents.
- [x] Run parser/catalog/runtime/render tests green.

### Task 7: Add notification and secret metadata safety contracts

**Files**

- Modify/test: `rust/src/grafana/api/app_platform.rs`
- Modify/test: `rust/src/commands/resource/runtime.rs`
- Add fixtures under: `tests/fixtures/grafana13/`

- [x] Add failing path/round-trip tests for receivers, routing trees, template
      groups, time intervals, and inhibition rules under
      `notifications.alerting.grafana.app/v1beta1`.
- [x] Preserve the returned notification objects exactly and leave existing
      `/api/v1/provisioning` mutations untouched.
- [x] Add a hostile secure-value fixture containing `spec.value` and assert no
      rendered/list/get document contains the value.
- [x] Strip `spec.value` defensively from secure-value items and reject secret
      export/mutation additions in this surface.
- [x] Run focused resource and redaction tests green.

### Task 8: Recognize Grafana 13.1 Git Sync repositories

**Files**

- Modify/test: `rust/src/commands/dashboard/files/source.rs`
- Modify/test: `rust/src/commands/sync/workspace_discovery.rs`
- Modify: `rust/src/commands/sync/workspace_discovery_rules.rs`
- Modify/test: `rust/src/commands/sync/discovery_model.rs`

- [x] Add failing tests for a repository whose `.git` is a file, root-level
      dashboard CRDs, nested `_folder.json`, and `README.md` provenance.
- [x] Recognize `.git` files or directories and a resource root containing
      dashboard CRDs; do not require `dashboards/git-sync/{raw,provisioning}`.
- [x] Add read-only discovery inputs for Git Sync root, folder metadata, and
      README files; retain `git-sync-managed` ownership and mutation blockers.
- [x] Run focused dashboard source and sync discovery tests green.

### Task 9: Update public contracts and documentation

**Files**

- Modify: `scripts/contracts/command-surface.json`
- Modify: `docs/commands/{en,zh-TW}/dashboard-export.md`
- Modify: `docs/commands/{en,zh-TW}/dashboard-history.md`
- Modify: `docs/commands/{en,zh-TW}/resource.md`
- Modify: `docs/user-guide/{en,zh-TW}/`
- Modify: `docs/DEVELOPER.md`
- Modify: `docs/todo/todo.md`
- Regenerate: `docs/man/*.1`, `docs/html/`

- [x] Update source docs with stable/beta maturity, discovery behavior,
      namespace rules, legacy compatibility, and secret metadata limitations.
- [x] Refresh the stale dashboard resource tracker to mark v1 complete and v2
      implemented by this slice.
- [x] Update command contracts and run `make quality-docs-surface`.
- [x] Regenerate `make man` and `make html`; run both drift checks.

### Task 10: Broad verification and grouped closeout

- [ ] Run `cargo fmt --manifest-path rust/Cargo.toml --all --check`.
- [ ] Run `cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings`.
- [ ] Run `cargo test --manifest-path rust/Cargo.toml --quiet`.
- [ ] Run `cargo check --manifest-path rust/Cargo.toml --features browser`.
- [ ] Run `make poetry-test`, `make quality-architecture`,
      `make quality-output-contracts`, `make quality-ai-workflow`,
      `make quality-docs-surface`, `make man-check`, and `make html-check`.
- [ ] Run `python3 -m unittest -v scripts.test_tui_inventory_report` and
      `python3 scripts/tui_inventory_report.py --registry-check --json`.
- [ ] Run `git diff --check` and confirm no generated or test side effects.
- [ ] Commit by concern using the repository hook format; do not mix unrelated
      implementation families.

## Official Contracts

- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/apis/>
- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/dashboard/>
- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/resource-history/>
- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/folder/>
- <https://grafana.com/docs/grafana/latest/upgrade-guide/upgrade-v13.0/>
- <https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/secrets_management/>
- <https://grafana.com/docs/grafana/latest/whatsnew/whats-new-in-v13-1/>

## Evidence Log

- 2026-07-10: User approved PyYAML as a runtime dependency and authorized all
  actionable review findings.
- 2026-07-10: Isolated branch/worktree created after adding `.worktrees/` to
  `.gitignore`.
- 2026-07-10: Rust baseline passed 1,882 tests with one ignored test.
- 2026-07-10: After `poetry install`, Poetry ran 1,226 tests and reproduced
  exactly one failure and seven errors from the JSON-only YAML fallback.
- 2026-07-10: Task 1 red test observed JSON output instead of YAML, then passed
  after declaring PyYAML. `make poetry-test` discovered and passed 1,227 tests
  with two skips.
- 2026-07-10: The broader Python quality target found a pre-existing Black
  25.11.0 drift across 96 files. The same Black version is present in the
  baseline lock, while the two Task 1 files pass Black and Ruff in isolation;
  a formatter-only Task 1B was added instead of mixing 96 mechanical rewrites
  into the YAML fix.
- 2026-07-10: Formatting with the quality script's exact Python 3.9 target made
  all 173 files Black-clean and the 1,227-test suite remained green. The next
  quality stages exposed an independent baseline of 55 Ruff and 56 MyPy errors;
  these are isolated as Task 1C instead of being hidden by the formatter fix.
- 2026-07-10: Task 1C restored Ruff, MyPy, and Black gates and fixed three
  runtime defects found by static analysis: sync partial status referenced a
  misspelled constant, single-image full-page screenshots referenced an
  unloaded Pillow symbol, and Python resource get called a nonexistent
  dashboard client method. Each received a failing regression test before its
  fix. `make poetry-quality-python` passed all stages with 1,230 tests and two
  skips.
- 2026-07-10: Task 2 red test showed a nonnumeric datasource UID 404 was
  replaced by an unclassified error after the legacy request attempt. The
  legacy closure is now available only for numeric identities; all nine
  resource runtime tests pass and the original UID 404 is preserved.
- 2026-07-10: Task 3 upgraded the TUI registry to schema 2 with concrete parser
  evidence and test paths, added access user/team browse, corrected consolidated
  access browse to read-only, and removed the nonexistent `dashboard topology`
  public path. Four 60x10 access browser render/modal tests and two detail
  cursor clamping tests pass with the `tui` feature; all 14 registry tests and
  the live registry check are green.
- 2026-07-10: Task 4 added a shared App Platform client for v1, v2, and
  v1beta1 resources. Request tests cover 404-only discovery, authorization
  error preservation, encoded continue pagination, current gets, stable
  history selectors, and historical `resourceVersion` reads. All 25 shared API
  tests pass.
- 2026-07-12: Task 5 enabled verified `dashboard.grafana.app/v2` GET/export,
  added per-lane discovery with explicit-version errors and 404-only skipping
  for `all`, and writes complete `resource-v2` indexes/metadata without
  rewriting `spec.elements`. Dashboard history list/export now accepts
  `--history-source classic|v1|v2` plus an App Platform namespace and follows
  resource-history pagination. Focused history (57), resource export (2), and
  dashboard API client (6) tests pass.
- 2026-07-12: Tasks 6-7 expanded `status resource` to explicit v1, v2, and
  v1beta1 modes; dashboard auto resolves v2 to v1 to legacy, folders resolve
  v1 to legacy, and public-preview notification/secret resources require an
  explicit beta mode. Eleven kinds are cataloged, nested metadata/spec render
  correctly, notification fixtures round-trip unchanged, and hostile
  `spec.value` input is stripped before any renderer sees it. Resource (26),
  parser surface (24), and App Platform (3) focused tests pass.
- 2026-07-12: Task 8 recognizes Grafana 13.1 Git Sync repositories with either
  a `.git` directory or worktree `.git` file and root/nested dashboard CRDs.
  Root resource trees resolve as Git Sync raw sources, `_folder.json` is kept
  as provenance instead of loaded as a dashboard, and discovery records the
  repo root, first folder metadata file, and README. Source-loader (6),
  workspace-discovery (9), repo-layout (3), and dashboard-file (6) tests pass.
- 2026-07-12: Task 9 updated the public command contract, bilingual dashboard
  export/history/resource references, status-workspace handbook, developer
  boundaries, active resource tracker, and AI trace. `make
  quality-docs-surface`, `make man-check`, `make html-check`, and `make
  quality-ai-workflow` all pass after regenerating 88 manpages and 286 HTML
  pages.
