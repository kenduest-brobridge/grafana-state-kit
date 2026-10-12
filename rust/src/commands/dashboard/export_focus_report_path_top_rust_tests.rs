//! Dashboard domain test suite.
//! Covers parser surfaces, formatter/output contracts, and export/import/inspect/list/diff
//! behavior with in-memory/mocked request fixtures.
#![allow(unused_imports)]

use super::test_support;
use super::{export_dashboards_with_request, make_common_args, ExportArgs};
use serde_json::{json, Value};
use std::cell::Cell;
use std::fs;
use tempfile::tempdir;

fn make_history_only_export_args(
    output_dir: std::path::PathBuf,
    org_id: Option<i64>,
    all_orgs: bool,
    overwrite: bool,
) -> ExportArgs {
    ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        output_dir,
        run: None,
        run_id: None,
        page_size: 500,
        org_id,
        all_orgs,
        flat: false,
        overwrite,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        without_dashboard_provisioning: true,
        include_history: true,
        native: false,
        portable_v2: false,
        resource_format: crate::dashboard::DashboardResourceFormat::None,
        resource_namespace: None,
        provisioning_provider_name: "grafana-utils-dashboards".to_string(),
        provisioning_provider_org_id: None,
        provisioning_provider_path: None,
        provisioning_provider_disable_deletion: false,
        provisioning_provider_allow_ui_updates: false,
        provisioning_provider_update_interval_seconds: 30,
        dry_run: false,
        progress: false,
        verbose: false,
    }
}

#[test]
fn export_dashboards_with_request_all_orgs_aggregates_results() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        output_dir: temp.path().join("dashboards"),
        run: None,
        run_id: None,
        page_size: 500,
        org_id: None,
        all_orgs: true,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        without_dashboard_provisioning: true,
        include_history: false,
        native: false,
        portable_v2: false,
        resource_format: crate::dashboard::DashboardResourceFormat::None,
        resource_namespace: None,
        provisioning_provider_name: "grafana-utils-dashboards".to_string(),
        provisioning_provider_org_id: None,
        provisioning_provider_path: None,
        provisioning_provider_disable_deletion: false,
        provisioning_provider_allow_ui_updates: false,
        provisioning_provider_update_interval_seconds: 30,
        dry_run: false,
        progress: false,
        verbose: false,
    };
    let mut calls = Vec::new();

    let count = export_dashboards_with_request(
        |method, path, params, payload| {
            calls.push((
                method.to_string(),
                path.to_string(),
                params.to_vec(),
                payload.cloned(),
            ));
            let scoped_org = params
                .iter()
                .find(|(key, _)| key == "orgId")
                .map(|(_, value)| value.as_str());
            match (path, scoped_org) {
                ("/api/orgs", None) => Ok(Some(json!([
                    {"id": 1, "name": "Main Org"},
                    {"id": 2, "name": "Ops Org"}
                ]))),
                ("/api/org", Some("1")) => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
                ("/api/org", Some("2")) => Ok(Some(json!({"id": 2, "name": "Ops Org"}))),
                ("/api/search", Some("1")) => Ok(Some(
                    json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }]),
                )),
                ("/api/datasources", Some("1")) => Ok(Some(json!([
                    {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
                ]))),
                ("/api/search", Some("2")) => Ok(Some(
                    json!([{ "uid": "xyz", "title": "Logs", "folderTitle": "Ops" }]),
                )),
                ("/api/datasources", Some("2")) => Ok(Some(json!([
                    {"uid": "logs-main", "name": "Logs Main", "type": "loki", "url": "http://loki:3100", "access": "proxy", "isDefault": false}
                ]))),
                ("/api/dashboards/uid/abc", Some("1")) => Ok(Some(
                    json!({"dashboard": {"id": 7, "uid": "abc", "title": "CPU", "panels": [
                        {"datasource": {"uid": "prom-main", "type": "prometheus"}}
                    ]}}),
                )),
                ("/api/dashboards/uid/xyz", Some("2")) => Ok(Some(
                    json!({"dashboard": {"id": 8, "uid": "xyz", "title": "Logs", "panels": [
                        {"datasource": {"uid": "logs-main", "type": "loki"}}
                    ]}}),
                )),
                ("/api/dashboards/uid/abc/permissions", Some("1")) => Ok(Some(json!([
                    {"userId": 11, "userLogin": "ops", "permission": 4}
                ]))),
                ("/api/dashboards/uid/xyz/permissions", Some("2")) => Ok(Some(json!([
                    {"teamId": 21, "team": "SRE", "permission": 2}
                ]))),
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert!(args
        .output_dir
        .join("org_1_Main_Org/raw/Infra/abc.json")
        .is_file());
    assert!(args
        .output_dir
        .join("org_1_Main_Org/raw/index.json")
        .is_file());
    assert!(args
        .output_dir
        .join("org_1_Main_Org/raw/export-metadata.json")
        .is_file());
    assert!(args
        .output_dir
        .join("org_1_Main_Org/raw/folders.json")
        .is_file());
    assert!(args
        .output_dir
        .join("org_1_Main_Org/raw/datasources.json")
        .is_file());
    assert!(args
        .output_dir
        .join("org_1_Main_Org/raw/permissions.json")
        .is_file());
    assert!(args
        .output_dir
        .join("org_2_Ops_Org/raw/Ops/xyz.json")
        .is_file());
    assert!(args
        .output_dir
        .join("org_2_Ops_Org/raw/index.json")
        .is_file());
    assert!(args
        .output_dir
        .join("org_2_Ops_Org/raw/permissions.json")
        .is_file());
    let aggregate_root_index: Value =
        serde_json::from_str(&fs::read_to_string(args.output_dir.join("index.json")).unwrap())
            .unwrap();
    let aggregate_root_metadata: Value = serde_json::from_str(
        &fs::read_to_string(args.output_dir.join("export-metadata.json")).unwrap(),
    )
    .unwrap();
    let org_one_metadata: Value = serde_json::from_str(
        &fs::read_to_string(
            args.output_dir
                .join("org_1_Main_Org/raw/export-metadata.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let org_two_metadata: Value = serde_json::from_str(
        &fs::read_to_string(
            args.output_dir
                .join("org_2_Ops_Org/raw/export-metadata.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        org_one_metadata["org"],
        Value::String("Main Org".to_string())
    );
    assert_eq!(org_one_metadata["orgId"], Value::String("1".to_string()));
    assert_eq!(
        org_one_metadata["permissionsFile"],
        Value::String("permissions.json".to_string())
    );
    assert_eq!(
        org_two_metadata["org"],
        Value::String("Ops Org".to_string())
    );
    assert_eq!(org_two_metadata["orgId"], Value::String("2".to_string()));
    assert_eq!(
        org_two_metadata["permissionsFile"],
        Value::String("permissions.json".to_string())
    );
    assert_eq!(aggregate_root_index["items"].as_array().unwrap().len(), 2);
    assert!(aggregate_root_index["variants"]["raw"].is_null());
    assert!(aggregate_root_index["variants"]["provisioning"].is_null());
    assert_eq!(
        aggregate_root_index["items"][0]["raw_path"],
        Value::String(
            args.output_dir
                .join("org_1_Main_Org/raw/Infra/abc.json")
                .display()
                .to_string()
        )
    );
    assert_eq!(
        aggregate_root_index["items"][1]["raw_path"],
        Value::String(
            args.output_dir
                .join("org_2_Ops_Org/raw/Ops/xyz.json")
                .display()
                .to_string()
        )
    );
    assert_eq!(
        aggregate_root_metadata["variant"],
        Value::String("root".to_string())
    );
    assert_eq!(
        aggregate_root_metadata["scopeKind"],
        Value::String("all-orgs-root".to_string())
    );
    assert_eq!(
        aggregate_root_metadata["source"]["orgScope"],
        Value::String("all-orgs".to_string())
    );
    assert!(aggregate_root_metadata.get("org").is_none());
    assert!(aggregate_root_metadata.get("orgId").is_none());
    assert_eq!(
        aggregate_root_metadata["indexFile"],
        Value::String("index.json".to_string())
    );
    assert_eq!(aggregate_root_metadata["orgCount"], Value::Number(2.into()));
    assert_eq!(aggregate_root_metadata["orgs"].as_array().unwrap().len(), 2);
    let org_entries = aggregate_root_metadata["orgs"].as_array().unwrap();
    let org_one_entry = org_entries
        .iter()
        .find(|entry| entry["orgId"] == Value::String("1".to_string()))
        .unwrap();
    let org_two_entry = org_entries
        .iter()
        .find(|entry| entry["orgId"] == Value::String("2".to_string()))
        .unwrap();
    assert_eq!(
        org_one_entry["usedDatasourceCount"],
        Value::Number(1.into())
    );
    assert_eq!(
        org_one_entry["exportDir"],
        Value::String(args.output_dir.join("org_1_Main_Org").display().to_string())
    );
    assert_eq!(
        org_one_entry["usedDatasources"][0]["uid"],
        Value::String("prom-main".to_string())
    );
    assert_eq!(
        org_two_entry["usedDatasourceCount"],
        Value::Number(1.into())
    );
    assert_eq!(
        org_two_entry["exportDir"],
        Value::String(args.output_dir.join("org_2_Ops_Org").display().to_string())
    );
    assert_eq!(
        org_two_entry["usedDatasources"][0]["uid"],
        Value::String("logs-main".to_string())
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, _, _)| path == "/api/orgs")
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params, _)| path == "/api/search"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "1"))
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|(_, path, params, _)| path == "/api/search"
                && params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "2"))
            .count(),
        1
    );
}

