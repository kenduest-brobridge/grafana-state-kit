# Grafana 13 Review Follow-up Fixes

## Status

- State: active
- Branch: `codex/review-followup-fixes`
- Worktree: `.worktrees/review-followup-fixes`
- Source review range: `a32c12c..9496bce`

## Goal

Correct every confirmed regression found by the post-merge Grafana 13 review
without changing classic dashboard import or mutation behavior.

## Requirements

- Represent resource history with `metadata.generation` as the user-facing
  revision number, retain `resourceVersion` for historical reads, and surface
  the documented `grafana.app/updatedBy` actor.
- Honor history `--limit` as a maximum result count while still supporting App
  Platform pagination.
- Make a resource-only dashboard export enumerate and write App Platform
  objects without legacy datasource/search/dashboard calls.
- Permit an explicit App Platform namespace for dashboard resource export so
  Grafana Cloud `stacks-<STACK_ID>` namespaces work.
- Make Git Sync root detection cycle-safe and avoid following directory
  symlinks.
- Cover each correction with a red-green Rust regression test, refresh public
  command docs/contracts/generated artifacts if CLI help changes, then run the
  relevant Rust and documentation gates.

## Execution Plan

- [x] Add failing history tests for generation/updatedBy and global limit.
- [x] Implement history metadata and bounded pagination fixes.
- [x] Add a failing resource-only export test that rejects every legacy API
      request, then implement resource-lane enumeration and export.
- [x] Add an export namespace CLI option, Cloud namespace regression coverage,
      and update command documentation/contracts/generated output.
- [x] Add a symlink-cycle Git Sync root regression test and make discovery
      symlink-safe.
- [x] Run focused and broad verification, update this plan and the todo index,
      then commit by concern.

## Acceptance Checks

- `dashboard history list/export --history-source v1|v2 --limit N` returns at
  most N newest entries, reports `generation`, and includes the updater actor.
- A dashboard export with every classic variant disabled and `--resource-format
  v2` makes only required App Platform and organization-context requests.
- `dashboard export --resource-namespace stacks-123` uses that namespace for
  every App Platform request.
- A Git repository containing a directory symlink cycle completes layout
  detection without recursion or a stack overflow.
