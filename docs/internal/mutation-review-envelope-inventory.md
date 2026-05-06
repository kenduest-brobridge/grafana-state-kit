# Mutation Review Envelope Inventory

This note records the current review/dry-run/apply shapes before introducing a
shared mutation review envelope.

## Current Shared Vocabulary

`rust/src/commands/review_contract.rs` already centralizes action, status,
reason, and hint strings used across plan, preview, apply, and TUI-adjacent
flows.

Current stable vocabulary:

- actions: `would-create`, `would-update`, `would-delete`, `same`,
  `extra-remote`, `unmanaged`, and `blocked-*`
- statuses: `ready`, `same`, `warning`, `blocked`
- reasons: ambiguous live name match, missing target org, provisioned/managed
  target, read-only target, UID/name mismatch
- hints: missing remote, remote-only, requires secret values

## Current Domain Shapes

- Workspace preview/review already has the closest internal adapter:
  `WorkspaceReviewView`, `WorkspaceReviewAction`, `WorkspaceReviewDomain`, and
  `WorkspaceReviewSummary`. `WorkspaceReviewAction` is now an alias of the
  shared `ReviewMutationAction`.
- Dashboard plan, datasource plan, and access plan now project into
  `ReviewMutationActionInput` / `ReviewMutationAction` without changing their
  public JSON contracts.
- Alert plan rows carry the same broad action evidence (`domain`,
  `resourceKind`, `identity`, `actionId`, `action`, `status`, `reason`,
  `blockedReason`, `reviewHints`, `desired`, and `live`), but alert action
  strings still use local `create` / `update` / `noop` / `delete` values rather
  than the shared `would-*` vocabulary.
- Dashboard import dry-run and history restore preview expose review evidence,
  but they are not yet normalized as generic mutation actions.
- Access import dry-run, datasource import dry-run, and datasource live mutation
  preview rows are close enough to review actions to be candidates for the next
  adapter pass, but each still has local gaps:
  - Access import dry-run rows expose `identity`, `action`, `status`, `blocked`,
    `detail`, optional `blockers`, and optional `target`, but not
    `blockedReason` or shared `would-*` actions.
  - Datasource import dry-run rows expose `action`, `blockedReason`,
    `targetUid`, `targetVersion`, and `targetReadOnly`, but not a normalized
    `status`.
  - Datasource live mutation preview rows expose `operation`, `uid`, `name`,
    `type`, `match`, `action`, and `targetId`; failures still use local
    `would-fail-*` action strings.
- Sync apply-intent and live-apply already use the shared apply-result adapter
  through `ReviewApplyResult`, and sync preview adapts plan operations into the
  shared mutation envelope.
- Snapshot review is inventory-oriented and should not be forced into a mutation
  envelope.

## Risk Evidence

`ReviewRisk` should not be introduced yet.

Dashboard governance has a mature risk contract:

- `GovernanceRiskSpec`: `category`, `severity`, and `recommendation`.
- `GovernanceRiskRow`: `kind`, `severity`, `category`, `dashboardUid`,
  `panelId`, `datasource`, `detail`, and `recommendation`.
- Query and dashboard audit rows also carry `score`, `severity`, `reasons`, and
  `recommendations`.

Other domains do not yet prove the same semantics:

- Sync and alert provide preflight/check evidence such as `kind`, `identity`,
  `status`, `detail`, `blocking`, `managedFields`, and review hints, but not a
  stable `severity` / `category` / `recommendation` risk taxonomy.
- Access live status and datasource status expose grouped finding counts such
  as `kind`, `count`, `source`, signal group, and signal key. These are status
  signals, not mutation risk rows.
- Dashboard target ownership review exposes `kind` and ownership/provenance
  evidence, but that is apply-safety evidence rather than governance risk.

Treat blocker/check/status evidence as domain-local until a second mutation
review domain emits dashboard-grade risk fields.

## Request Evidence

`ReviewRequest` should not be introduced yet.

Request-like shapes exist, but they represent different layers:

- Dashboard review source args carry command/source context such as `common`,
  `page_size`, `org_id`, `all_orgs`, `input_dir`, `input_format`,
  `input_type`, `governance`, `queries`, and `require_queries`.
- Dashboard plan reports carry review context such as `mode`, `scope`,
  `inputType`, `inputLayout`, and `prune`.
- Sync apply-intent operations carry mutation intent: `kind`, `identity`,
  `action`, `desired`, `ownership`, and `provenance`.
- Datasource import has a transport-like internal request plan:
  `PreparedDatasourceImportRequest { method, path, payload }`.
- Access workflows mostly use request closures directly and expose dry-run
  intent rows rather than reusable request specs.

Do not merge command provenance, mutation intent, and HTTP transport request
specs under one `ReviewRequest` name until at least two domains need the same
layer and fields.

## Constraints

- Do not change public JSON contracts just to introduce the shared envelope.
- Keep domain payloads behind an adapter until at least two concrete domains use
  the same fields.
- TUI consumers should read a normalized view model, not drive plan-builder
  shape decisions.
- Shared fields should be derived from existing domain outputs, not invented
  ahead of real consumers.

## Candidate Internal Envelope

Continue with internal adapters only:

- `action_id`
- `domain`
- `resource_kind`
- `identity`
- `action`
- `status`
- `reason`
- `blocked_reason`
- `review_hints`
- `raw`

Do not add `risk` to this envelope until there is cross-domain risk evidence.
Do not promote this to a public contract until the adapter has covered the next
dry-run/import surfaces without requiring lossy mapping.

## Next Implementation Order

1. Keep `ReviewRisk` and `ReviewRequest` blocked.
2. Extend shared mutation action adapters only where existing rows already prove
   action/status/blocked-reason semantics:
   - access import dry-run rows,
   - datasource import dry-run rows,
   - datasource live mutation preview rows,
   - alert plan rows after action vocabulary mapping is explicit,
   - legacy dashboard import dry-run only after its `would-skip-*` /
     `would-fail-*` actions are mapped without hiding operator-facing reasons.
3. Keep public JSON unchanged in the first adapter pass; expose normalized views
   only internally for TUI, summaries, or focused tests.
