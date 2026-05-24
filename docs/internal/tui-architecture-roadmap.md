# TUI Architecture Roadmap

Phase 0 owns the inventory and planning layer plus low-risk, focused browse
maturity cleanup where the behavior is already covered by narrow tests. The
follow-on consistency pass aligned shared search/status/control language across
several existing Rust TUI surfaces without changing public CLI paths.

## Current Inventory

| Surface | Entrypoint | Current tier | Notes |
| --- | --- | --- | --- |
| Dashboard browse | `grafana-util dashboard browse` | Mature | Dedicated live/local tree browser under `rust/src/commands/dashboard/browse/`; keep current ownership disjoint. |
| Access browse | `grafana-util access browse`, `access user browse`, `access team browse` | Active implementation | Consolidated access browse now has in-session filtering, selection summaries, and shared-shell header/footer language. User/team specialized browsers remain separate richer flows; repeat-search now skips the selected row and wraps across matching rows. |
| Datasource browse | `grafana-util datasource browse`, local `datasource list --interactive` | Active implementation | Live/local datasource browser under `rust/src/commands/datasource/browse/`; it surfaces row, kind, search context, repeat-search wrap, and a first-class Review pane for secret/provider/read-only plus compatible local plan/diff/import evidence carried in datasource details. Local `datasource list --interactive` reuses that Review projection in its item details for already-loaded artifacts. |
| Status overview | `grafana-util status overview live --output-format interactive` and staged/live status interactive output | Mature document browser | Uses the overview/status document model, then projects into TUI. It now supports current-view `/`, `?`, and `n` item search with explicit search status in the shell. |
| Dashboard summary / inspect workbench | `grafana-util dashboard summary --interactive` | Mature review workbench | Query, dashboard, and governance rows share the inspect workbench. |
| Dashboard import review | `grafana-util dashboard import --interactive` | Mature but specialized | Client-backed selector and focused review flow are import-specific. Keep changes evidence-led. |
| Dashboard policy / dependencies / impact / topology | `--interactive` review modes | Mixed maturity | Useful precedent for shared review navigation, but several surfaces still have domain-specific runners. |
| Snapshot review | `grafana-util snapshot review --interactive` | Document-backed review | Browser-style review over snapshot artifacts. |
| Sync audit/review | `grafana-util sync ... --interactive` internal flows | Internal review surface | Keeps shared review ideas, but public docs should continue steering users through `workspace` where applicable. |
| Shared TUI shell | `rust/src/common/tui/` and `rust/src/common/browser/session.rs` | Shared primitive | Owns visual shell/session primitives and read-only browser search. Shared read-only browser repeat-search skips the current row and wraps within the active filter. It is not yet a complete domain-neutral workbench framework. |

## Maturity Tiers

- Tier 0: terminal prompts, such as `--prompt`, that are not full-screen TUI.
- Tier 1: one-off interactive selectors that are tied to one command and one data shape.
- Tier 2: domain workbenches that separate data collection from render/input state.
- Tier 3: document-backed browsers where text/JSON/TUI consume the same domain document.
- Tier 4: shared review framework where actions, risks, hints, and navigation are reusable across domains.

The target shape is Tier 3 for inventory/read surfaces and Tier 4 for review or
mutation previews. New work should avoid adding another Tier 1 surface unless
the command is explicitly experimental.

## Architecture Debt

- Public flag language is still mixed: `--interactive` is used for full-screen
  TUI, while some older human-operated flows were migrated to `--prompt`.
- Shared visual shell helpers and shared read-only browser search exist, but many
  workbenches still own local state, footer controls, filtering, and focus
  models.
- Feature gating is still coarse. Default builds include `tui`; `--no-default-features`
  must keep clear errors for interactive paths, but TUI modules are not fully
  isolated behind a separate artifact lane.
- Several review surfaces duplicate similar action/status/risk projection logic
  instead of consuming one normalized review model.
- Docs mention interactive surfaces across command pages and handbooks, but there
  has not been a lightweight inventory report to keep architecture discussions
  grounded in current files.

## Approved Phase Plan

1. Phase 0: document the inventory, tiers, debt, and next phases; add a
   read-only report script if it fits repo conventions; and allow focused
   access/datasource browse maturity cleanup when covered by narrow tests.
2. Phase 1: define shared TUI ownership rules for document-backed browsers,
   review workbenches, prompt-vs-TUI naming, and `tui` feature fallback messages.
3. Phase 2: extract common review workbench contracts only where two or more
   domains already prove the same shape. Start from read-only projections before
   mutation actions.
4. Phase 3: migrate one low-risk Tier 1 or Tier 2 surface to the shared contract
   with focused parser/render tests and no public CLI drift.
5. Phase 4: update command docs and generated artifacts only after behavior or
   public CLI wording changes. Run the command-surface and generated-doc checks
   for those later phases.

## Phase 0 Report Script

Run the current inventory helper manually when planning TUI work:

```bash
python3 scripts/tui_inventory_report.py
```

The script is intentionally non-blocking and is not wired into CI. It scans
`rust/src`, English command docs, English user guide docs, and `docs/internal`
while skipping generated HTML and Cargo build output.

## Recent Follow-Up

