# dashboard import

## 用途
透過對應的 Grafana API 匯入 classic dashboard JSON 或原生 dashboard schema v2 resource JSON。

## 何時使用
當您手上有屬於 Grafana API 狀態管理的本地匯出樹，想把儀表板推回 Grafana，無論是實際執行或 dry run，都可以使用這個指令。Classic 匯入使用 `raw/` 或 `provisioning/` 輸入。原生 schema v2 匯入只有在明確傳入 `--input-format resource-v2` 時，才會讀取 `resource-v2/` JSON resource；不會讀取 Grafana UI 的 `prompt/` lane。

不要用這個指令繞過來源管理。如果目標 dashboard 是 Grafana Git Sync 或 file provisioning 管理，應該修改 Git repository、PR 或 provisioning source，再由該流程部署，而不是強制走 dashboard API 匯入。

## 採用前後對照
- **採用前**：匯入比較像盲目 replay，folder、org 或 schema 問題往往要打到 live 後才知道。
- **採用後**：匯入會先變成可 preview 的回放步驟，先用 `--dry-run` 看清楚，再決定是否真的動 live。

## 重點旗標
- `--input-dir`：classic lane、匯出的 `resource-v2/` 目錄或合併匯出根目錄。
- `--input-format`：選擇 `raw`、`provisioning`，或明確指定原生 resource lane `resource-v2`。
- `--resource-namespace`：`resource-v2` 匯入使用的 App Platform namespace，預設為 `default`；實際選定的值具有最終權威，會取代來源文件中的 namespace。
- `--local`、`--run`、`--run-id`：不傳 `--input-dir`，改從 artifact workspace 的 dashboard lane 讀取來源。
- `--org-id`、`--use-export-org`、`--only-org-id`、`--create-missing-orgs`：控制跨 org 路由。
- `--import-folder-uid`：強制指定目的資料夾 UID。
- `--ensure-folders`、`--replace-existing`、`--update-existing-only`：控制匯入行為。
- `--require-matching-folder-path`、`--require-matching-export-org`、`--strict-schema`、`--target-schema-version`：安全檢查。
- `--import-message`：儲存在 Grafana 的修訂訊息。
- `--interactive`、`--dry-run`、`--table`、`--json`、`--output-format`、`--output-columns`、`--list-columns`、`--no-header`、`--progress`、`--verbose`：預覽與回報控制。若想看完整 dry-run 表格欄位，可用 `--output-columns all`。

## 成功判準
- dry-run 先把 create/update 動作列清楚，再進入 live replay
- dry-run 也會顯示目標 UID 的 live dashboard ownership evidence，包含 provisioned 或 managed-state 警告
- 目的 org 與 folder 路由足夠明確，可以先 review
- 這次匯入使用的是正確的輸入 lane：`raw`、`provisioning` 或明確指定的 `resource-v2`，不是 `prompt`
- Git Sync 或 file-provisioned targets 會被視為 source-owned，應回到 repository 或 provisioning workflow 處理

## 原生 resource-v2 安全邊界

`--input-format resource-v2` 只接受 JSON resource，且必須包含 `apiVersion: dashboard.grafana.app/v2`、`kind: Dashboard`、穩定且非空的 `metadata.name`，以及 object 型態的 `spec`。此 lane 會保留 v2 schema，不會轉換成 classic dashboard JSON。

此 lane 只會呼叫 `/apis/dashboard.grafana.app/v2/namespaces/<namespace>/dashboards`，絕不退回 legacy `/api/dashboards/db` endpoint。v2 API 回傳 `404` 代表伺服器不支援這個匯入 lane；驗證或授權失敗仍會直接回報錯誤。

Resource-v2 匯入僅支援單一 namespace，不能搭配以下 classic 路由與驗證旗標：`--org-id`、`--use-export-org`、`--only-org-id`、`--create-missing-orgs`、`--require-matching-export-org`、`--require-matching-folder-path`、`--ensure-folders`、`--strict-schema`、`--target-schema-version`。

