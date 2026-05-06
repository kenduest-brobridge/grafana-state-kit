# `grafana-util access browse`

## 用途

在同一個 access review TUI 中瀏覽 live user、team、org 與 service account。

## 何時使用

- 你需要先用唯讀方式盤點 access inventory，再決定要進哪個更窄的 access 指令。
- 你想一起檢視 user、team、org 與 service-account row。
- 你需要 service-account token 脈絡，但不能顯示 token secret。

## 說明

`access browse` 會讀取 live access inventory，並開啟互動式 review 畫面。這是整合檢視：先用它檢查 identity 與影響範圍，再用 `access user`、`access team`、`access org`、`access service-account` 或 `access plan` 做後續生命週期操作。

Service-account row 只顯示 role、disabled state、org id 與 token metadata/count。永遠不顯示 token secret value。

## 工作流

- **Inspect**：跨 user、team、org、service account 檢視 access inventory。
- **Choose the next command**：決定後續要交給哪個 resource-specific command。
- **Review Before Mutate**：在 import、delete、token 或 plan 指令前確認影響的 identity 類型。

## 重要旗標

- `--query`：依 user、team、org 或 service-account 文字過濾 row。
- `--include-users`、`--include-teams`、`--include-orgs`、`--include-service-accounts`：限制要載入的 inventory 類型。若都沒指定，會包含全部。
- `--per-page`：global user inventory 的分頁大小。
- `--profile`、`--url`、`--token`、`--basic-user`、`--basic-password`。

## 範例

```bash
# 互動式瀏覽所有 access 類型。
grafana-util access browse --url http://localhost:3000 --basic-user admin --basic-password admin
```

```bash
# 只瀏覽符合 ops 的 user 與 team row。
grafana-util access browse --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --include-users --include-teams --query ops
```

## 相關指令

- [access](./access.md)
- [access user](./access-user.md)
- [access team](./access-team.md)
- [access org](./access-org.md)
- [access service-account](./access-service-account.md)
- [access plan](./access-plan.md)
