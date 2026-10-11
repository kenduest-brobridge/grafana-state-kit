# Development priorities and simplification — 2026-10-11

## Scope and constraints

- Start at `aaad74f2` with a clean working tree; preserve unrelated work.
- Rust-first inventory of remaining product work, excess abstraction and stale,
  redundant or implementation-mirroring unit tests; verify old backlog claims.
- Select a bounded simplification that removes concrete duplication while
  preserving supported CLI/API/security and artifact contracts.
- Keep Python compatibility and Grafana 12/13 support. Do not delete tests based
  only on age, count, filename or runtime. No index rebuild, push or deployment.

## Plan

- [x] Verify useful next development work against current implementation.
- [x] Inventory abstraction and test duplication with source evidence.
- [x] Implement an appropriate bounded simplification and focused verification.
- [x] Reconcile the actionable backlog, verification and remaining boundaries.

## Verification baseline

- Previous task at `aaad74f2`: Rust 2,052 passing tests, one ignored macOS
  Keychain smoke; Python 1,213 tests with two skips; fmt, Clippy, documentation
  contracts and five Grafana live compatibility lanes passed.
- Reuse that evidence where unchanged; new checks follow selected behavior
  impact. Test removals require identifying retained coverage of their contract.

## Worth developing next

| Order | Operator need and smallest useful slice | Acceptance and effort |
| --- | --- | --- |
| 0 | Reconcile unregistered Dashboard tests with current behavior: migrate valuable unique assertions into active owners and retire only proven replacements. | Named cases appear in Cargo's test list and run; restore meaningful coverage, not the entire old module tree. The first three focused-review cases are restored in this slice; 51 remain to classify. |
| 1 | Harden reviewed Alert updates: re-read every rule before the first write and compare with the reviewed baseline; on runtime failure report completed, failed and not-attempted identities. | Unchanged baseline proceeds; drift sends no writes; a second-write error reports exact partial progress with nonzero exit. M; preserve narrow allowed fields and approval. |
| 2 | Add focused server capability diagnostics for dashboard Classic/v1/v2 APIs, reusing existing discovery/validation. | Distinguish 401/403, 404, 5xx and malformed success; explicit namespace; bounded read-only probes. M; no general plugin registry or persistent cache. |
| 3 | Add explicit portable native-v2 export while keeping archival export as the default. | Preserve the complete opaque `spec`; strip only verified source/server metadata in the opt-in mode; verify import into the chosen destination. S–M; metadata policy needs official API evidence before implementation. |
| 4 | Add playlist list/get/describe if operators actually use rotation playlists. | Reuse resource collection/item/pagination helpers, strict identity, auth errors and unsupported capability handling. S–M; validate the stable server contract first; no mutation or sync expansion. |

Source evidence:

- `alert/runtime_support.rs::prepare_controlled_alert_live_apply_row` compares
  desired against stored live data; `execute_alert_plan_with_request` accumulates
  results but returns early on a write error. `grafana/api/alert_live.rs` sends the
  prepared PUT without re-reading the rule. These are next safety improvements,
  not failures claimed by the previous live smoke results.
- `status/cli.rs` has staged/live leaves; `status/live_discovery.rs` supplies
  instance health. `dashboard/import/auto.rs` already performs native-v2 probes.
- `dashboard/cli_defs_command_export.rs` has native format/namespace options,
  while `export_scope.rs` writes the archival resource unchanged.
- `resource/catalog.rs` and `resource/runtime.rs` do not include playlists.
- Sync's `grafana/api/sync_live_apply_phase.rs` also loses accumulated progress
  on the first error. Extend partial reporting later; sync drift checking needs
  an artifact baseline contract change. Do not build a cross-domain transaction
  or automatic rollback framework as part of the narrow Alert slice.

All source paths above are relative to `rust/src/`. New product features are
ranked proposals, not implemented in this convergence slice.

## Concrete convergence implemented

| Area | Evidence and change | Preserved boundaries |
| --- | --- | --- |
| Import lookup | Removed two forwarding structs, a six-method trait and two implementations. Only the summary-list method was used generically; unused org/folder mutation methods required dead-code allowances. Direct existing client/request calls share one pure cache-population helper. | Search runs once; positive/negative and folder caches, fallback reads, UID encoding and different request/client action decisions remain intact. |
| Authentication | Dashboard and Access auth contexts duplicated their existing connection builders. Delegate to those builders: six resolver blocks become three, 71 net lines removed. | Profile/CLI/env precedence, prompting, TLS, Basic-auth restrictions and org-scoped/no-org policy stay with their existing owners. |
| Test-only prototype | Entire alternate `root_preflight` implementation was compiled only under `cfg(test)` and had no production caller. Removed it and two self-tests. | Current bundle preflight/runtime tests remain; its one independent Alert assessment test moved unchanged to `alert/sync.rs::tests`. |
| Split-file test copies | Removed 16 exact duplicate active tests: seven render, seven query/filter/column, two Access CLI cases. Deleted the redundant render file and registration. | Identical registered child suites retain CSV escaping, rendering, analysis, columns, invalid selectors and destructive-command prompt behavior. |
| Dormant useful coverage | Moved three unique Classic focused-review request/cache scenarios from the unregistered subtree to the active review test owner. | Original fixtures/assertions retained; Cargo's list and exact executions now prove the cases actually run. |

Removed 18 registered tests, restored three previously dormant cases: a net
15 fewer compiled tests, with stronger meaningful coverage. Rust source shrinks
by 1,456 net lines, including the obsolete test-only prototype. Runtime auth
and lookup scaffolding alone shrink by 210 lines. These are source reductions,
not measured speed improvements. The prototype's passing tests could not
establish production coverage.

