# `grafana-util status resource`

## 用途
用通用、唯讀的查詢介面讀取少數幾種 live Grafana 資源。

## 何時使用
當你需要先做唯讀的 live lookup，但 `dashboard`、`alert`、`datasource`、`access`、`workspace` 還沒有對應的高階工作流時，可以先用這個命名空間。

## 說明
這個命名空間刻意比主要維運工作流更通用、更窄。它的目的不是取代既有的高階操作，而是提供一條唯讀資源查詢路徑，讓你能先檢查少數支援的 live Grafana resource kinds。把它當成通用查詢工具，不是日常 mutation 的主入口。

## 使用方式
- 先用 `status resource describe` 看各 kind 的 selector 格式與 endpoint 形狀。
- 先用 `status resource kinds` 看目前支援哪些 live resource kinds。
- 需要盤點某一類資源時，用 `status resource list <kind>`。
- 需要抓某一筆完整 live payload 時，用 `status resource get <kind>/<identity>`。

## 目前支援的 kinds
- `dashboards`
- `folders`
- `datasources`
- `alert-rules`
- `orgs`
- `playlists`（stable v1，唯讀）
- `receivers`、`routing-trees`、`template-groups`、`time-intervals`、`inhibition-rules`
- `secure-values`（只讀 metadata；會防禦性移除 `spec.value`）

## 輸出
- `kinds` 支援 `text`、`table`、`json`、`yaml`
- `list` 支援 `text`、`table`、`json`、`yaml`
- `get` 支援 `text`、`table`、`json`、`yaml`
- `--api-mode` 支援 `auto`、`legacy`、`v1`、`v2`、`v1beta1`。
- Dashboard `auto` 依序嘗試 App Platform v2、v1、classic，且只有 404 才 fallback；folder 先試 v1 再走 classic。Datasource UID lookup 保留數字 ID legacy fallback。
- Notification 與 secure-value 是 public preview，必須明確指定 `--api-mode v1beta1`，不會由 `auto` 自動 probe。
- `--namespace` 指定 App Platform namespace，預設 `default`。
- Playlist 的 `auto` 與 `v1` 使用 `playlist.grafana.app/v1`，會跟隨
  continuation token 分頁並驗證資源身分。其他 API mode 會被拒絕；
  HTTP 錯誤（包含 404）會直接回報，不會改讀已棄用的 API。

## 建議模式
- Stable dashboard/folder inspection 優先用 `auto`；script 需要固定 envelope 時指定 `v1` 或 `v2`，classic-only 使用 `legacy`，preview resource 才使用 `v1beta1`。

## 範例
```bash
# 唯讀盤點輪播清單並查看完整設定。
grafana-util status resource list playlists --profile prod --namespace default --output-format table
grafana-util status resource get playlists/ops-rotation --profile prod --namespace default
grafana-util status resource describe playlists
```

```bash
# 說明目前支援的 live resource kinds 與 selector 格式。
grafana-util status resource describe
```

```bash
# 顯示目前支援的 resource kinds。
grafana-util status resource kinds
```

```bash
# 從本機 Grafana 列出 live dashboards。
grafana-util status resource list dashboards --url http://localhost:3000 --basic-user admin --basic-password admin
```

```bash
# 以 YAML 取得單一 datasource payload。
grafana-util status resource get datasources/prom-main --profile prod --output-format yaml
```

```bash
# 有需要時，強制 legacy 相容模式。
grafana-util status resource get datasources/10 --api-mode legacy
```

```bash
# 透過明確的 preview API 檢查 Grafana 13 notification receivers。
grafana-util status resource list receivers --api-mode v1beta1 --namespace default --profile prod --output-format json
```

`secure-values` 只提供唯讀 metadata inspection，沒有 export 或 mutation 路徑；即使異常 server 回傳 `spec.value`，也會先移除再交給 renderer。

## 相關指令
- [resource describe](./resource-describe.md)
- [resource kinds](./resource-kinds.md)
- [resource list](./resource-list.md)
- [resource get](./resource-get.md)
- [dashboard](./dashboard.md)