- `access user browse` and `access team browse` repeat-search behavior now skips
  the selected row and wraps in line with datasource browse and status overview,
  so `n` can continue within the current result set after reaching the first or
  last matching row.
- `datasource browse` now uses the same boundary rule, so repeated search does
  not reselect the first or last matching row before wrapping.
- `datasource browse` now separates secret placeholder, blocker, and
  review-required evidence into a dedicated Review pane, leaving general
  datasource metadata in Facts and keeping resolved credential values hidden.
- `datasource browse` Review lines now also recognize compatible review
  evidence fields already carried in datasource details, including action,
  status, match basis, destination, source file, target UID/version/read-only,
  blocked reason, changed fields, review hints, and secret-value requirements.
  Secret-like changed-field paths such as `secureJsonData.*` stay hidden.
- Local `datasource list --interactive` now appends the same safe Review
  evidence to datasource item details when local inventory/provisioning records
  already carry compatible plan/diff/import fields. This reuses the browse
  projection without adding local input flags to the live browse command.
- The shared read-only browser now applies the same repeat-search boundary rule
  as datasource/access/status browsers: `n` skips the selected row first, then
  wraps forward or backward within the active filter.
- Dashboard inspect workbench footer controls now use the same compact `Esc/q`
  exit label as the other Rust TUI browse/review surfaces.
- Dashboard inspect workbench full-detail viewer status and footer controls now
  use compact `Enter/Esc/q` close language instead of split close hints.
- Dashboard inspect workbench repeat-search now also skips the selected row
  before wrapping backward at the first matching row.
- Dashboard browse repeat-search now follows the same backward boundary rule,
  so it does not reselect the first matching tree node before wrapping.
- Snapshot review datasource rows now preserve safe local review evidence and
  reuse the same datasource Review projection in browser details.
- Sync review TUI operation previews now reuse the datasource safe changed-field
  rule, so secret-like paths such as `secureJsonData.*` stay out of the preview.
- Access user/team and datasource browse destructive confirmation dialogs now
  use compact `Confirm: y` and `Cancel: n/Esc/q` body copy.
- Access user/team, dashboard browse, datasource browse, dashboard inspect
  workbench, and status overview search prompts now share the compact
  `Enter search`, `Esc cancel`, and `n repeat` hint language.
- The shared read-only browser now also uses the compact pending search summary
  and footer copy, so downstream artifact browsers do not regress to
  `Enter apply` wording or advertise repeat while the prompt is accepting text.
- Shared read-only browser detail titles now use the filtered visible position
  and total, matching the list numbering and footer selection count.
- Dashboard policy/governance gate TUI now supports current-group finding
  search/filtering and surfaces idle, active, and prompt search state in the
  footer.
- Dashboard impact TUI now supports current-group item search/filtering and
  surfaces idle, active, and prompt search state in the footer.
- Dashboard topology/dependencies TUI now supports the same current-group node
  search/filtering pattern and footer search state copy.
- Sync audit TUI now supports current-group drift row search/filtering and the
  same idle, active, and prompt search state footer copy.
- Sync review checklist mode now supports current-view operation
  search/filtering with the same footer search state copy while keeping `n` as
  the existing select-none command.
- Access plan interactive review now consumes shared review-contract action
  detail lines for generic action, identity, status, blocker, and detail
  evidence while keeping its access-specific narrative and next-check guidance.
- Dashboard TUI feature fallback now uses the shared `tui_feature_required`
  error helper so dashboard interactive paths report the same TUI error
  category and `requires the `tui` feature` wording as the other Rust TUI
  surfaces.
- Dashboard import review footer copy now advertises the existing page jump,
  bounds jump, and `Esc/q`/`Ctrl-C` cancel controls instead of only `q`.
- Dashboard browse delete review summary now splits `Confirm: y` from
  `Cancel: n/Esc/q`, matching the footer/input behavior used by the other
  destructive TUI confirmations.
- Status TUI detail scrolling now clamps to the selected detail content so
  repeated scroll keys cannot move the detail pane beyond its rendered lines.
- Status TUI footer copy now advertises the existing `Home/End` jump behavior
  alongside movement, detail scroll, and `Esc/q` exit controls.
- Status TUI now supports `/`, `?`, and `n` search over domain and action lists
  with the same compact search prompt hint language used by the browse surfaces.
- Dashboard browse header context now matches datasource browse by surfacing the
  selected row position, selected node kind, and current search state.
- Dashboard browse overlay modes such as search now surface as explicit header
  modes instead of continuing to read as plain browse/local-browse.
- Datasource browse edit dialog now wraps Tab/Shift+Tab field navigation and
  advertises both next and previous field controls in the dialog header.
- The change stayed in state/tests. Public CLI/docs and generated docs remain
  unchanged because the user-facing command surface did not change.

## Next Follow-Up

- Continue reducing duplicated TUI review/detail projection on the remaining
  local artifact browsers that can consume shared safe review lines without
  changing public CLI flags; datasource local and snapshot datasource rows
  already reuse the datasource Review projection, and access plan generic action
  evidence now reuses the shared review-contract projection.
- Continue tightening `--no-default-features` TUI isolation. A focused dashboard
  fallback test can run under the default feature set, but the current
  no-default Rust test build still fails earlier on unrelated TUI-gated module
  imports.
