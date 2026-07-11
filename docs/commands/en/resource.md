# `grafana-util status resource`

## Purpose
Run generic read-only Grafana resource queries through the `status` namespace.

## When to use
Use this when you need a read-only live lookup before a richer workflow exists in `dashboard`, `alert`, `datasource`, `access`, or `workspace`.

## Description
This namespace is intentionally narrower and more generic than the main operator workflows. It exists so you can inspect a supported live Grafana resource kind without waiting for a full domain-specific command surface. Treat it as a read-only utility surface, not as the primary entrypoint for day-to-day mutation work.

## Workflow
- Start with `status resource describe` to see the supported selector patterns and endpoint shape for each kind.
- Use `status resource kinds` to see the currently supported live resource kinds.
- Use `status resource list <kind>` when you need inventory for one kind.
- Use `status resource get <kind>/<identity>` when you need the full live payload for one item.

## Supported kinds
- `dashboards`
- `folders`
- `datasources`
- `alert-rules`
- `orgs`
- `receivers`, `routing-trees`, `template-groups`, `time-intervals`, `inhibition-rules`
- `secure-values` (metadata only; `spec.value` is stripped defensively)

## Output
- `kinds` supports `text`, `table`, `json`, and `yaml`
- `list` supports `text`, `table`, `json`, and `yaml`
- `get` supports `text`, `table`, `json`, and `yaml`
- `--api-mode` accepts `auto`, `legacy`, `v1`, `v2`, and `v1beta1`.
- Dashboard `auto` tries App Platform v2, then v1, then classic endpoints, falling back only on 404. Folder `auto` tries v1 before classic. Datasource UID lookup keeps its numeric legacy fallback.
- Notification and secure-value resources are public preview and require explicit `--api-mode v1beta1`; they are never auto-probed.
- `--namespace` selects the App Platform namespace and defaults to `default`.

## Recommended mode
- Prefer default `auto` for stable dashboard/folder inspection. Use explicit `v1` or `v2` when a script requires one envelope, `legacy` for classic-only access, and `v1beta1` only for the documented preview resources.

## Examples
```bash
# Describe the supported live resource kinds and selector patterns.
grafana-util status resource describe
```

```bash
# Show the currently supported generic resource kinds.
grafana-util status resource kinds
```

```bash
# List live dashboard resources from a local Grafana.
grafana-util status resource list dashboards --url http://localhost:3000 --basic-user admin --basic-password admin
```

```bash
# Fetch one datasource payload as YAML.
grafana-util status resource get datasources/prom-main --profile prod --output-format yaml
```

```bash
# Force legacy compatibility mode when needed.
grafana-util status resource get datasources/10 --api-mode legacy
```

```bash
# Inspect Grafana 13 notification receivers through the explicit preview API.
grafana-util status resource list receivers --api-mode v1beta1 --namespace default --profile prod --output-format json
```

`secure-values` is intentionally read-only metadata inspection. The command has no export or mutation path and removes `spec.value` even if a hostile or nonconforming server sends it.

## Related commands
- [resource describe](./resource-describe.md)
- [resource kinds](./resource-kinds.md)
- [resource list](./resource-list.md)
- [resource get](./resource-get.md)
- [dashboard](./dashboard.md)
