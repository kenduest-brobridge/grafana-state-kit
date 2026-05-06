# `grafana-util access browse`

## Purpose

Browse live users, teams, orgs, and service accounts in one access review TUI.

## When to use

- You need one read-only access inventory view before choosing a narrower access command.
- You want to review user, team, org, and service-account rows together.
- You need service-account token context without exposing token secrets.

## Description

`access browse` loads live access inventory and opens an interactive review surface. It is a consolidation view: use it to inspect identities and choose the next action, then use `access user`, `access team`, `access org`, `access service-account`, or `access plan` for focused lifecycle work.

Service-account rows show role, disabled state, org id, and token metadata/counts only. Token secret values are never displayed.

## Workflow lanes

- **Inspect**: review access inventory across users, teams, orgs, and service accounts.
- **Choose the next command**: decide which resource-specific command should handle the follow-up.
- **Review Before Mutate**: confirm the affected identity family before running import, delete, token, or plan commands.

## Key flags

- `--query`: filter rows by user, team, org, or service-account text.
- `--include-users`, `--include-teams`, `--include-orgs`, `--include-service-accounts`: limit the inventory families. If none are set, all are included.
- `--per-page`: user page size for global user inventory.
- `--profile`, `--url`, `--token`, `--basic-user`, `--basic-password`.

## Examples

```bash
# Browse all access families interactively.
grafana-util access browse --url http://localhost:3000 --basic-user admin --basic-password admin
```

```bash
# Browse only user and team rows matching ops.
grafana-util access browse --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --include-users --include-teams --query ops
```

## Related commands

- [access](./access.md)
- [access user](./access-user.md)
- [access team](./access-team.md)
- [access org](./access-org.md)
- [access service-account](./access-service-account.md)
- [access plan](./access-plan.md)
