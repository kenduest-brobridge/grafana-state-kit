# ai-changes.md

Current AI change log only.

- Older detailed history moved to [`archive/ai-changes-archive-2026-03-24.md`](docs/internal/archive/ai-changes-archive-2026-03-24.md).
- Detailed 2026-03-27 entries moved to [`archive/ai-changes-archive-2026-03-27.md`](docs/internal/archive/ai-changes-archive-2026-03-27.md).
- Detailed 2026-03-28 task notes were condensed into [`archive/ai-changes-archive-2026-03-28.md`](docs/internal/archive/ai-changes-archive-2026-03-28.md).
- Detailed 2026-03-29 through 2026-03-31 entries moved to [`archive/ai-changes-archive-2026-03-31.md`](docs/internal/archive/ai-changes-archive-2026-03-31.md).
- Detailed 2026-04-01 through 2026-04-12 entries moved to [`archive/ai-changes-archive-2026-04-12.md`](docs/internal/archive/ai-changes-archive-2026-04-12.md).
- Keep this file limited to the latest active architecture and maintenance changes.
- Older entries moved to [`ai-changes-archive-2026-04-13.md`](docs/internal/archive/ai-changes-archive-2026-04-13.md).
- Older entries moved to [`ai-changes-archive-2026-04-14.md`](docs/internal/archive/ai-changes-archive-2026-04-14.md).
- Older entries moved to [`ai-changes-archive-2026-04-15.md`](docs/internal/archive/ai-changes-archive-2026-04-15.md).
- Older entries moved to [`ai-changes-archive-2026-04-16.md`](docs/internal/archive/ai-changes-archive-2026-04-16.md).
- Older entries moved to [`ai-changes-archive-2026-04-17.md`](docs/internal/archive/ai-changes-archive-2026-04-17.md).
- Older entries moved to [`ai-changes-archive-2026-04-18.md`](docs/internal/archive/ai-changes-archive-2026-04-18.md).
- Older entries moved to [`ai-changes-archive-2026-04-19.md`](docs/internal/archive/ai-changes-archive-2026-04-19.md).
- Older entries moved to [`ai-changes-archive-2026-04-20.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-20.md).
- Older entries moved to [`ai-changes-archive-2026-04-26.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-26.md).
- Older entries moved to [`ai-changes-archive-2026-04-27.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-27.md).
- Older entries moved to [`ai-changes-archive-2026-04-28.md`](/Users/kendlee/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-04-28.md).
- Older entries moved to [`ai-changes-archive-2026-05-02.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-02.md).
- Older entries moved to [`ai-changes-archive-2026-05-14.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-14.md).
- Older entries moved to [`ai-changes-archive-2026-05-16.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-16.md).
- Older entries moved to [`ai-changes-archive-2026-05-25.md`](/Users/ken/work/grafana-utils/docs/internal/archive/ai-changes-archive-2026-05-25.md).

## 2026-05-25 - TUI completion audit
- Summary: Replaced the open-ended TUI follow-up section with a completion audit that maps current evidence to the finished shared review/detail/diff projection work and records why domain-specific input loops remain local.
- Tests: cargo test --quiet; cargo test --quiet user_browse; cargo test --quiet team_browse; cargo test --quiet datasource_browse; cargo test --quiet status_tui; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow; git diff --check
- Impact: The roadmap now has an evidence-backed completion audit instead of stale continue-follow-up items. Public CLI paths, help text, generated docs, Rust runtime behavior, and command contracts are unchanged.
- Rollback/Risk: Low. Documentation-only audit update based on current inventory/search/test evidence.
- Follow-up: Treat future TUI work as new scoped feature work unless a fresh inventory or user report identifies a concrete regression or duplication.

## 2026-05-25 - Shared review narrative and impact projection
- Summary: Moved access plan narrative and impact row projection into the shared review contract so mutation review surfaces can reuse action/status/changed-field guidance text.
- Tests: cargo test --quiet review_mutation_action_narrative_and_impact_lines_project_action_guidance; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Narrative and Why this matters rows while review_contract now owns the generic action narrative and changed-field impact projection. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent projection code into review_contract, removes the now-unused access review action alias, and focused access/review-contract tests cover the old output.
- Follow-up: Continue auditing remaining compatible local artifact browser review/detail projections before declaring the broader TUI design work complete.

## 2026-05-25 - Shared review context projection
- Summary: Moved access plan warning and blocker context row projection into the shared review contract so mutation review surfaces can reuse blocked reasons, safe warning changed fields, and blocked target flag evidence.
- Tests: cargo test --quiet review_mutation_action_context_lines_project_warning_and_blocker_evidence; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Blocked context, Warning context, and Blocked evidence rows while review_contract now owns the generic warning/blocker context projection for mutation actions. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent context projection code into review_contract and focused access/review-contract tests cover the old output.
- Follow-up: Continue auditing the remaining access-specific narrative and impact rows before deciding whether they belong in shared review projections.

