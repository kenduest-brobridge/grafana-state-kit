# `grafana-util status`

## Root

Purpose: read live and staged Grafana state through one canonical status surface.

When to use: when you want readiness, overview, snapshot, or direct live reads without moving into mutation work.

Description: `status` is the user-facing read-only entrypoint. Use `live` for current-state gating, `staged` for local artifact review, `overview` for project-wide summaries, `snapshot` for offline bundle capture and review, and `resource` for generic live reads when a richer domain workflow is not needed yet.

## Command groups

- Live Read-Only: `live`, `capabilities`, `overview`, `resource`, `snapshot`
- Staged Review: `staged`, `overview`

Open this page when you know the task is read-only but you still need to decide whether the next step is a direct health check, a generic resource lookup, a project-wide overview, or a snapshot export for later review.

## Before / After

- **Before**: read-only Grafana checks are often split across one-off scripts, direct API calls, ad hoc exports, and workflow-specific commands.
- **After**: the `status` namespace keeps the common read surfaces together so operators can choose the right read path without immediately entering mutation workflows.

## What success looks like

- the next read path is obvious before you open a deeper command page
- live checks, staged review, and snapshot review share one predictable namespace
- generic resource reads stay available without forcing a full export or domain-specific flow

## Failure checks

- if a live command returns less data than expected, confirm the auth scope and org visibility first
- if a staged overview looks incomplete, verify the input directories and file paths before assuming the parser is wrong
- if downstream automation reads the output, set `--output-format` explicitly on the leaf command

### Key flags

- the root `status` command is a namespace; operational flags live on `live`, `staged`, `overview`, `snapshot`, and `resource`
- `--help-schema` is available on the `status` surface for schema-oriented help paths

### Examples

```bash
# inspect the read-only entrypoint before choosing a lane.
grafana-util status --help
```

```bash
# check live Grafana reachability and health.
grafana-util status live --profile prod --output-format yaml
```

```bash
# review staged desired-state artifacts before a preview or apply flow.
grafana-util status staged --desired-file ./desired.json --output-format json
```

```bash
# summarize one staged dashboard and alert tree.
grafana-util status overview --dashboard-export-dir ./dashboards/raw --alert-export-dir ./alerts --output-format table
```

```bash
# open the live overview path as an interactive workbench.
grafana-util status overview live --url http://localhost:3000 --basic-user admin --basic-password admin --output-format interactive
```

```bash
# describe the supported generic resource selectors.
grafana-util status resource describe dashboards --output-format json
```

```bash
# export a snapshot bundle for later offline review.
grafana-util status snapshot export --profile prod --output-dir ./snapshot
```

## Related commands

- `grafana-util export`
- `grafana-util workspace`
- `grafana-util config profile`

## `live`

Purpose: render a live readiness view from Grafana read surfaces.

Machine-readable JSON/YAML output includes the shared project status contract:
`kind`, `schemaVersion`, `toolVersion`, `scope`, `overall`, `domains`,
`topBlockers`, and `nextActions`. Live output also adds
`discovery.instance` from Grafana `GET /api/health` when the endpoint can be
read. That section carries `source: api-health`, `status: available`, and the
raw health fields such as `database`, `version`, and `commit`. If the health
read fails, `discovery.instance.status` is `unavailable` with an `error`; this
does not by itself block the domain readiness result.

Successful instance metadata:

```json
{
  "discovery": {
    "instance": {
      "source": "api-health",
      "status": "available",
      "health": {
        "database": "ok",
        "version": "12.4.0",
        "commit": "abc123"
      }
    }
  }
}
```

Unavailable health metadata:

```json
{
  "discovery": {
    "instance": {
      "source": "api-health",
      "status": "unavailable",
      "error": "..."
    }
  }
}
```

Use `discovery.instance.health.version` and
`discovery.instance.health.commit` when automation needs the Grafana build.
Use `overall` and `domains` for readiness gates.

## `staged`

Purpose: render a readiness gate from staged artifacts.

## `capabilities`

Purpose: check dashboard list-read API availability for the current credentials
and an explicit Grafana namespace before choosing a dashboard workflow.

```bash
# Check dashboard API availability with the prod profile.
grafana-util status capabilities --profile prod --namespace default --output-format json
```

The command makes three GET requests, each with `limit=1`: Classic
`/api/search?type=dash-db&page=1`, and the dashboard resource `v1` and `v2`
collection routes under `/apis/dashboard.grafana.app/`. It does not page through
dashboards or modify Grafana. `--namespace` is required; `--org-id` scopes the
credentials' organization independently and does not select the namespace.
Choose the namespace used by your Grafana instance.

JSON/YAML output contains `kind: grafana-dashboard-api-capabilities`,
`schemaVersion: 1`, `namespace`, and `probes`. Each probe has `api`, `path`,
`status`, `detail`, and `httpStatus` when a non-success HTTP response was received.
An empty, valid list is `available`. This establishes only a list read, not write
permission, dashboard inventory completeness, or acceptance of a particular
dashboard document. Probe results are observations from this run and are not cached.

| Status | Meaning |
| --- | --- |
| `available` | The expected list contract was returned. |
| `unauthorized` | HTTP 401; fix authentication and retry. |
| `forbidden` | HTTP 403; check read permissions and retry. |
| `not-served` | HTTP 404; the requested route or namespace was not found. |
| `failed` | Another HTTP error, including 5xx, or a transport failure. |
| `malformed-response` | Invalid JSON, no response body, or an unexpected list contract. |

401/403, 404, failures, and malformed responses do not prove global API support
or lack of support. In particular, a 404 can reflect the selected namespace or a
proxy route. The command prints all probe observations and exits successfully
even if a probe fails; automation should inspect each `status`. Invalid local
arguments or connection setup still produce a command error. Reports omit
credentials, response bodies, and transport error details.

API routes and list semantics follow the official
[Dashboard API](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/dashboard/)
and [Classic search API](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/api-legacy/folder_dashboard_search/).

## `overview`

Purpose: summarize staged artifacts or open the live overview.

## `snapshot`

Purpose: review or export a snapshot-style artifact bundle.

## `resource`

Purpose: inspect supported live Grafana resources through a generic read-only query surface.
