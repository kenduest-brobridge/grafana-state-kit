# `grafana-util alert apply`

## 目的

套用一份已審閱過的 alert 管理計畫。

## 使用時機

- 執行已審閱、且只包含窄範圍 alert-rule update row 的計畫。
- 在碰觸 Grafana 之前要求明確確認。
- 讓較廣的 alerting 變更先停在 staged/review 狀態，直到 mutation 語意足夠明確。

## 安全邊界

`alert apply` 是 controlled live-apply 路徑。目前只允許 `ready` 的 alert-rule `update` row，且 changed fields 僅限 `condition`、`data`、`for`、`noDataState` 或 `execErrState`。

Create、delete、contact-point、mute-timing、policy、template，以及語意較不明確的 rule field 變更都會 fail closed。請用 `alert plan` 和 review output 保留這些較廣變更的可見性，不要直接 apply。

第一次寫入前會驗證所有預計執行的變更與實際更新內容。允許欄位檢查會比較計畫中的 live 與 desired 規則值，不會只信任 `changedFields`。
計畫文件必須使用 `schemaVersion: 1`。

第一次寫入前，apply 會重新讀取所有待更新的 rule，並使用與 `alert plan` 相同的正規化比較方式，核對已審閱的 `live` 基線。
只要任何 rule 已變更、已不存在，或讀取失敗，整份計畫就會停止，且不會寫入任何更新。這項檢查不會鎖住 Grafana；讀取完成後仍可能有其他人修改 rule。

更新請求失敗時，apply 會停止後續寫入，並以狀態碼 1 結束。先前已完成的更新會保留。
文字與 JSON 輸出會逐筆標示 `completed`（已完成）、`failed`（請求失敗）或 `not-attempted`（尚未送出）；`appliedCount` 只計算已完成的更新。
請求失敗不代表該 rule 的 live 狀態已確定，重試前請先核對。Apply 不會自動重試或回復先前的更新。

使用 `--output-format json` 時，結果會先寫入 stdout，再將失敗摘要寫入 stderr。
結果的 `status` 為 `completed`、`partial` 或 `failed`，另包含 `failedCount`、`notAttemptedCount` 與逐筆的 `results[].status`。

## 採用前後對照

- **採用前**：即使 alert plan 已審閱完成，真正碰到 live Grafana 的最後一步仍然很容易變成不透明的人工操作。
- **採用後**：`alert apply` 會把最後一步變成明確命令，保留核准、可重現驗證與可機器判讀的輸出。

## 主要旗標

- `--plan-file` 指向已審閱的計畫文件。
- `--approve` 是允許執行前的必要確認。
- `--output-format` 可將套用輸出呈現為 `text` 或 `json`。

## 範例

```bash
# 套用一份已審閱過的 alert 管理計畫。
grafana-util alert apply --profile prod --plan-file ./alert-plan-reviewed.json --approve
```

```bash
# 套用一份已審閱過的 alert 管理計畫。
grafana-util alert apply --url http://localhost:3000 --basic-user admin --basic-password admin --plan-file ./alert-plan-reviewed.json --approve
```

```bash
# 套用一份已審閱過的 alert 管理計畫。
grafana-util alert apply --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --plan-file ./alert-plan-reviewed.json --approve
```

## 成功判準

- 窄範圍且已審閱的 alert-rule update 能直接套用，不必再手改 YAML 或重做一串 UI 操作
- live apply 步驟會保留明確的核准，不會只藏在 shell history 裡
- JSON 輸出穩定到足以交給 CI、變更紀錄或套用後驗證流程

## 失敗時先檢查

- 如果 apply 不肯執行，先確認你給的是 reviewed plan，而且有帶 `--approve`
- 如果 live 結果和預期 plan 差很多，先核對認證、org scope，以及 reviewed plan 是否仍對應目前目標 Grafana
- 如果基線檢查或更新失敗，先核對目前的 live rule、重建計畫，再審閱剩餘變更後重新套用
- 如果自動化要吃輸出，建議用 `--output-format json`，並先驗證結果 shape 再判定成功

## 相關命令

- [alert](./alert.md)
- [alert plan](./alert-plan.md)
- [alert delete](./alert-delete.md)