#[test]
fn export_dashboards_writes_resource_v1_lane_without_classic_variants() {
    let temp = tempdir().unwrap();
    let mut args = make_history_only_export_args(temp.path().join("dashboards"), None, false, true);
    args.include_history = false;
    args.without_dashboard_raw = true;
    args.without_dashboard_prompt = true;
    args.without_dashboard_provisioning = true;
    args.resource_format = crate::dashboard::DashboardResourceFormat::V1;
    args.resource_namespace = Some("default".to_string());

    let count = export_dashboards_with_request(
        |method, path, params, _payload| match (method.as_str(), path) {
            ("GET", "/api/org") => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
            (
                "GET",
                "/apis/dashboard.grafana.app/v1/namespaces/default/dashboards",
            ) => {
                assert_eq!(params, &[("limit".to_string(), "500".to_string())]);
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v1",
                    "kind": "DashboardList",
                    "items": [{
                        "kind": "Dashboard",
                        "apiVersion": "dashboard.grafana.app/v1",
                        "metadata": {
                            "name": "cpu-main",
                            "namespace": "default",
                            "uid": "internal-resource-id",
                            "annotations": {"grafana.app/folder": "infra"}
                        },
                        "spec": {"title": "CPU Main", "panels": []},
                        "status": {}
                    }],
                    "metadata": {}
                })))
            }
            ("GET", "/api/datasources") => Ok(Some(json!([
                {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
            ]))),
            ("GET", "/api/search") => Ok(Some(json!([
                {"uid": "cpu-main", "title": "CPU Main", "folderTitle": "Infra", "folderUid": "infra"}
            ]))),
            ("GET", "/api/folders/infra") => Ok(Some(json!({
                "uid": "infra",
                "title": "Infra",
                "parents": [{"uid": "platform", "title": "Platform"}]
            }))),
            ("GET", "/api/dashboards/uid/cpu-main") => Ok(Some(json!({
                "dashboard": {
                    "uid": "cpu-main",
                    "title": "CPU Main",
                    "panels": [{"datasource": {"uid": "prom-main", "type": "prometheus"}}]
                }
            }))),
            (
                "GET",
                "/apis/dashboard.grafana.app/v1/namespaces/default/dashboards/cpu-main",
            ) => {
                assert!(params.is_empty());
                Ok(Some(json!({
                    "kind": "Dashboard",
                    "apiVersion": "dashboard.grafana.app/v1",
                    "metadata": {
                        "name": "cpu-main",
                        "namespace": "default",
                        "uid": "internal-resource-id",
                        "annotations": {"grafana.app/folder": "infra"}
                    },
                    "spec": {
                        "title": "CPU Main",
                        "panels": []
                    },
                    "status": {}
                })))
            }
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    let resource_path = args.output_dir.join("resource-v1/objects/cpu-main.json");
    assert!(resource_path.is_file());
    assert!(!args.output_dir.join("raw").exists());
    assert!(!args.output_dir.join("classic/raw").exists());
    assert!(args.output_dir.join("resource-v1/index.json").is_file());
    assert!(args
        .output_dir
        .join("resource-v1/export-metadata.json")
        .is_file());

    let resource: Value =
        serde_json::from_str(&fs::read_to_string(&resource_path).unwrap()).unwrap();
    assert_eq!(resource["kind"], Value::String("Dashboard".to_string()));
    assert_eq!(
        resource["metadata"]["name"],
        Value::String("cpu-main".to_string())
    );

    let lane_index: Value = serde_json::from_str(
        &fs::read_to_string(args.output_dir.join("resource-v1/index.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(lane_index.as_array().unwrap().len(), 1);
    assert_eq!(
        lane_index[0]["path"],
        Value::String(resource_path.display().to_string())
    );
    assert_eq!(
        lane_index[0]["apiVersion"],
        Value::String("dashboard.grafana.app/v1".to_string())
    );

    let lane_metadata: Value = serde_json::from_str(
        &fs::read_to_string(args.output_dir.join("resource-v1/export-metadata.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        lane_metadata["variant"],
        Value::String("resource-v1".to_string())
    );
    assert_eq!(
        lane_metadata["format"],
        Value::String("grafana-dashboard-resource-v1".to_string())
    );
    assert_eq!(
        lane_metadata["resourceApiVersion"],
        Value::String("dashboard.grafana.app/v1".to_string())
    );
    assert_eq!(lane_metadata["uiImportCompatible"], Value::Bool(false));

    let root_index: Value =
        serde_json::from_str(&fs::read_to_string(args.output_dir.join("index.json")).unwrap())
            .unwrap();
    assert!(root_index["variants"]["raw"].is_null());
    assert_eq!(
        root_index["variants"]["resource-v1"],
        Value::String(
            args.output_dir
                .join("resource-v1/index.json")
                .display()
                .to_string()
        )
    );
    assert_eq!(
        root_index["items"][0]["resource_v1_path"],
        Value::String(resource_path.display().to_string())
    );
}

#[test]
fn export_dashboards_writes_resource_v2_without_rewriting_elements() {
    let temp = tempdir().unwrap();
    let mut args = make_history_only_export_args(temp.path().join("dashboards"), None, false, true);
    args.include_history = false;
    args.without_dashboard_raw = true;
    args.without_dashboard_prompt = true;
    args.without_dashboard_provisioning = true;
    args.resource_format = crate::dashboard::DashboardResourceFormat::V2;
    args.resource_namespace = Some("default".to_string());

    let count = export_dashboards_with_request(
        |method, path, params, _payload| match (method.as_str(), path) {
            ("GET", "/api/org") => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
            ("GET", "/apis/dashboard.grafana.app/v2/namespaces/default/dashboards") => {
                assert_eq!(params, &[("limit".to_string(), "500".to_string())]);
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "DashboardList",
                    "items": [{
                        "kind": "Dashboard",
                        "apiVersion": "dashboard.grafana.app/v2",
                        "metadata": {"name": "cpu-main", "namespace": "default"},
                        "spec": {
                            "title": "CPU Main",
                            "elements": {
                                "panel-1": {"kind": "Panel", "spec": {"title": "CPU"}}
                            }
                        }
                    }],
                    "metadata": {}
                })))
            }
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    let resource_path = args.output_dir.join("resource-v2/objects/cpu-main.json");
    let resource: Value =
        serde_json::from_str(&fs::read_to_string(&resource_path).unwrap()).unwrap();
    assert_eq!(resource["apiVersion"], "dashboard.grafana.app/v2");
    assert_eq!(resource["spec"]["elements"]["panel-1"]["kind"], "Panel");
    assert!(resource["spec"].get("panels").is_none());

    let root_index: Value =
        serde_json::from_str(&fs::read_to_string(args.output_dir.join("index.json")).unwrap())
            .unwrap();
    assert_eq!(
        root_index["variants"]["resource-v2"],
        Value::String(
            args.output_dir
                .join("resource-v2/index.json")
                .display()
                .to_string()
        )
    );
    assert_eq!(
        root_index["items"][0]["resource_v2_path"],
        Value::String(resource_path.display().to_string())
    );
}

#[test]
fn portable_v2_export_round_trip_prepares_destination_create_and_update_requests() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/dashboard_resource_v2_import_golden.json"
    ))
    .unwrap();
    let source = &fixture["source"];
    for portable_v2 in [false, true] {
        let temp = tempdir().unwrap();
        let mut args =
            make_history_only_export_args(temp.path().join("dashboards"), None, false, true);
        args.include_history = false;
        args.without_dashboard_raw = true;
        args.resource_format = crate::dashboard::DashboardResourceFormat::V2;
        args.resource_namespace = Some("source-namespace".to_string());
        args.portable_v2 = portable_v2;
        assert_eq!(export_dashboards_with_request(|method, path, _, _| {
            assert_eq!(method, reqwest::Method::GET);
            assert_eq!(path, "/apis/dashboard.grafana.app/v2/namespaces/source-namespace/dashboards");
            Ok(Some(json!({"apiVersion":"dashboard.grafana.app/v2","kind":"DashboardList","metadata":{},"items":[source.clone()]})))
        }, &args).unwrap(), 1);
        let artifact = args
            .output_dir
            .join("resource-v2/objects/cpu-overview.json");
        let artifact_text = fs::read_to_string(&artifact).unwrap();
        let exported: Value = serde_json::from_str(&artifact_text).unwrap();
        if !portable_v2 {
            assert_eq!(exported, *source);
            assert_eq!(
                artifact_text,
                serde_json::to_string_pretty(source).unwrap() + "\n"
            );
        }
        assert_eq!(exported["spec"], source["spec"]);
        assert_eq!(exported["metadata"]["namespace"], "source-namespace");
        assert_eq!(
            exported["metadata"]["annotations"]["grafana.app/folder"],
            "source-folder"
        );
        assert_eq!(
            exported["metadata"].get("resourceVersion").is_none(),
            portable_v2
        );
        for update in [false, true] {
            let import_args = match crate::dashboard::parse_cli_from([
                "grafana-util",
                "import",
                "--input-dir",
                args.output_dir.to_str().unwrap(),
                "--input-format",
                "resource-v2",
                "--resource-namespace",
                "target-namespace",
                "--replace-existing",
            ])
            .command
            {
                crate::dashboard::DashboardCommand::Import(args) => args,
                _ => unreachable!(),
            };
            let live = json!({"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard", "metadata":{"name":"cpu-overview","namespace":"target-namespace","resourceVersion":"target-version"},"spec":{"title":"Old"}});
            let mut writes = Vec::new();
            assert_eq!(crate::dashboard::import::import_dashboards_with_request(|method, path, _, payload| {
                if method == reqwest::Method::GET && path.ends_with("/dashboards") {
                    return Ok(Some(json!({"apiVersion":"dashboard.grafana.app/v2","kind":"DashboardList","metadata":{},"items":[]})));
                }
                if method == reqwest::Method::GET {
                    if update { return Ok(Some(live.clone())); }
                    return Err(crate::common::api_response(404, path.to_string(), "not found".to_string()));
                }
                writes.push((method.clone(), path.to_string(), payload.unwrap().clone()));
                Ok(payload.cloned())
            }, &import_args).unwrap(), 1);
            assert_eq!(writes.len(), 1);
            let (method, path, payload) = &writes[0];
            assert_eq!(
                *method,
                if update {
                    reqwest::Method::PUT
                } else {
                    reqwest::Method::POST
                }
            );
            assert_eq!(
                path,
                &format!(
                    "/apis/dashboard.grafana.app/v2/namespaces/target-namespace/dashboards{}",
                    if update { "/cpu-overview" } else { "" }
                )
            );
            assert_eq!(payload["spec"], source["spec"]);
            assert_eq!(payload["metadata"]["namespace"], "target-namespace");
            assert_eq!(
                payload["metadata"]["annotations"]["grafana.app/folder"],
                "source-folder"
            );
            assert!(payload["metadata"].get("uid").is_none());
            assert!(payload["metadata"].get("generation").is_none());
            assert!(payload["metadata"]["annotations"]
                .get("grafana.app/createdBy")
                .is_none());
            if update {
                assert_eq!(payload["metadata"]["resourceVersion"], "target-version");
            } else {
                assert!(payload["metadata"].get("resourceVersion").is_none());
            }
        }
    }
}

