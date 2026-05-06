# `grafana-util alert apply`

## Purpose

Apply a reviewed alert management plan.

## When to use

- Execute a reviewed plan that contains narrow alert-rule update rows.
- Require explicit acknowledgement before touching Grafana.
- Keep broader alerting changes staged until their mutation semantics are explicit.

## Safety boundary

`alert apply` is a controlled live-apply lane. It currently permits only ready alert-rule `update` rows whose changed fields are limited to `condition`, `data`, `for`, `noDataState`, or `execErrState`.

Create, delete, contact-point, mute-timing, policy, template, and ambiguous rule-field changes fail closed. Use `alert plan` and review output to keep those broader changes visible without applying them directly.

## Before / After

- **Before**: a reviewed alert plan still leaves one risky question open: what actually happens when someone presses go against live Grafana?
- **After**: `alert apply` turns that final step into an explicit command with approval, reproducible auth, and machine-readable output.

## Key flags

- `--plan-file` points to the reviewed plan document.
- `--approve` is required before execution is allowed.
- `--output-format` renders apply output as `text` or `json`.

## Examples

```bash
# Apply a reviewed alert management plan.
grafana-util alert apply --plan-file ./alert-plan-reviewed.json --approve
```

```bash
# Apply a reviewed alert management plan.
grafana-util alert apply --url http://localhost:3000 --basic-user admin --basic-password admin --plan-file ./alert-plan-reviewed.json --approve
```

## What success looks like

- narrow reviewed alert-rule updates can be applied without hand-editing YAML or replaying a sequence of UI clicks
- the live apply step keeps approval explicit instead of hiding it in shell history
- JSON output is stable enough to feed into CI, workspace records, or a post-apply verification step

## Failure checks

- if apply refuses to run, confirm that the plan file is the reviewed artifact and that `--approve` is present
- if the live result differs from the expected plan, re-check credentials, org scope, and whether the reviewed plan matches the current target Grafana
- if automation reads the output, prefer `--output-format json` and validate the result shape before treating the apply as successful

## Related commands

- [alert](./alert.md)
- [alert plan](./alert-plan.md)
- [alert delete](./alert-delete.md)
