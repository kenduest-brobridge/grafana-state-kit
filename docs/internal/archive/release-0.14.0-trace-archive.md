# Release preparation history

## 2026-07-12 - Grafana 13 App Platform integration
- State: Done
- Scope: shared App Platform client; dashboard v2 export/history; status resource v1/v2/v1beta1; notification/secret safety; Grafana 13.1 Git Sync discovery
- Current Update: Added 404-only discovery and pagination, native v2 resource lanes, stable resource history, explicit preview inspection, secret redaction, and root-level Git Sync CRD discovery.
- Result: Classic behavior stays default while Grafana 13 resources are available through explicit, tested read-only/export lanes.

## 2026-06-11 - Datasource browse TUI chrome split
- Summary: Moved datasource browse footer controls and search prompt rendering into a dedicated TUI chrome module while keeping the main renderer focused on frame, list, and detail layout.
- Tests: cargo test --quiet datasource_browse --features tui; make quality-rust; make quality-architecture; cargo check --quiet --no-default-features --all-targets; python3 scripts/tui_inventory_report.py --json; make quality-ai-workflow
- Impact: `datasource/browse/render.rs` is now below the architecture warning threshold, reducing the warning list from four files to three without changing public CLI paths, help text, generated docs, or interactive behavior.
- Rollback/Risk: Low; this is a structural move of existing footer/search rendering covered by focused datasource browse render tests.
- Follow-up: Continue reducing the remaining TUI/shared warning-threshold files: review_contract and shared browser session.