#[test]
fn portable_v2_export_rejects_incompatible_selection_before_network() {
    let temp = tempdir().unwrap();
    for format in [
        crate::dashboard::DashboardResourceFormat::None,
        crate::dashboard::DashboardResourceFormat::V1,
    ] {
        let mut args =
            make_history_only_export_args(temp.path().join("dashboards"), None, true, true);
        args.portable_v2 = true;
        args.resource_format = format;
        let error = export_dashboards_with_request(
            |_, _, _, _| panic!("invalid portable selection must not query Grafana"),
            &args,
        )
        .unwrap_err();
        assert!(error.to_string().contains("--portable-v2 requires"));
        assert!(!args.output_dir.exists());
    }
}

#[test]
fn portable_v2_export_all_requires_served_v2_before_writing_v1() {
    let temp = tempdir().unwrap();
    let mut args = make_history_only_export_args(temp.path().join("dashboards"), None, false, true);
    args.include_history = false;
    args.without_dashboard_raw = true;
    args.resource_format = crate::dashboard::DashboardResourceFormat::All;
    args.resource_namespace = Some("source".to_string());
    args.portable_v2 = true;
    let error = export_dashboards_with_request(|method, path, _, _| {
        assert_eq!(method, reqwest::Method::GET);
        if path.contains("/v2/") {
            return Err(crate::common::api_response(404, path.to_string(), "not served".to_string()));
        }
        Ok(Some(json!({"apiVersion":"dashboard.grafana.app/v1","kind":"DashboardList","metadata":{},"items":[{"apiVersion":"dashboard.grafana.app/v1","kind":"Dashboard","metadata":{"name":"v1","namespace":"source"},"spec":{"title":"V1","panels":[]}}]})))
    }, &args).unwrap_err();
    assert!(error
        .to_string()
        .contains("--portable-v2 requires the dashboard.grafana.app/v2 API"));
    assert!(!args.output_dir.join("resource-v1/objects/v1.json").exists());
}

