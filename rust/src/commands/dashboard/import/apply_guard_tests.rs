use reqwest::Method;
use serde_json::{json, Value};
use std::fs;

use crate::common::{api_response, message, Result};
use crate::dashboard::dashboard_rust_tests::make_import_args;
use crate::dashboard::{ImportArgs, FOLDER_INVENTORY_FILENAME};

fn write_flat_raw_export(root: &std::path::Path, dashboard: Value) {
    fs::write(root.join("cpu.json"), dashboard.to_string()).unwrap();
    fs::write(
        root.join(FOLDER_INVENTORY_FILENAME),
        json!([{
            "uid": "infra",
            "title": "Infra",
            "path": "Platform / Infra",
            "parentUid": "platform",
            "org": "Main Org",
            "orgId": "1"
        }])
        .to_string(),
    )
    .unwrap();
}

#[allow(clippy::type_complexity)]
fn import_request<'a>(
    destination_folder_uid: &'a str,
    folder_exists: bool,
    writes: &'a mut Vec<Value>,
) -> impl FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>> + 'a {
    move |method, path, _, payload| match (method.as_str(), path) {
        ("GET", "/api/search") => Ok(Some(json!([{
            "uid": "cpu-main",
            "folderUid": destination_folder_uid
        }]))),
        ("GET", "/api/dashboards/uid/cpu-main") => Ok(Some(json!({
            "dashboard": {"id": 1, "uid": "cpu-main", "title": "Live CPU", "panels": []},
            "meta": {"folderUid": destination_folder_uid}
        }))),
        ("GET", "/api/folders/infra") if folder_exists => Ok(Some(json!({
            "uid": "infra",
            "title": "Infra",
            "parents": [{"uid": "platform", "title": "Platform"}]
        }))),
        ("GET", "/api/folders/infra") => Err(api_response(404, path, "folder not found")),
        ("POST", "/api/dashboards/db") => {
            writes.push(payload.unwrap().clone());
            Ok(Some(json!({"status": "success"})))
        }
        _ => Err(message(format!("Unexpected request: {method} {path}"))),
    }
}

fn folder_guard_args(root: &std::path::Path) -> ImportArgs {
    write_flat_raw_export(
        root,
        json!({
            "dashboard": {"uid": "cpu-main", "title": "Exported CPU", "schemaVersion": 38, "panels": []},
            "meta": {"folderUid": "infra"}
        }),
    );
    let mut args = make_import_args(root.to_path_buf());
    args.replace_existing = true;
    args.require_matching_folder_path = true;
    args
}

#[test]
fn flat_export_folder_guard_updates_matching_folder_in_preview_and_apply() {
    let temp = tempfile::tempdir().unwrap();
    let mut args = folder_guard_args(temp.path());
    let mut writes = Vec::new();
    let preview = super::dry_run::collect_import_dry_run_report_with_request(
        import_request("infra", true, &mut writes),
        &args,
    )
    .unwrap();
    assert_eq!(preview.dashboard_records[0][2], "update");
    assert_eq!(preview.dashboard_records[0][4], "Platform / Infra");
    assert!(writes.is_empty());

    args.dry_run = false;
    let imported = super::apply::import_dashboards_with_request(
        import_request("infra", true, &mut writes),
        &args,
    )
    .unwrap();
    assert_eq!(imported, 1);
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0]["folderUid"], "infra");
}

#[test]
fn flat_export_folder_guard_skips_general_target_in_preview_and_apply() {
    let temp = tempfile::tempdir().unwrap();
    let mut args = folder_guard_args(temp.path());
    let mut writes = Vec::new();
    let preview = super::dry_run::collect_import_dry_run_report_with_request(
        import_request("general", true, &mut writes),
        &args,
    )
    .unwrap();
    assert_eq!(preview.dashboard_records[0][2], "skip-folder-mismatch");
    assert!(writes.is_empty());

    args.dry_run = false;
    let imported = super::apply::import_dashboards_with_request(
        import_request("general", true, &mut writes),
        &args,
    )
    .unwrap();
    assert_eq!(imported, 0);
    assert!(writes.is_empty());
}

#[test]
fn folder_guard_skips_existing_target_with_unknown_folder_path() {
    let temp = tempfile::tempdir().unwrap();
    let mut args = folder_guard_args(temp.path());
    let mut writes = Vec::new();
    let preview = super::dry_run::collect_import_dry_run_report_with_request(
        import_request("infra", false, &mut writes),
        &args,
    )
    .unwrap();
    assert_eq!(preview.dashboard_records[0][2], "skip-folder-mismatch");
    assert!(preview.dashboard_records[0][6].starts_with("folder-path-unknown"));
    assert!(writes.is_empty());

    args.dry_run = false;
    let imported = super::apply::import_dashboards_with_request(
        import_request("infra", false, &mut writes),
        &args,
    )
    .unwrap();
    assert_eq!(imported, 0);
    assert!(writes.is_empty());
}

#[test]
fn update_existing_only_skips_dashboard_without_uid() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(
        temp.path().join("cpu.json"),
        json!({"title": "New CPU", "schemaVersion": 38, "panels": []}).to_string(),
    )
    .unwrap();
    let mut args = make_import_args(temp.path().to_path_buf());
    args.update_existing_only = true;
    let mut writes = Vec::new();
    let preview = super::dry_run::collect_import_dry_run_report_with_request(
        import_request("general", true, &mut writes),
        &args,
    )
    .unwrap();
    assert_eq!(preview.dashboard_records[0][2], "skip-missing");
    assert_eq!(preview.skipped_missing_count, 1);

    args.dry_run = false;
    let imported = super::apply::import_dashboards_with_request(
        import_request("general", true, &mut writes),
        &args,
    )
    .unwrap();
    assert_eq!(imported, 0);
    assert!(writes.is_empty());
}
