# `grafana-util status`

## Root

說明：用單一 status surface 讀取 live 與 staged 的 Grafana 狀態。

適用時機：當你想看 readiness、overview、snapshot，或直接讀 live 資料，但還不想進 mutation 流程時。

說明：`status` 是給使用者看的唯讀入口。`live` 用來做即時 gate，`staged` 用來看本機 artifact，`overview` 用來看全域摘要，`snapshot` 用來看 bundle 風格的 review，`resource` 則用來直接讀 live resource。

範例：

```bash
# 執行這個範例指令。
grafana-util status live --profile prod --output-format yaml
# 執行這個範例指令。
grafana-util status staged --desired-file ./desired.json --output-format json
# 執行這個範例指令。
grafana-util status overview --dashboard-export-dir ./dashboards/raw --alert-export-dir ./alerts --output-format table
# 執行這個範例指令。
grafana-util status overview live --url http://localhost:3000 --basic-user admin --basic-password admin --output-format interactive
```

相關指令：`grafana-util export`、`grafana-util workspace`、`grafana-util config profile`。

## `live`

說明：從 Grafana live read surface 產生 readiness 視圖。

JSON/YAML 輸出會包含共用 project status contract：`kind`、`schemaVersion`、
`toolVersion`、`scope`、`overall`、`domains`、`topBlockers`、`nextActions`。
live 輸出也會從 Grafana `GET /api/health` 補上 `discovery.instance`。
成功時會有 `source: api-health`、`status: available`，以及 `database`、
`version`、`commit` 等 health 欄位。若 health 讀取失敗，
`discovery.instance.status` 會是 `unavailable` 並帶 `error`；這不會單獨把
domain readiness 判成 blocked。

成功時的 instance metadata：

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

health 讀取失敗時：

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

automation 如果需要 Grafana build 資訊，讀
`discovery.instance.health.version` 與 `discovery.instance.health.commit`。
readiness gate 仍應讀 `overall` 與 `domains`。

## `capabilities`

用途：在選擇 dashboard 工作流程前，確認目前憑證及指定 Grafana namespace
可讀取哪些 dashboard 清單 API。

```bash
# 使用 prod 設定檔檢查 dashboard API 是否可讀取。
grafana-util status capabilities --profile prod --namespace default --output-format json
```

此命令只送出三次 GET，各帶 `limit=1`：Classic
`/api/search?type=dash-db&page=1`，以及 `/apis/dashboard.grafana.app/` 下的
`v1`、`v2` dashboard collection。不翻頁、不修改 Grafana。
必須明確指定 `--namespace`；請填入該 Grafana 實例使用的 namespace。
`--org-id` 另行限定組織，不會替你選擇 namespace。

JSON/YAML 包含 `kind: grafana-dashboard-api-capabilities`、`schemaVersion: 1`、
`namespace` 及 `probes`。每筆 probe 包含 `api`、`path`、`status`、`detail`；
收到非成功 HTTP 回應時另有 `httpStatus`。
有效的空清單也算 `available`，但只代表清單讀取成功，不代表有寫入權限、
清單完整，或某份 dashboard 文件一定可匯入。結果只記錄本次觀察，不快取。

| 狀態 | 意義 |
| --- | --- |
| `available` | 回應符合預期清單格式。 |
| `unauthorized` | HTTP 401；修正認證後重試。 |
| `forbidden` | HTTP 403；確認讀取權限後重試。 |
| `not-served` | HTTP 404；指定路由或 namespace 找不到。 |
| `failed` | 其他 HTTP 錯誤（含 5xx）或傳輸失敗。 |
| `malformed-response` | JSON 無效、無本文，或清單格式不符。 |

401/403、404、失敗及格式不符，都不能判定整個實例是否支援該 API。
尤其 404 可能來自指定 namespace 或代理路由。命令會輸出全部 probe 結果；
即使某次 probe 失敗，命令仍成功結束，automation 應檢查各筆 `status`。
本機參數無效或連線設定失敗仍會回報命令錯誤。
報告不包含憑證、回應本文或傳輸錯誤的細節。

路由及清單語意參照官方
[Dashboard API](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/dashboard/)
與 [Classic search API](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/api-legacy/folder_dashboard_search/)。