#[test]
fn grafana_13_native_v2_nested_layout_survives_export_import() {
    let temp = tempdir().unwrap();
    let mut args = make_history_only_export_args(temp.path().join("dashboards"), None, false, true);
    args.include_history = false;
    args.without_dashboard_raw = true;
    args.without_dashboard_prompt = true;
    args.without_dashboard_provisioning = true;
    args.resource_format = crate::dashboard::DashboardResourceFormat::V2;
    args.portable_v2 = true;
    args.resource_namespace = Some("default".to_string());
    // v13.2.1 v2 schema shapes; this checks JSON preservation, not server acceptance.
    let inner_tabs = json!({"kind":"TabsLayout","spec":{"tabs":[{
        "kind":"TabsLayoutTab","spec":{"title":"CPU details",
            "layout":{"kind":"GridLayout","spec":{"items":[{
                "kind":"GridLayoutItem","spec":{"x":0,"y":0,"width":12,"height":8,
                    "element":{"kind":"ElementReference","name":"panel-1"}}
            }]}}}
    }]}});
    let mut spec = json!({
        "title":"Nested layouts",
        "elements":{"panel-1":{"kind":"Panel","spec":{"id":1,"title":"CPU",
            "customPanelOption":{"enabled":true}}}},
        "layout":{"kind":"TabsLayout","spec":{"tabs":[{"kind":"TabsLayoutTab","spec":{
            "title":"Overview","layout":{"kind":"RowsLayout","spec":{"rows":[{
                "kind":"RowsLayoutRow","spec":{"title":"Compute","collapse":true,
                    "conditionalRendering":{"kind":"ConditionalRenderingGroup","spec":{
                        "visibility":"show","condition":"and","items":[{
                            "kind":"ConditionalRenderingVariable","spec":{
                                "variable":"instance","operator":"equals","value":"node-1"}
                        }]}},
                    "layout":inner_tabs}
            }]}}
        }}]}},
        "customSpecOption":{"nested":[null,false,17,{"keep":"verbatim"}]}
    });
    spec["variables"] = json!([
        {"kind":"QueryVariable","spec":{"name":"instance",
            "current":{"text":"node-1","value":"node-1"},"hide":"dontHide",
            "refresh":"onDashboardLoad","skipUrlSync":false,
            "query":{"kind":"DataQuery","group":"prometheus","version":"v0",
                "datasource":{"name":"prom-main"},"spec":{"query":"label_values(up, instance)"}},
            "regex":"","sort":"disabled","options":[],"multi":false,
            "includeAll":false,"allowCustomValue":true}},
        {"kind":"AdhocVariable","group":"prometheus","datasource":{"name":"prom-main"},
            "spec":{"name":"filters","baseFilters":[],
                "filters":[{"key":"job","operator":"=","value":"node"}],"defaultKeys":[],
                "hide":"dontHide","skipUrlSync":false,"allowCustomValue":true,"enableGroupBy":true}},
        {"kind":"GroupByVariable","group":"prometheus","datasource":{"name":"prom-main"},
            "spec":{"name":"grouping","current":{"text":["instance"],"value":["instance"]},
                "options":[],"multi":true,"hide":"dontHide","skipUrlSync":false}}
    ]);
    let source = json!({"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard",
        "metadata":{"name":"nested","namespace":"default","resourceVersion":"7"},"spec":spec});
    export_dashboards_with_request(
        |method, path, _, _| match (method.as_str(), path) {
            ("GET", "/api/org") => Ok(Some(json!({"id":1,"name":"Main Org"}))),
            ("GET", "/apis/dashboard.grafana.app/v2/namespaces/default/dashboards") => Ok(Some(
                json!({"apiVersion":"dashboard.grafana.app/v2","kind":"DashboardList",
                "metadata":{},"items":[source.clone()]}),
            )),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();
    let exported: Value = serde_json::from_str(
        &fs::read_to_string(args.output_dir.join("resource-v2/objects/nested.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(exported["spec"], spec);
    let discovered =
        crate::dashboard::import::discover_dashboard_v2_resources(&args.output_dir).unwrap();
    assert_eq!(discovered.len(), 1);
    let create = crate::dashboard::import::normalize_dashboard_v2_create_payload(
        &exported,
        "target",
        Some("folder"),
        "",
    )
    .unwrap();
    let live = json!({"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard",
        "metadata":{"name":"nested","namespace":"target","resourceVersion":"9"},"spec":{}});
    let update = crate::dashboard::import::normalize_dashboard_v2_update_payload(
        &exported,
        &live,
        "target",
        Some("folder"),
        "",
    )
    .unwrap();
    assert_eq!(create["spec"], spec);
    assert_eq!(update["spec"], spec);
    assert_eq!(update["metadata"]["resourceVersion"], "9");
}

#[test]
fn export_dashboards_uses_explicit_resource_namespace_for_resource_only_export() {
    let temp = tempdir().unwrap();
    let mut args = make_history_only_export_args(temp.path().join("dashboards"), None, false, true);
    args.include_history = false;
    args.without_dashboard_raw = true;
    args.without_dashboard_prompt = true;
    args.without_dashboard_provisioning = true;
    args.resource_format = crate::dashboard::DashboardResourceFormat::V2;
    args.resource_namespace = Some("stacks-123".to_string());

    let count = export_dashboards_with_request(
        |method, path, params, _payload| match (method.as_str(), path) {
            ("GET", "/apis/dashboard.grafana.app/v2/namespaces/stacks-123/dashboards") => {
                assert_eq!(params, &[("limit".to_string(), "500".to_string())]);
                Ok(Some(json!({
                    "apiVersion": "dashboard.grafana.app/v2",
                    "kind": "DashboardList",
                    "items": [{
                        "kind": "Dashboard",
                        "apiVersion": "dashboard.grafana.app/v2",
                        "metadata": {"name": "cpu-main", "namespace": "stacks-123"},
                        "spec": {"title": "CPU Main", "elements": {}}
                    }],
                    "metadata": {}
                })))
            }
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(args
        .output_dir
        .join("resource-v2/objects/cpu-main.json")
        .is_file());
    let lane_metadata: Value = serde_json::from_str(
        &fs::read_to_string(args.output_dir.join("resource-v2/export-metadata.json")).unwrap(),
    )
    .unwrap();
    let root_metadata: Value = serde_json::from_str(
        &fs::read_to_string(args.output_dir.join("export-metadata.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(lane_metadata["orgId"], Value::String("unknown".to_string()));
    assert_eq!(root_metadata["orgId"], Value::String("unknown".to_string()));
}

#[test]
fn export_dashboards_mirrors_nested_folder_paths_for_raw_and_prompt() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        output_dir: temp.path().join("dashboards"),
        run: None,
        run_id: None,
        page_size: 500,
        org_id: None,
        all_orgs: false,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: false,
        without_dashboard_provisioning: false,
        include_history: false,
        native: false,
        portable_v2: false,
        resource_format: crate::dashboard::DashboardResourceFormat::None,
        resource_namespace: None,
        provisioning_provider_name: "grafana-utils-dashboards".to_string(),
        provisioning_provider_org_id: None,
        provisioning_provider_path: None,
        provisioning_provider_disable_deletion: false,
        provisioning_provider_allow_ui_updates: false,
        provisioning_provider_update_interval_seconds: 30,
        dry_run: false,
        progress: false,
        verbose: false,
    };

    let count = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
            "/api/search" => Ok(Some(json!([
                {
                    "uid": "cpu-main",
                    "title": "CPU",
                    "folderUid": "infra-a",
                    "folderTitle": "Infra"
                },
                {
                    "uid": "api-main",
                    "title": "API",
                    "folderUid": "infra-b",
                    "folderTitle": "Infra"
                }
            ]))),
            "/api/datasources" => Ok(Some(json!([]))),
            "/api/folders/infra-a" => Ok(Some(json!({
                "uid": "infra-a",
                "title": "Infra",
                "parents": [
                    {"uid": "platform", "title": "Platform"},
                    {"uid": "team-a", "title": "Team A"}
                ]
            }))),
            "/api/folders/infra-b" => Ok(Some(json!({
                "uid": "infra-b",
                "title": "Infra",
                "parents": [
                    {"uid": "apps", "title": "Apps"},
                    {"uid": "team-b", "title": "Team B"}
                ]
            }))),
            "/api/dashboards/uid/cpu-main" => Ok(Some(json!({
                "dashboard": {"id": 7, "uid": "cpu-main", "title": "CPU", "panels": []}
            }))),
            "/api/dashboards/uid/api-main" => Ok(Some(json!({
                "dashboard": {"id": 8, "uid": "api-main", "title": "API", "panels": []}
            }))),
            path if path.ends_with("/permissions") => Ok(Some(json!([]))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert!(args
        .output_dir
        .join("raw/Platform/Team_A/Infra/cpu-main.json")
        .is_file());
    assert!(args
        .output_dir
        .join("prompt/Platform/Team_A/Infra/cpu-main.json")
        .is_file());
    assert!(args
        .output_dir
        .join("raw/Apps/Team_B/Infra/api-main.json")
        .is_file());
    assert!(args
        .output_dir
        .join("prompt/Apps/Team_B/Infra/api-main.json")
        .is_file());
    assert!(args
        .output_dir
        .join("provisioning/dashboards/Infra/cpu-main.json")
        .is_file());
    assert!(args
        .output_dir
        .join("provisioning/dashboards/Infra/api-main.json")
        .is_file());
}

#[test]
fn export_dashboards_with_request_include_history_writes_scope_history_artifacts() {
    let current_temp = tempdir().unwrap();
    let current_args =
        make_history_only_export_args(current_temp.path().join("current"), None, false, true);
    let current_dashboard_fetches = Cell::new(0usize);

    let current_count = export_dashboards_with_request(
        |_method, path, params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
            "/api/search" => Ok(Some(json!([
                { "uid": "cpu-main", "title": "CPU Main", "folderTitle": "General" }
            ]))),
            "/api/datasources" => Ok(Some(json!([]))),
            "/api/folders/general" => Ok(Some(json!({"uid": "general", "title": "General"}))),
            "/api/dashboards/uid/cpu-main" => {
                current_dashboard_fetches.set(current_dashboard_fetches.get() + 1);
                Ok(Some(json!({
                    "dashboard": {"id": 7, "uid": "cpu-main", "title": "CPU Main", "version": 21}
                })))
            }
            "/api/dashboards/uid/cpu-main/versions" => {
                assert!(params
                    .iter()
                    .any(|(key, value)| key == "limit" && value == "20"));
                Ok(Some(json!([
                    {
                        "version": 21,
                        "created": "2026-04-02T12:00:00Z",
                        "createdBy": "ops",
                        "message": "Tune thresholds"
                    }
                ])))
            }
            "/api/dashboards/uid/cpu-main/versions/21" => Ok(Some(json!({
                "data": {"uid": "cpu-main", "title": "CPU Main", "version": 21}
            }))),
            "/api/dashboards/uid/cpu-main/permissions" => Ok(Some(json!([]))),
            "/api/folders/general/permissions" => Ok(Some(json!([]))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &current_args,
    )
    .unwrap();

    assert_eq!(current_count, 1);
    assert_eq!(current_dashboard_fetches.get(), 1);
    let current_history = current_temp
        .path()
        .join("current/history/cpu-main/v21.history.json");
    assert!(current_history.is_file());
    assert!(!current_temp
        .path()
        .join("current/history/cpu-main.history.json")
        .exists());
    let current_document: Value =
        serde_json::from_str(&fs::read_to_string(&current_history).unwrap()).unwrap();
    assert_eq!(
        current_document["kind"],
        Value::String("grafana-util-dashboard-history-export".to_string())
    );
    assert_eq!(
        current_document["dashboardUid"],
        Value::String("cpu-main".to_string())
    );
    assert_eq!(current_document["versionCount"], Value::Number(1.into()));
    assert_eq!(
        current_document["versions"][0]["version"],
        Value::Number(21.into())
    );

    let org_temp = tempdir().unwrap();
    let org_args = make_history_only_export_args(org_temp.path().join("org"), Some(2), false, true);

    let org_count = export_dashboards_with_request(
        |_method, path, params, _payload| match path {
            "/api/org" => {
                assert!(params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "2"));
                Ok(Some(json!({"id": 2, "name": "Ops Org"})))
            }
            "/api/search" => {
                assert!(params
                    .iter()
                    .any(|(key, value)| key == "orgId" && value == "2"));
                Ok(Some(json!([
                    { "uid": "ops-main", "title": "Ops Main", "folderTitle": "Ops" }
                ])))
            }
            "/api/datasources" => Ok(Some(json!([]))),
            "/api/folders/ops" => Ok(Some(json!({"uid": "ops", "title": "Ops"}))),
            "/api/dashboards/uid/ops-main" => Ok(Some(json!({
                "dashboard": {"id": 8, "uid": "ops-main", "title": "Ops Main", "version": 4}
            }))),
            "/api/dashboards/uid/ops-main/versions" => Ok(Some(json!([
                {
                    "version": 4,
                    "created": "2026-04-03T12:00:00Z",
                    "createdBy": "ops",
                    "message": "Record ops baseline"
                }
            ]))),
            "/api/dashboards/uid/ops-main/versions/4" => Ok(Some(json!({
                "data": {"uid": "ops-main", "title": "Ops Main", "version": 4}
            }))),
            "/api/dashboards/uid/ops-main/permissions" => Ok(Some(json!([]))),
            "/api/folders/ops/permissions" => Ok(Some(json!([]))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &org_args,
    )
    .unwrap();

    assert_eq!(org_count, 1);
    let org_history = org_temp.path().join("org/history/ops-main/v4.history.json");
    assert!(org_history.is_file());
    assert!(!org_temp
        .path()
        .join("org/history/ops-main.history.json")
        .exists());
    let org_document: Value =
        serde_json::from_str(&fs::read_to_string(&org_history).unwrap()).unwrap();
    assert_eq!(
        org_document["kind"],
        Value::String("grafana-util-dashboard-history-export".to_string())
    );
    assert_eq!(
        org_document["dashboardUid"],
        Value::String("ops-main".to_string())
    );
    assert_eq!(org_document["versionCount"], Value::Number(1.into()));
    assert_eq!(
        org_document["versions"][0]["version"],
        Value::Number(4.into())
    );
}

#[test]
fn export_dashboards_with_request_include_history_writes_all_org_history_artifacts() {
    let temp = tempdir().unwrap();
    let args = make_history_only_export_args(temp.path().join("all-orgs"), None, true, true);

    let count = export_dashboards_with_request(
        |method, path, params, _payload| {
            let scoped_org = params
                .iter()
                .find(|(key, _)| key == "orgId")
                .map(|(_, value)| value.as_str());
            match (method.clone(), path, scoped_org) {
                (reqwest::Method::GET, "/api/orgs", _) => Ok(Some(json!([
                    {"id": 1, "name": "Main Org"},
                    {"id": 2, "name": "Ops Org"}
                ]))),
                (reqwest::Method::GET, "/api/org", Some("1")) => {
                    Ok(Some(json!({"id": 1, "name": "Main Org"})))
                }
                (reqwest::Method::GET, "/api/org", Some("2")) => {
                    Ok(Some(json!({"id": 2, "name": "Ops Org"})))
                }
                (reqwest::Method::GET, "/api/search", Some("1")) => Ok(Some(json!([
                    { "uid": "cpu-main", "title": "CPU Main", "folderTitle": "General" }
                ]))),
                (reqwest::Method::GET, "/api/search", Some("2")) => Ok(Some(json!([
                    { "uid": "ops-main", "title": "Ops Main", "folderTitle": "Ops" }
                ]))),
                (reqwest::Method::GET, "/api/datasources", Some("1")) => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/datasources", Some("2")) => Ok(Some(json!([]))),
                (reqwest::Method::GET, "/api/folders/general", Some("1")) => {
                    Ok(Some(json!({"uid": "general", "title": "General"})))
                }
                (reqwest::Method::GET, "/api/folders/ops", Some("2")) => {
                    Ok(Some(json!({"uid": "ops", "title": "Ops"})))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/cpu-main", Some("1")) => {
                    Ok(Some(json!({"dashboard": {"id": 7, "uid": "cpu-main", "title": "CPU Main", "version": 21}})))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/ops-main", Some("2")) => {
                    Ok(Some(json!({"dashboard": {"id": 8, "uid": "ops-main", "title": "Ops Main", "version": 4}})))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/cpu-main/versions", Some("1")) => {
                    Ok(Some(json!([
                        {
                            "version": 21,
                            "created": "2026-04-02T12:00:00Z",
                            "createdBy": "ops",
                            "message": "Tune thresholds"
                        }
                    ])))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/ops-main/versions", Some("2")) => {
                    Ok(Some(json!([
                        {
                            "version": 4,
                            "created": "2026-04-03T12:00:00Z",
                            "createdBy": "ops",
                            "message": "Record ops baseline"
                        }
                    ])))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/cpu-main/versions/21", Some("1")) => {
                    Ok(Some(json!({
                        "data": {"uid": "cpu-main", "title": "CPU Main", "version": 21}
                    })))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/ops-main/versions/4", Some("2")) => {
                    Ok(Some(json!({
                        "data": {"uid": "ops-main", "title": "Ops Main", "version": 4}
                    })))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/cpu-main/permissions", Some("1")) => {
                    Ok(Some(json!([])))
                }
                (reqwest::Method::GET, "/api/dashboards/uid/ops-main/permissions", Some("2")) => {
                    Ok(Some(json!([])))
                }
                (reqwest::Method::GET, "/api/folders/general/permissions", Some("1")) => {
                    Ok(Some(json!([])))
                }
                (reqwest::Method::GET, "/api/folders/ops/permissions", Some("2")) => {
                    Ok(Some(json!([])))
                }
                _ => Err(test_support::message(format!("unexpected path {path}"))),
            }
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 2);
    assert!(temp
        .path()
        .join("all-orgs/org_1_Main_Org/history/cpu-main/v21.history.json")
        .is_file());
    assert!(temp
        .path()
        .join("all-orgs/org_2_Ops_Org/history/ops-main/v4.history.json")
        .is_file());
    assert!(!temp
        .path()
        .join("all-orgs/org_1_Main_Org/history/cpu-main.history.json")
        .exists());
}

#[test]
fn export_dashboards_with_request_include_history_respects_overwrite() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("current");
    fs::create_dir_all(output_dir.join("history/cpu-main")).unwrap();
    fs::write(
        output_dir.join("history/cpu-main/v21.history.json"),
        "{\"kind\":\"existing\"}\n",
    )
    .unwrap();
    let args = make_history_only_export_args(output_dir.clone(), None, false, false);

    let error = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
            "/api/search" => Ok(Some(json!([
                { "uid": "cpu-main", "title": "CPU Main", "folderTitle": "General" }
            ]))),
            "/api/datasources" => Ok(Some(json!([]))),
            "/api/folders/general" => Ok(Some(json!({"uid": "general", "title": "General"}))),
            "/api/dashboards/uid/cpu-main" => Ok(Some(json!({
                "dashboard": {"id": 7, "uid": "cpu-main", "title": "CPU Main", "version": 21}
            }))),
            "/api/dashboards/uid/cpu-main/versions" => Ok(Some(json!([
                {
                    "version": 21,
                    "created": "2026-04-02T12:00:00Z",
                    "createdBy": "ops",
                    "message": "Tune thresholds"
                }
            ]))),
            "/api/dashboards/uid/cpu-main/versions/21" => Ok(Some(json!({
                "data": {"uid": "cpu-main", "title": "CPU Main", "version": 21}
            }))),
            "/api/dashboards/uid/cpu-main/permissions" => Ok(Some(json!([]))),
            "/api/folders/general/permissions" => Ok(Some(json!([]))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("Refusing to overwrite existing file"));
}

#[test]
fn export_dashboards_with_request_include_history_overwrite_removes_stale_version_files() {
    let temp = tempdir().unwrap();
    let output_dir = temp.path().join("current");
    fs::create_dir_all(output_dir.join("history/cpu-main")).unwrap();
    fs::write(
        output_dir.join("history/cpu-main/v20.history.json"),
        "{\"kind\":\"stale\"}\n",
    )
    .unwrap();
    fs::write(output_dir.join("history/cpu-main/notes.txt"), "keep\n").unwrap();
    let args = make_history_only_export_args(output_dir.clone(), None, false, true);

    let count = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 1, "name": "Main Org"}))),
            "/api/search" => Ok(Some(json!([
                { "uid": "cpu-main", "title": "CPU Main", "folderTitle": "General" }
            ]))),
            "/api/datasources" => Ok(Some(json!([]))),
            "/api/folders/general" => Ok(Some(json!({"uid": "general", "title": "General"}))),
            "/api/dashboards/uid/cpu-main" => Ok(Some(json!({
                "dashboard": {"id": 7, "uid": "cpu-main", "title": "CPU Main", "version": 21}
            }))),
            "/api/dashboards/uid/cpu-main/versions" => Ok(Some(json!([
                {
                    "version": 21,
                    "created": "2026-04-02T12:00:00Z",
                    "createdBy": "ops",
                    "message": "Tune thresholds"
                }
            ]))),
            "/api/dashboards/uid/cpu-main/versions/21" => Ok(Some(json!({
                "data": {"uid": "cpu-main", "title": "CPU Main", "version": 21}
            }))),
            "/api/dashboards/uid/cpu-main/permissions" => Ok(Some(json!([]))),
            "/api/folders/general/permissions" => Ok(Some(json!([]))),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(!output_dir
        .join("history/cpu-main/v20.history.json")
        .exists());
    assert!(output_dir
        .join("history/cpu-main/v21.history.json")
        .is_file());
    assert!(output_dir.join("history/cpu-main/notes.txt").is_file());
}

#[test]
fn export_dashboards_with_dry_run_keeps_output_dir_empty() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        output_dir: temp.path().join("dashboards"),
        run: None,
        run_id: None,
        page_size: 500,
        org_id: None,
        all_orgs: false,
        flat: false,
        overwrite: true,
        without_dashboard_raw: false,
        without_dashboard_prompt: true,
        without_dashboard_provisioning: true,
        include_history: false,
        native: false,
        portable_v2: false,
        resource_format: crate::dashboard::DashboardResourceFormat::None,
        resource_namespace: None,
        provisioning_provider_name: "grafana-utils-dashboards".to_string(),
        provisioning_provider_org_id: None,
        provisioning_provider_path: None,
        provisioning_provider_disable_deletion: false,
        provisioning_provider_allow_ui_updates: false,
        provisioning_provider_update_interval_seconds: 30,
        dry_run: true,
        progress: false,
        verbose: false,
    };

    let count = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 1, "name": "Main Org."}))),
            "/api/search" => Ok(Some(
                json!([{ "uid": "abc", "title": "CPU", "folderTitle": "Infra" }]),
            )),
            "/api/datasources" => Ok(Some(json!([
                {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
            ]))),
            "/api/dashboards/uid/abc" => Ok(Some(
                json!({"dashboard": {"id": 7, "uid": "abc", "title": "CPU"}}),
            )),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(!args.output_dir.exists());
}

#[test]
fn export_dashboards_writes_provisioning_artifacts_in_separate_lane() {
    let temp = tempdir().unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        output_dir: temp.path().join("dashboards"),
        run: None,
        run_id: None,
        page_size: 500,
        org_id: None,
        all_orgs: false,
        flat: true,
        overwrite: true,
        without_dashboard_raw: true,
        without_dashboard_prompt: true,
        without_dashboard_provisioning: false,
        include_history: false,
        native: false,
        portable_v2: false,
        resource_format: crate::dashboard::DashboardResourceFormat::None,
        resource_namespace: None,
        provisioning_provider_name: "grafana-utils-dashboards".to_string(),
        provisioning_provider_org_id: None,
        provisioning_provider_path: None,
        provisioning_provider_disable_deletion: false,
        provisioning_provider_allow_ui_updates: false,
        provisioning_provider_update_interval_seconds: 30,
        dry_run: false,
        progress: false,
        verbose: false,
    };

    let count = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 7, "name": "Platform Org"}))),
            "/api/search" => Ok(Some(
                json!([{ "uid": "cpu-main", "title": "CPU", "folderTitle": "Infra" }]),
            )),
            "/api/datasources" => Ok(Some(json!([
                {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
            ]))),
            "/api/dashboards/uid/cpu-main" => Ok(Some(
                json!({"dashboard": {"id": 7, "uid": "cpu-main", "title": "CPU", "panels": []}}),
            )),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    assert!(args
        .output_dir
        .join("provisioning/dashboards/Infra/cpu-main.json")
        .is_file());
    assert!(args.output_dir.join("provisioning/index.json").is_file());
    assert!(args
        .output_dir
        .join("provisioning/export-metadata.json")
        .is_file());
    assert!(args
        .output_dir
        .join("provisioning/provisioning/dashboards.yaml")
        .is_file());

    let root_index: Value =
        serde_json::from_str(&fs::read_to_string(args.output_dir.join("index.json")).unwrap())
            .unwrap();
    assert_eq!(
        root_index["variants"]["provisioning"],
        Value::String(
            args.output_dir
                .join("provisioning/index.json")
                .display()
                .to_string()
        )
    );
    assert_eq!(
        root_index["items"][0]["provisioning_path"],
        Value::String(
            args.output_dir
                .join("provisioning/dashboards/Infra/cpu-main.json")
                .display()
                .to_string()
        )
    );

    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(args.output_dir.join("provisioning/export-metadata.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        metadata["variant"],
        Value::String("provisioning".to_string())
    );
    assert_eq!(
        metadata["format"],
        Value::String("grafana-file-provisioning-dashboard".to_string())
    );

    let provider_yaml = fs::read_to_string(
        args.output_dir
            .join("provisioning/provisioning/dashboards.yaml"),
    )
    .unwrap();
    assert!(provider_yaml.contains("apiVersion: 1"));
    assert!(provider_yaml.contains("providers:"));
    assert!(provider_yaml.contains("orgId: 7"));
    assert!(provider_yaml.contains("type: file"));
    assert!(provider_yaml.contains("foldersFromFilesStructure: true"));
    let expected_dashboard_path = fs::canonicalize(args.output_dir.join("provisioning/dashboards"))
        .unwrap()
        .display()
        .to_string();
    assert!(provider_yaml.contains(&format!("path: {expected_dashboard_path}")));
    assert!(!provider_yaml.contains("REPLACE_WITH_PROVISIONING_DASHBOARD_PATH"));
}

#[test]
fn export_dashboards_writes_custom_provisioning_provider_settings() {
    let temp = tempdir().unwrap();
    let custom_provider_path = temp.path().join("provider-target");
    fs::create_dir_all(&custom_provider_path).unwrap();
    let args = ExportArgs {
        common: make_common_args("http://127.0.0.1:3000".to_string()),
        output_dir: temp.path().join("dashboards"),
        run: None,
        run_id: None,
        page_size: 500,
        org_id: None,
        all_orgs: false,
        flat: true,
        overwrite: true,
        without_dashboard_raw: true,
        without_dashboard_prompt: true,
        without_dashboard_provisioning: false,
        include_history: false,
        native: false,
        portable_v2: false,
        resource_format: crate::dashboard::DashboardResourceFormat::None,
        resource_namespace: None,
        provisioning_provider_name: "grafana-utils-prod".to_string(),
        provisioning_provider_org_id: Some(42),
        provisioning_provider_path: Some(custom_provider_path.clone()),
        provisioning_provider_disable_deletion: true,
        provisioning_provider_allow_ui_updates: true,
        provisioning_provider_update_interval_seconds: 120,
        dry_run: false,
        progress: false,
        verbose: false,
    };

    let count = export_dashboards_with_request(
        |_method, path, _params, _payload| match path {
            "/api/org" => Ok(Some(json!({"id": 7, "name": "Platform Org"}))),
            "/api/search" => Ok(Some(
                json!([{ "uid": "cpu-main", "title": "CPU", "folderTitle": "Infra" }]),
            )),
            "/api/datasources" => Ok(Some(json!([
                {"uid": "prom-main", "name": "Prometheus Main", "type": "prometheus", "url": "http://prometheus:9090", "access": "proxy", "isDefault": true}
            ]))),
            "/api/dashboards/uid/cpu-main" => Ok(Some(
                json!({"dashboard": {"id": 7, "uid": "cpu-main", "title": "CPU", "panels": []}}),
            )),
            _ => Err(test_support::message(format!("unexpected path {path}"))),
        },
        &args,
    )
    .unwrap();

    assert_eq!(count, 1);
    let provider_yaml = fs::read_to_string(
        args.output_dir
            .join("provisioning/provisioning/dashboards.yaml"),
    )
    .unwrap();
    assert!(provider_yaml.contains("name: grafana-utils-prod"));
    assert!(provider_yaml.contains("orgId: 42"));
    assert!(provider_yaml.contains("disableDeletion: true"));
    assert!(provider_yaml.contains("allowUiUpdates: true"));
    assert!(provider_yaml.contains("updateIntervalSeconds: 120"));
    assert!(provider_yaml.contains("foldersFromFilesStructure: true"));
    let expected_provider_path = fs::canonicalize(&custom_provider_path)
        .unwrap()
        .display()
        .to_string();
    assert!(provider_yaml.contains(&format!("path: {expected_provider_path}")));
}