Retained coverage map:

- Original dashboard render copies →
  `dashboard/export_focus_report_query_presentation_rendering_rust_tests.rs`
  (all seven identical registered cases).
- Original query copies →
  `dashboard/export_focus_report_query_presentation_contract_filters_rust_tests.rs`
  (two) and `dashboard/export_focus_report_query_presentation_contract_columns_rust_tests.rs`
  (five).
- Access root copies → `access/access_cli_help_examples_rust_tests.rs` and
  `access/access_cli_org_team_rust_tests.rs`.
- Old root prototype → production `sync/bundle_preflight.rs` and its contract
  suite; the independent `assess_alert_sync_specs_reports_plan_only_and_blocked_states`
  assertion remains in the actual Alert assessment owner.

## Next convergence candidates, not changed

An additional confirmed registration gap requires different handling from
duplicate deletion: `dashboard_list_render_rust_tests.rs` has no incoming Rust
module declaration. It includes the browse workflow router and six children;
these eight files contain 54 dormant `#[test]` declarations, with no same-name
definitions outside that subtree. A focused Cargo filter matching zero tests
exposed this gap; zero matching tests is not validation.

Three unique Classic focused-review request/cache cases have moved into
the existing registered `import_interactive_review.rs` tests. They cover
create-only blocked-existing review, update-only skip-missing review, and changed
live summary. Production still invokes the resolver from
`import_interactive_render.rs`; noninteractive and native-v2 tests do not cover
this exact boundary. The remaining 51 cases are not proven obsolete, and their old
bodies are uncompiled/unverified. Inventory them individually before migrating
or deleting them; do not re-register the entire old tree blindly.

1. Share the exact service-account search/name lookup in
   `access/service_account.rs` and `access/pending_delete_service_account.rs`.
   Keep token secrecy, review, confirmation and delete execution domain-owned.
2. Share dashboard search pagination between `dashboard/live.rs` and
   `grafana/api/dashboard.rs`; keep domain response validation. Do not create a
   universal pagination framework for unlike APIs.
3. Further exact test pairs can merge after preserving their assertions:
   - `dashboard/dashboard_cli_inspect_help_rust_tests.rs`:
     `parse_cli_supports_summary_live_output_format_flag` duplicates
     `parse_cli_supports_summary_live_governance_json_output_format`.
   - `dashboard/inspect_export_rust_tests.rs`: output-format flag/dependency
     cases duplicate report tree-table/dependency cases.
   - `sync/cli_apply_review_rust_tests.rs`: the non-plan-lineage and wrong-stage
     rejection tests are exact copies.
   - `sync/cli_apply_review_exec_review_rust_tests.rs`: explicit-audit and
     marks-plan-reviewed tests pass identical metadata but assert only success.
     Prefer one test verifying persisted audit fields, rather than keeping two
     success-only assertions.
4. `Cargo.toml` still declares direct dev `tower` without source/test references.
   Consider removing it in a separate dependency slice; transitive HTTP tower
   usage is unrelated. No build-time improvement was measured.

## Keep these protections and boundaries

- Classic/native-v2 import separation protects schema, namespace, ownership and
  resource-version semantics. `LiveImportBackend` shares real apply orchestration
  and remains useful; it differs from the removed unused lookup abstraction.
- Security, secret redaction, source ownership, dry-run zero-write, auth/scope
  boundaries, CLI rejection, parser/help and API error tests remain.
- Similar user/team browser tests operate on different state types; similarity
  alone is not sufficient to delete them.
- Retired screenshot/legacy CLI tests prove intentional rejection. The ignored
  macOS Keychain test and optional Python `httpx` skips are environment-bound
  acceptance, not obsolete code. They remain explicitly reported.
- No Rust/Python rewrite, generalized workbench/plugin framework, speculative
  async conversion, caching layer or splitting by file length is justified.

## Backlog reconciliation

- Mark the previous deep review complete in `todo/todo.md`.
- The July auto-capability plan still described raw default and future auto
  detection; add current shipped-state reconciliation instead of reopening it.
- The August compatibility plan's SGC-000/001/002 are completed; current lanes
  use 13.2.1. Playlists and portable export remain open, separately scoped.
- Older June maintainability splits are already completed; file-count and
  size-only warnings do not justify redoing them.

## Verification and remaining boundaries

- Focused existing tests: auth/profile connection 15; query presentation 28;
  Access CLI 55; moved Alert assessment one; canonical bundle preflight 10.
- Parent import-cache regressions: two existing cache/folder cases and four
  import guard cases passed. The old interactive filter initially matched zero
  tests; this is the registration gap described above, not a passing check.
- Combined check before restoring the three dormant cases: full Rust 1,992
  library plus 42 integration tests passed, one intentionally ignored;
  fmt/Clippy passed. AI workflow and architecture checks passed (five unchanged
  size-only warnings).
- Restored cases: Cargo's test list contains all three fully qualified names;
  exact execution passed one test per name. Zero-match initial filters were
  corrected and never counted as passing validation. Subsequent changes affect
  only tests/records; reuse the passing runtime/integration evidence above.
- Final fmt/Clippy, AI workflow and architecture checks after case migration
  passed. Architecture retains the same five size-only warnings; no new warning.
- Reviewed convergence scope committed locally by Luna.
- Reuse previous Python and five-lane Grafana evidence where unchanged; do not
  claim newly rerun live acceptance. No production deployment or OS keyring/TUI
  session validation is part of this cleanup.
