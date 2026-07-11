# `grafana-util status resource get`

## Purpose
Fetch one supported live Grafana resource by selector.

## When to use
Use this when you need the live payload for one supported resource item and want a simpler generic query than building a full export or inspect workflow.

## Key flags
- positional `SELECTOR`: required `<kind>/<identity>` selector, for example `dashboards/cpu-main` or `datasources/prom-main`
- `--profile`, `--url`, `--token`, `--basic-user`, `--basic-password`: live Grafana connection settings
- `--output-format`: choose `text`, `table`, `json`, or `yaml`
- `--api-mode`: `auto`, `legacy`, `v1`, `v2`, or explicit preview `v1beta1`
- `--namespace`: App Platform namespace, default `default`

## Notes
- Selector kinds include stable resources plus the Grafana 13 preview notification and `secure-values` kinds listed by `status resource kinds`.
- `secure-values` never returns secret material through this command; `spec.value` is stripped before text, table, JSON, or YAML rendering.
- `text` and `table` outputs summarize the fetched object. Use `json` or `yaml` when you need the full payload.

## Examples
```bash
# Fetch one live dashboard by UID.
grafana-util status resource get dashboards/cpu-main --url http://localhost:3000 --basic-user admin --basic-password admin
```

```bash
# Fetch one datasource payload as YAML.
grafana-util status resource get datasources/prom-main --profile prod --output-format yaml
```

```bash
# Fetch one org payload by numeric ID.
grafana-util status resource get orgs/1 --profile prod --output-format json
```

```bash
# Prefer the legacy datasource numeric-ID path.
grafana-util status resource get datasources/10 --api-mode legacy --url http://localhost:3000
```

## Related commands
- [resource](./resource.md)
- [resource describe](./resource-describe.md)
- [resource kinds](./resource-kinds.md)
- [resource list](./resource-list.md)