## 2026-05-25 - Shared review target evidence projection
- Summary: Moved access plan live-target evidence row projection into the shared review contract so mutation review surfaces can reuse known target field rows.
- Tests: cargo test --quiet review_mutation_action_target_evidence_lines_project_known_live_target_fields; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Live target: key=value rows while review_contract now owns the known target field projection for generic mutation actions. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent target projection code into review_contract and focused access/review-contract tests cover the old output.
- Follow-up: Continue moving compatible warning/blocker context rows out of per-surface TUI renderers and into shared review projections.

## 2026-05-25 - Shared review change-detail projection
- Summary: Moved access plan action change-detail row projection into the shared review contract so mutation review surfaces can reuse safe Change: field bundle/live rows.
- Tests: cargo test --quiet review_mutation_action_change_detail_lines_hide_secret_like_fields; cargo test --quiet access_plan_interactive_browser; cargo test --quiet access_plan_interactive_shared_diff_preview_hides_secret_like_fields; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same Change: field bundle/live rows while the reusable review contract now owns safe changed-field filtering and compact value formatting for generic mutation action changes. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent projection code into review_contract and focused access/review-contract tests cover safe field filtering plus existing TUI output.
- Follow-up: Continue moving compatible mutation review target/context rows out of per-surface TUI renderers and into shared review projections.

## 2026-05-25 - Shared review diff preview projection
- Summary: Moved access plan action diff-preview line projection into the shared review contract so mutation review surfaces can reuse safe live/desired preview rows.
- Tests: cargo test --quiet review_mutation_action_diff_preview_lines_hide_secret_like_fields; cargo test --quiet access_plan_interactive_shared_diff_preview_hides_secret_like_fields; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan TUI keeps the same shared diff preview output and secret-like field filtering while the reusable review contract now owns the generic action changes to ReviewDiffModel preview projection. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves equivalent projection code into review_contract and focused access/review-contract tests cover safe field filtering plus existing TUI output.
- Follow-up: Continue moving compatible mutation review detail rows out of per-surface TUI renderers and into shared review projections.

## 2026-05-25 - Shared review next-check projection
- Summary: Moved access plan action next-check line projection into the shared review contract so TUI review surfaces can reuse hint/default follow-up guidance.
- Tests: cargo test --quiet review_mutation_action_next_check_lines_project_hints_and_default_guidance; cargo test --quiet access_plan_interactive_browser; cargo test --quiet review_contract; cargo test --quiet access (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Access plan interactive reviews now consume shared review-contract next-check lines while preserving existing hint, blocker, warning, create/update/delete, and no-op guidance strings. Public CLI paths, help text, generated docs, and command contracts are unchanged.
- Rollback/Risk: Low. This moves existing string projection into the shared review contract and focused access/review-contract tests cover the old output.
- Follow-up: Continue migrating compatible review panes onto shared review-contract detail and diff projection helpers.

## 2026-05-25 - Shared datasource artifact detail projection
- Summary: Added a shared datasource browser detail projection for local artifact records and routed datasource local inspect plus snapshot datasource review rows through it before appending review evidence.
- Tests: cargo test --quiet datasource_browser_detail_lines_from_details_formats_local_artifact_identity; cargo test --quiet tail_inspect; cargo test --quiet snapshot_review_browser; cargo test --quiet snapshot; cargo test --quiet datasource (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Datasource local list and snapshot datasource review browser rows now share identity fact shaping for Name, UID, Type, Org, URL, Access, and Default while preserving existing shared review evidence projection and public CLI/doc surfaces.
- Rollback/Risk: Low. The change centralizes equivalent datasource fact strings and focused datasource/snapshot tests cover both local artifact browser paths.
- Follow-up: Continue migrating compatible local artifact browser detail sections onto shared projections where output contracts are already aligned.

## 2026-05-25 - Cleared dashboard browse helper drift
- Summary: Renamed the dashboard browse fact-line builder to describe its dashboard-specific filtering and live-details badge behavior instead of looking like a generic build_info_lines helper.
- Tests: cargo test --quiet dashboard_browse_detail_does_not_keep_generic_build_info_wrapper; cargo test --quiet dashboard_browse; cargo test --quiet dashboard (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: The manual TUI inventory report now has zero helper-drift candidates while dashboard browse fact rendering keeps the existing shared browser_detail_info_lines_with output path and behavior.
- Rollback/Risk: Low. This is a local rename plus a regression that guards against reintroducing the generic helper-drift shape.
- Follow-up: Use the roadmap for any remaining non-helper-drift TUI design work.

## 2026-05-25 - Shared datasource review empty lines
- Summary: Moved datasource browse REVIEW empty-state line formatting into a shared browser helper and renamed the datasource review panel builder to avoid a generic helper-drift wrapper.
- Tests: cargo test --quiet browser_review_empty_line_formats_review_prefixed_message; cargo test --quiet datasource_review_panel_does_not_keep_generic_build_review_wrapper; cargo test --quiet datasource_browse; cargo test --quiet interactive_browser; cargo test --quiet datasource (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Datasource browse review empty states now share browser-level REVIEW message formatting while review evidence rendering and public CLI/doc surfaces remain unchanged.
- Rollback/Risk: Low. The shared helper renders the same REVIEW-prefixed message and focused datasource/browser tests cover both empty-state and evidence rows.
- Follow-up: Re-run the TUI inventory report and evaluate the final dashboard browse build_info_lines candidate separately.