先執行 `--dry-run`，在不送出 mutation 的情況下檢查 create、update、no-change、skip 或 blocked 動作。不存在的 resource 可以建立；已存在的 resource 只有在傳入 `--replace-existing` 時才會更新；`--update-existing-only` 會跳過不存在的 resource。若 Grafana 回報 Git Sync 或 file provisioning ownership，直接更新仍會被阻擋，也沒有繞過 ownership 的旗標。Resource 匯入不會刪除、prune 或 rollback dashboard。

Resource-v2 dry-run 的 text、table 與 JSON 都會顯示來源路徑、API version、namespace、resource name、folder、ownership、blocker 與 changed fields。Table 的 `--output-columns` 可使用 `api_version`、`namespace`、`name`、`source_path`、`action`、`folder`、`ownership`、`blocker` 與 `changed_fields`。使用具 TUI 功能的 build 執行 `--interactive` 時，可搜尋並捲動檢查相同證據；只有明確確認後才會寫入 create/update resource。Interactive mode 需要 stdin 與 stdout 都是 TTY。

JSON dry-run 會輸出單一版本化文件，包含 `kind`、`mode`、`dashboards` 與 action `summary`。Combined-root index path 與 standalone discovery 都不得離開選定的 input root，並會拒絕 symbolic link。比較時會正規化 Grafana 自動補入的 v2 schema default，因此相同來源重跑會回報 `no_change`，不會誤判為 update。

Resource-v2 live apply 會依序執行，第一個 write 失敗時立即停止。錯誤會列出 completed、failed、not-attempted resource identity，保留原始 HTTP status，並明確說明未嘗試 rollback。

## 失敗時先檢查
- 如果 folder 或 org 落點不對，先檢查路由旗標，不要直接重跑 live import
- 如果看起來會刪或覆蓋太多，先停在 `--dry-run` 並回頭檢查匯出樹
- 如果 Grafana 回報目標 UID 是 provisioned 或 Git Sync-managed dashboard，不要改用直接 import 重試；請更新它的來源並沿原本 lane 重新部署
- 如果 schema 被擋下來，先確認來源資料是不是需要先正規化再匯入
- 如果 resource-v2 回報 API 不可用，請確認 Grafana 有提供 v2 `/apis` endpoint；不要改用 classic 匯入重試同一份 resource
- 如果 resource-v2 選到錯誤 namespace，請明確傳入 `--resource-namespace` 並重新檢查 dry-run；來源的 `metadata.namespace` 不具有最終權威

## 範例
```bash
# 用 Grafana API 匯入儀表板 JSON 檔案。
grafana-util dashboard import --profile prod --input-dir ./dashboards/raw --replace-existing
```

```bash
# 用 Grafana API 匯入儀表板 JSON 檔案。
grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw --dry-run --table
```

```bash
# 用 Grafana API 匯入儀表板 JSON 檔案。
grafana-util dashboard import --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --input-dir ./dashboards/raw --dry-run --table
```

```bash
# 從最新的 profile artifact workspace dashboard run 做 dry-run import。
grafana-util dashboard import --profile prod --local --dry-run --table
```

```bash
# 在具有最終權威的 default namespace 預覽原生 schema v2 resource。
grafana-util dashboard import --profile prod --input-dir ./dashboards/resource-v2 --input-format resource-v2 --resource-namespace default --dry-run --table
```

```bash
# 透過原生 /apis v2 endpoint 執行建立與允許的更新。
grafana-util dashboard import --profile prod --input-dir ./dashboards/resource-v2 --input-format resource-v2 --resource-namespace default --replace-existing
```

## 相關指令
- [dashboard export](./dashboard-export.md)
- [dashboard convert raw-to-prompt](./dashboard-convert-raw-to-prompt.md)
- [dashboard diff](./dashboard-diff.md)
- [dashboard publish](./dashboard-publish.md)
