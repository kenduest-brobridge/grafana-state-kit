# dashboard import

## Purpose
Import classic dashboard JSON or native dashboard schema v2 resource JSON through the matching Grafana API.

## When to use
Use this when you have a local export tree for dashboards that are owned by Grafana's API state and need to push dashboards back into Grafana, either live or as a dry run. Classic import consumes `raw/` or `provisioning/` inputs. Native schema v2 import consumes `resource-v2/` JSON resources only when you explicitly pass `--input-format resource-v2`. The command does not consume the Grafana UI `prompt/` lane.

Do not use this command to bypass source-owned dashboards. If the target dashboard is managed by Grafana Git Sync or file provisioning, change the repository, PR, or provisioning source and let that workflow deploy it instead of forcing a direct dashboard API import.

## Before / After
- **Before**: import is a blind replay step, and you find folder, org, or schema problems only after the live call.
- **After**: import becomes a controlled replay step that can be previewed first with `--dry-run`, then executed with clearer intent.

## Key flags
- `--input-dir`: source directory for a classic lane, an exported `resource-v2/` directory, or a combined export root.
- `--input-format`: choose `raw`, `provisioning`, or the explicit native resource lane `resource-v2`.
- `--resource-namespace`: App Platform namespace for `resource-v2` import. It defaults to `default`; the effective value is authoritative and replaces any namespace carried in a source document.
- `--local`, `--run`, `--run-id`: read the source dashboard lane from the artifact workspace instead of passing `--input-dir`.
- `--org-id`, `--use-export-org`, `--only-org-id`, `--create-missing-orgs`: control cross-org routing.
- `--import-folder-uid`: force a destination folder UID.
- `--ensure-folders`, `--replace-existing`, `--update-existing-only`: control import behavior.
- `--require-matching-folder-path`, `--require-matching-export-org`, `--strict-schema`, `--target-schema-version`: safety checks.
- `--import-message`: revision message stored in Grafana.
- `--interactive`, `--dry-run`, `--table`, `--json`, `--output-format`, `--output-columns`, `--list-columns`, `--no-header`, `--progress`, `--verbose`: preview and reporting controls. Use `--output-columns all` for the full dry-run table.

## What success looks like
- dry-run shows the expected create/update actions before you touch the live server
- dry-run also surfaces live dashboard target evidence when Grafana already owns the UID, including provisioned or managed-state warnings
- the destination org and folder routing are explicit enough to review
- the chosen input lane matches the replay goal: `raw`, `provisioning`, or explicit `resource-v2`, not `prompt`
- Git Sync or file-provisioned targets are treated as source-owned and routed back to their repository or provisioning workflow

## Native resource-v2 safety boundary

`--input-format resource-v2` accepts JSON resources with `apiVersion: dashboard.grafana.app/v2`, `kind: Dashboard`, a stable non-empty `metadata.name`, and an object-valued `spec`. It preserves the v2 schema instead of converting it to classic dashboard JSON.

This lane talks only to `/apis/dashboard.grafana.app/v2/namespaces/<namespace>/dashboards`. It never falls back to the legacy `/api/dashboards/db` endpoint. A `404` from the v2 API means the server does not support this import lane; authentication and authorization failures remain errors.

Resource-v2 import is single-namespace. The following classic routing and validation flags are incompatible: `--org-id`, `--use-export-org`, `--only-org-id`, `--create-missing-orgs`, `--require-matching-export-org`, `--require-matching-folder-path`, `--ensure-folders`, `--strict-schema`, and `--target-schema-version`.

Run `--dry-run` first to review create, update, no-change, skip, or blocked actions without sending mutations. A missing resource can be created. An existing resource is updated only with `--replace-existing`; `--update-existing-only` skips missing resources. Direct updates remain blocked when Grafana reports Git Sync or file-provisioning ownership, and there is no ownership-bypass flag. Resource import does not delete, prune, or roll back dashboards.

Resource-v2 dry-run text, table, and JSON include the source path, API version, namespace, resource name, folder, ownership, blocker, and changed fields. Table output accepts `api_version`, `namespace`, `name`, `source_path`, `action`, `folder`, `ownership`, `blocker`, and `changed_fields` in `--output-columns`. With `--interactive` and a TUI-enabled build, the same review evidence is searchable and scrollable; create/update resources are written only after explicit confirmation. Interactive mode requires stdin and stdout TTYs.

JSON dry-run output is one versioned document with `kind`, `mode`, `dashboards`, and action `summary` fields. Combined-root index paths and standalone discovery must remain inside the selected input root; symbolic links are rejected. Grafana-added v2 schema defaults are normalized for comparison so replaying an unchanged source reports `no_change` instead of a false update.

Live resource-v2 apply is sequential and stops at the first failed write. Its error reports completed, failed, and not-attempted resource identities, preserves the original HTTP status, and explicitly states that no rollback was attempted.

## Failure checks
- if folder or org placement looks wrong, verify the routing flags before re-running live import
- if the replay looks too destructive, stop at `--dry-run` and inspect the export tree first
- if Grafana reports a provisioned or Git Sync-managed dashboard at the target UID, do not retry with a direct import; update the owning source and redeploy through that lane
- if the schema check blocks replay, confirm whether the source tree needs normalization before import
- if resource-v2 reports that the API is unavailable, verify Grafana serves the v2 `/apis` endpoint; do not retry the same resource through classic import
- if resource-v2 selects the wrong namespace, pass `--resource-namespace` explicitly and review the dry-run; source `metadata.namespace` is not authoritative

## Examples
```bash
# Import dashboard JSON files through the Grafana API.
grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw --replace-existing
```

```bash
# Import dashboard JSON files through the Grafana API.
grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw --dry-run --table
```

```bash
# Dry-run import from the latest profile artifact workspace dashboard run.
grafana-util dashboard import --profile prod --local --dry-run --table
```

```bash
# Preview native schema v2 resources in the authoritative default namespace.
grafana-util dashboard import --profile prod --input-dir ./dashboards/resource-v2 --input-format resource-v2 --resource-namespace default --dry-run --table
```

```bash
# Apply creates and permitted updates through the native /apis v2 endpoint.
grafana-util dashboard import --profile prod --input-dir ./dashboards/resource-v2 --input-format resource-v2 --resource-namespace default --replace-existing
```

## Related commands
- [dashboard export](./dashboard-export.md)
- [dashboard convert raw-to-prompt](./dashboard-convert-raw-to-prompt.md)
- [dashboard diff](./dashboard-diff.md)
- [dashboard publish](./dashboard-publish.md)
