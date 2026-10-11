//! Task-first `grafana-util workspace` smoke regressions.
//! Exercises the repo-local scan/test/preview/apply lane from one staged workspace.

use super::{ChangeOutputArgs, ChangePreviewArgs};
use crate::access::{
    ACCESS_EXPORT_KIND_ORGS, ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS, ACCESS_EXPORT_KIND_TEAMS,
    ACCESS_EXPORT_KIND_USERS, ACCESS_EXPORT_METADATA_FILENAME, ACCESS_ORG_EXPORT_FILENAME,
    ACCESS_SERVICE_ACCOUNT_EXPORT_FILENAME, ACCESS_TEAM_EXPORT_FILENAME,
    ACCESS_USER_EXPORT_FILENAME,
};
use crate::common::tool_version;
use crate::overview::{execute_overview, OverviewArgs, OverviewOutputFormat};
use crate::project_status_command::{
    execute_project_status_staged, ProjectStatusOutputFormat, ProjectStatusStagedArgs,
};
use crate::sync::{
    discover_change_staged_inputs, run_sync_cli, SyncApplyArgs, SyncCliArgs, SyncGroupCommand,
    SyncOutputFormat,
};
use clap::Parser;
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn write_dashboard_raw_fixture(root: &Path) {
    fs::create_dir_all(root).unwrap();
    fs::write(
        root.join("export-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": 1,
            "variant": "raw",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-web-import-preserve-uid",
            "foldersFile": "folders.json"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("folders.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "general",
                "title": "General",
                "parentUid": null,
                "path": "General",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("cpu.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "cpu-main",
                "title": "CPU Main",
                "panels": []
            },
            "meta": {
                "folderUid": "general"
            }
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_dashboard_provisioning_fixture(root: &Path) {
    fs::create_dir_all(root.join("dashboards").join("team")).unwrap();
    fs::write(
        root.join("folders.json"),
        serde_json::to_string_pretty(&json!([
            {
                "uid": "team",
                "title": "Team",
                "parentUid": null,
                "path": "Team",
                "org": "Main Org.",
                "orgId": "1"
            }
        ]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("export-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "kind": "grafana-utils-dashboard-export-index",
            "schemaVersion": 1,
            "variant": "provisioning",
            "dashboardCount": 1,
            "indexFile": "index.json",
            "format": "grafana-file-provisioning-dashboard"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("dashboards").join("team").join("cpu-main.json"),
        serde_json::to_string_pretty(&json!({
            "dashboard": {
                "uid": "cpu-main",
                "title": "CPU Main",
                "panels": []
            }
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_datasource_provisioning_fixture(root: &Path) {
    fs::create_dir_all(root).unwrap();
    fs::write(
        root.join("datasources.yaml"),
        r#"apiVersion: 1
datasources:
  - name: Prometheus Main
    type: prometheus
    uid: prom-main
    access: proxy
    url: http://prometheus:9090
    isDefault: true
"#,
    )
    .unwrap();
}

fn write_alert_export_fixture(root: &Path) {
    fs::create_dir_all(root).unwrap();
    fs::create_dir_all(root.join("rules").join("general").join("cpu-alerts")).unwrap();
    fs::write(
        root.join("index.json"),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "apiVersion": 1,
            "kind": "grafana-util-alert-export-index",
            "rules": [{
                "kind": "grafana-alert-rule",
                "uid": "cpu-high",
                "title": "CPU High",
                "folderUID": "general",
                "ruleGroup": "cpu-alerts",
                "path": "rules/general/cpu-alerts/CPU_High__cpu-high.json"
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("rules")
            .join("general")
            .join("cpu-alerts")
            .join("CPU_High__cpu-high.json"),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "toolVersion": tool_version(),
            "apiVersion": 1,
            "kind": "grafana-alert-rule",
            "metadata": {
                "uid": "cpu-high",
                "title": "CPU High"
            },
            "spec": {
                "uid": "cpu-high",
                "title": "CPU High",
                "folderUID": "general",
                "ruleGroup": "cpu-alerts",
                "condition": "A",
                "data": [{
                    "refId": "A",
                    "datasourceUid": "prom-main",
                    "model": {
                        "expr": "up",
                        "refId": "A"
                    }
                }]
            }
        }))
        .unwrap(),
    )
    .unwrap();
}

fn write_access_export_fixture(root: &Path, payload_filename: &str, kind: &str) {
    fs::create_dir_all(root).unwrap();
    fs::write(
        root.join(payload_filename),
        serde_json::to_string_pretty(&json!({
            "kind": kind,
            "version": 1,
            "records": [{
                "uid": "sample",
                "name": "sample"
            }]
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        root.join(ACCESS_EXPORT_METADATA_FILENAME),
        serde_json::to_string_pretty(&json!({
            "kind": kind,
            "version": 1,
            "recordCount": 1
        }))
        .unwrap(),
    )
    .unwrap();
}

fn task_first_workspace_cli_args(
    command: &str,
    workspace: &Path,
    live_file: Option<&Path>,
    preview_file: Option<&Path>,
) -> SyncCliArgs {
    let mut argv = vec!["grafana-util", command, "--output-format", "json"];
    if command != "apply" {
        argv.push(workspace.to_str().unwrap());
    }
    if let Some(live_file) = live_file {
        argv.extend(["--live-file", live_file.to_str().unwrap()]);
    }
    if let Some(preview_file) = preview_file {
        argv.extend(["--preview-file", preview_file.to_str().unwrap()]);
    }
    if command == "preview" {
        argv.extend(["--trace-id", "workspace-task-first-smoke"]);
    }
    if command == "apply" {
        argv.push("--approve");
    }
    SyncCliArgs::parse_from(argv)
}

#[test]
fn task_first_workspace_scan_and_test_use_datasource_inventory_and_output_file() {
    let temp = tempdir().unwrap();
    let inventory = temp.path().join("selected-datasources.json");
    fs::write(
        &inventory,
        r#"[{"uid":"selected","name":"Selected","type":"prometheus"}]"#,
    )
    .unwrap();
    write_datasource_provisioning_fixture(&temp.path().join("datasources/provisioning"));
    for command in ["scan", "test"] {
        let output = temp.path().join(format!("{command}.json"));
        let args = SyncCliArgs::parse_from([
            "grafana-util",
            command,
            temp.path().to_str().unwrap(),
            "--datasource-export-file",
            inventory.to_str().unwrap(),
            "--output-format",
            "json",
            "--output-file",
            output.to_str().unwrap(),
        ]);
        run_sync_cli(args.command).unwrap();
        let document: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&output).unwrap()).unwrap();
        if command == "scan" {
            assert_eq!(
                document["artifacts"][0]["document"]["datasources"][0]["uid"],
                "selected"
            );
            assert_eq!(
                document["artifacts"][0]["inputs"][0]["value"],
                inventory.display().to_string()
            );
        } else {
            assert_eq!(document["domains"][0]["id"], "datasource");
            assert_eq!(document["domains"][0]["primaryCount"], 1);
        }
    }
}

#[test]
fn task_first_workspace_test_fetch_live_propagates_connection_failure() {
    let temp = tempdir().unwrap();
    let desired = temp.path().join("desired.json");
    let output = temp.path().join("status.json");
    fs::write(&desired, "[]").unwrap();
    let args = SyncCliArgs::parse_from([
        "grafana-util",
        "test",
        temp.path().to_str().unwrap(),
        "--desired-file",
        desired.to_str().unwrap(),
        "--fetch-live",
        "--url",
        "http://127.0.0.1:9",
        "--token",
        "test-token",
        "--timeout",
        "1",
        "--output-format",
        "json",
        "--output-file",
        output.to_str().unwrap(),
    ]);
    assert!(run_sync_cli(args.command).is_err());
    assert!(!output.exists());
}

#[test]
fn task_first_workspace_test_fetch_live_checks_returned_availability() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    for bundle_mode in [false, true] {
        for available in [false, true] {
            let temp = tempdir().unwrap();
            let desired = json!([{
                "kind": "datasource", "uid": "selected", "title": "Selected",
                "body": {"uid": "selected", "name": "Selected", "type": "prometheus"}
            }]);
            let source = temp.path().join(if bundle_mode {
                "source.json"
            } else {
                "desired.json"
            });
            let target = temp.path().join("destination.json");
            let output = temp.path().join("status.json");
            let input = if bundle_mode {
                super::build_sync_source_bundle_document(
                    &[],
                    desired.as_array().unwrap(),
                    &[],
                    &[],
                    None,
                    None,
                )
                .unwrap()
            } else {
                desired
            };
            fs::write(&source, serde_json::to_string(&input).unwrap()).unwrap();
            fs::write(&target, "{}").unwrap();
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let server = std::thread::spawn(move || {
                let mut requests = Vec::new();
                let deadline = Instant::now() + Duration::from_secs(5);
                while requests.len() < 3 && Instant::now() < deadline {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            stream
                                .set_read_timeout(Some(Duration::from_secs(2)))
                                .unwrap();
                            let mut buffer = [0; 8192];
                            let mut request = Vec::new();
                            while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                                let count = stream.read(&mut buffer).unwrap();
                                if count == 0 {
                                    break;
                                }
                                request.extend_from_slice(&buffer[..count]);
                            }
                            let request = String::from_utf8(request).unwrap();
                            let path = request
                                .lines()
                                .next()
                                .unwrap()
                                .split_whitespace()
                                .nth(1)
                                .unwrap();
                            let body = match path {
                                "/api/datasources" => "[]",
                                "/api/plugins" if available => r#"[{"id":"prometheus"}]"#,
                                "/api/plugins" | "/api/v1/provisioning/contact-points" => "[]",
                                _ => panic!("unexpected request {path}"),
                            };
                            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
                            requests.push(request);
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(error) => panic!("{error}"),
                    }
                }
                requests
            });
            let mut argv = vec![
                "grafana-util",
                "test",
                temp.path().to_str().unwrap(),
                if bundle_mode {
                    "--source-bundle"
                } else {
                    "--desired-file"
                },
                source.to_str().unwrap(),
                "--fetch-live",
                "--url",
                url.as_str(),
                "--token",
                "test-token",
                "--org-id",
                "7",
                "--output-format",
                "json",
                "--output-file",
                output.to_str().unwrap(),
            ];
            if bundle_mode {
                argv.extend(["--target-inventory", target.to_str().unwrap()]);
            }
            let result = run_sync_cli(SyncCliArgs::parse_from(argv).command);
            let requests = server.join().unwrap();
            assert!(result.is_ok(), "{result:?}");
            assert_eq!(requests.len(), 3);
            assert!(requests
                .iter()
                .all(|request| request.to_ascii_lowercase().contains("x-grafana-org-id: 7")));
            let status: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
            let sync = status["domains"]
                .as_array()
                .unwrap()
                .iter()
                .find(|domain| domain["id"] == "sync")
                .unwrap();
            assert_eq!(sync["blockerCount"].as_i64().unwrap() > 0, !available);
            assert_eq!(sync["status"] == "blocked", !available);
        }
    }
}

#[test]
fn task_first_workspace_uses_explicit_bundle_before_discovered_inputs() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(workspace.join("alerts")).unwrap();
    fs::write(workspace.join("desired.json"), "invalid unselected input").unwrap();
    let source = temp.path().join("selected-source.json");
    let target = workspace.join("target-inventory.json");
    let live = temp.path().join("live.json");
    fs::write(&source, super::build_sync_source_bundle_document(
        &[json!({"kind":"dashboard","uid":"selected","title":"Selected","body":{"title":"Selected"}})],
        &[], &[], &[], None, None,
    ).unwrap().to_string()).unwrap();
    fs::write(&target, "{}").unwrap();
    fs::write(&live, "[]").unwrap();
    for command in ["scan", "test", "preview"] {
        let output = temp.path().join(format!("{command}-selected.json"));
        let mut argv = vec![
            "grafana-util",
            command,
            workspace.to_str().unwrap(),
            "--source-bundle",
            source.to_str().unwrap(),
            "--output-format",
            "json",
            "--output-file",
            output.to_str().unwrap(),
        ];
        if command == "preview" {
            argv.extend(["--live-file", live.to_str().unwrap()]);
        }
        run_sync_cli(SyncCliArgs::parse_from(argv).command).unwrap();
        let document: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
        if command == "scan" {
            assert_eq!(document["summary"]["artifactCount"], 1);
            assert_eq!(
                document["artifacts"][0]["inputs"][0]["value"],
                source.display().to_string()
            );
        } else if command == "test" {
            let sync = document["domains"]
                .as_array()
                .unwrap()
                .iter()
                .find(|domain| domain["id"] == "sync")
                .unwrap();
            assert_eq!(sync["primaryCount"], 1);
        } else {
            assert_eq!(document["kind"], "grafana-utils-sync-bundle-preflight");
            assert_eq!(document["summary"]["resourceCount"], 1);
        }
    }
}

#[test]
fn task_first_workspace_lane_smoke_runs_from_repo_local_workspace() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let dashboards_raw = workspace.join("dashboards").join("raw");
    write_dashboard_raw_fixture(&dashboards_raw);

    let live_file = workspace.join("live.json");
    fs::write(&live_file, "[]").unwrap();
    let preview_file = workspace.join("workspace-preview.json");

    let inspect_args = task_first_workspace_cli_args("scan", &workspace, None, None);
    match inspect_args.command {
        SyncGroupCommand::Inspect(inner) => {
            assert_eq!(inner.inputs.workspace, workspace);
            assert!(run_sync_cli(SyncGroupCommand::Inspect(inner)).is_ok());
        }
        _ => panic!("expected scan"),
    }

    let check_args = task_first_workspace_cli_args("test", &workspace, None, None);
    match check_args.command {
        SyncGroupCommand::Check(inner) => {
            assert_eq!(inner.inputs.workspace, workspace);
            assert!(run_sync_cli(SyncGroupCommand::Check(inner)).is_ok());
        }
        _ => panic!("expected test"),
    }

    let preview_args = task_first_workspace_cli_args("preview", &workspace, Some(&live_file), None);
    match preview_args.command {
        SyncGroupCommand::Preview(inner) => {
            assert_eq!(inner.inputs.workspace, workspace);
            assert_eq!(inner.live_file, Some(live_file.clone()));
            assert_eq!(
                inner.trace_id.as_deref(),
                Some("workspace-task-first-smoke")
            );
            assert!(run_sync_cli(SyncGroupCommand::Preview(ChangePreviewArgs {
                output: ChangeOutputArgs {
                    output_file: Some(preview_file.clone()),
                    ..inner.output.clone()
                },
                ..inner
            }))
            .is_ok());
        }
        _ => panic!("expected preview"),
    }

    let preview_raw = fs::read_to_string(&preview_file).unwrap();
    assert!(!preview_raw.contains('\u{1b}'));
    assert!(preview_raw.ends_with('\n'));
    let preview_document: serde_json::Value = serde_json::from_str(&preview_raw).unwrap();
    assert_eq!(preview_document["kind"], json!("grafana-utils-sync-plan"));
    assert_eq!(
        preview_document["traceId"],
        json!("workspace-task-first-smoke")
    );
    assert_eq!(preview_document["reviewed"], json!(false));
    assert_eq!(
        preview_document["discovery"]["workspaceRoot"],
        json!(workspace.display().to_string())
    );
    assert!(preview_document["actions"].as_array().is_some());
    assert!(preview_document["operations"].as_array().is_some());
    assert!(preview_document["domains"].as_array().is_some());
    assert!(preview_document["blockedReasons"].as_array().is_some());
    let domains = preview_document["domains"].as_array().unwrap();
    assert!(domains
        .iter()
        .any(|domain| domain["id"] == json!("dashboard")));
    assert!(domains
        .iter()
        .any(|domain| domain["id"] == json!("datasource")));
    assert!(domains.iter().any(|domain| domain["id"] == json!("alert")));
    let first_action = preview_document["actions"]
        .as_array()
        .unwrap()
        .first()
        .unwrap();
    assert!(first_action["actionId"].as_str().is_some());
    assert!(first_action["domain"].as_str().is_some());
    assert!(first_action["resourceKind"].as_str().is_some());

    let apply_args = task_first_workspace_cli_args("apply", &workspace, None, Some(&preview_file));
    match apply_args.command {
        SyncGroupCommand::Apply(inner) => {
            assert_eq!(inner.plan_file, Some(preview_file.clone()));
            assert!(run_sync_cli(SyncGroupCommand::Apply(SyncApplyArgs {
                output_format: SyncOutputFormat::Json,
                ..inner
            }))
            .is_ok());
        }
        _ => panic!("expected apply"),
    }
}

#[test]
fn task_first_workspace_preview_marks_reviewed_and_preserves_audit_metadata() {
    let temp = tempdir().unwrap();
    let workspace = temp.path();
    fs::write(workspace.join("desired.json"), "[]").unwrap();
    let live_file = workspace.join("live.json");
    fs::write(&live_file, "[]").unwrap();
    let preview_file = workspace.join("preview.json");
    let mut args = task_first_workspace_cli_args("preview", workspace, Some(&live_file), None);
    let SyncGroupCommand::Preview(ref mut preview) = args.command else {
        panic!("expected preview");
    };
    preview.mark_reviewed = true;
    preview.reviewed_by = Some("operator".to_string());
    preview.reviewed_at = Some("2026-10-11T12:00:00Z".to_string());
    preview.review_note = Some("checked all operations".to_string());
    preview.output.output_file = Some(preview_file.clone());

    run_sync_cli(args.command).unwrap();

    let document: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&preview_file).unwrap()).unwrap();
    assert_eq!(document["reviewed"], json!(true));
    assert_eq!(document["stage"], json!("review"));
    assert_eq!(document["stepIndex"], json!(2));
    assert_eq!(document["reviewedBy"], json!("operator"));
    assert_eq!(document["reviewedAt"], json!("2026-10-11T12:00:00Z"));
    assert_eq!(document["reviewNote"], json!("checked all operations"));
    assert_eq!(document["parentTraceId"], document["traceId"]);
}

#[test]
fn task_first_workspace_preview_rejects_invalid_review_token_before_output() {
    let temp = tempdir().unwrap();
    let workspace = temp.path();
    fs::write(workspace.join("desired.json"), "[]").unwrap();
    let live_file = workspace.join("live.json");
    fs::write(&live_file, "[]").unwrap();
    let preview_file = workspace.join("preview.json");
    let mut args = task_first_workspace_cli_args("preview", workspace, Some(&live_file), None);
    let SyncGroupCommand::Preview(ref mut preview) = args.command else {
        panic!("expected preview");
    };
    preview.mark_reviewed = true;
    preview.review_token = "unrecognized-token".to_string();
    preview.output.output_file = Some(preview_file.clone());

    let error = run_sync_cli(args.command).unwrap_err();

    assert!(error.to_string().contains("review token rejected"));
    assert!(!preview_file.exists());
}

#[test]
fn task_first_workspace_lane_smoke_runs_from_git_sync_workspace_root() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(workspace.join(".git")).unwrap();
    let dashboards_raw = workspace.join("dashboards").join("git-sync").join("raw");
    write_dashboard_raw_fixture(&dashboards_raw);
    let live_file = workspace.join("live.json");
    fs::write(&live_file, "[]").unwrap();

    let inspect_args = task_first_workspace_cli_args("scan", &workspace, None, None);
    match inspect_args.command {
        SyncGroupCommand::Inspect(inner) => {
            assert_eq!(inner.inputs.workspace, workspace);
            assert!(run_sync_cli(SyncGroupCommand::Inspect(inner)).is_ok());
        }
        _ => panic!("expected scan"),
    }

    let check_args = task_first_workspace_cli_args("test", &workspace, None, None);
    match check_args.command {
        SyncGroupCommand::Check(inner) => {
            assert_eq!(inner.inputs.workspace, workspace);
            assert!(run_sync_cli(SyncGroupCommand::Check(inner)).is_ok());
        }
        _ => panic!("expected test"),
    }

    let preview_args = task_first_workspace_cli_args("preview", &workspace, Some(&live_file), None);
    match preview_args.command {
        SyncGroupCommand::Preview(inner) => {
            assert_eq!(inner.inputs.workspace, workspace);
            assert_eq!(inner.live_file, Some(live_file.clone()));
            assert!(run_sync_cli(SyncGroupCommand::Preview(ChangePreviewArgs {
                output: ChangeOutputArgs {
                    output_file: None,
                    ..inner.output.clone()
                },
                ..inner
            }))
            .is_ok());
        }
        _ => panic!("expected preview"),
    }
}

#[test]
fn task_first_workspace_lane_smoke_runs_from_git_sync_mixed_workspace_root() {
    let temp = tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(workspace.join(".git")).unwrap();
    let dashboards_raw = workspace.join("dashboards").join("git-sync").join("raw");
    let dashboards_provisioning = workspace
        .join("dashboards")
        .join("git-sync")
        .join("provisioning");
    write_dashboard_raw_fixture(&dashboards_raw);
    write_dashboard_provisioning_fixture(&dashboards_provisioning);
    write_datasource_provisioning_fixture(&workspace.join("datasources").join("provisioning"));
    write_alert_export_fixture(&workspace.join("alerts").join("raw"));
    write_access_export_fixture(
        &workspace.join("access-users"),
        ACCESS_USER_EXPORT_FILENAME,
        ACCESS_EXPORT_KIND_USERS,
    );
    write_access_export_fixture(
        &workspace.join("access-teams"),
        ACCESS_TEAM_EXPORT_FILENAME,
        ACCESS_EXPORT_KIND_TEAMS,
    );
    write_access_export_fixture(
        &workspace.join("access-orgs"),
        ACCESS_ORG_EXPORT_FILENAME,
        ACCESS_EXPORT_KIND_ORGS,
    );
    write_access_export_fixture(
        &workspace.join("access-service-accounts"),
        ACCESS_SERVICE_ACCOUNT_EXPORT_FILENAME,
        ACCESS_EXPORT_KIND_SERVICE_ACCOUNTS,
    );
    let live_file = workspace.join("live.json");
    fs::write(&live_file, "[]").unwrap();
    let preview_file = workspace.join("workspace-preview.json");

    let inspect_args = task_first_workspace_cli_args("scan", &workspace, None, None);
    match inspect_args.command {
        SyncGroupCommand::Inspect(inner) => {
            let result = run_sync_cli(SyncGroupCommand::Inspect(inner));
            assert!(result.is_ok(), "{result:?}");
        }
        _ => panic!("expected scan"),
    }

    let check_args = task_first_workspace_cli_args("test", &workspace, None, None);
    match check_args.command {
        SyncGroupCommand::Check(inner) => {
            let result = run_sync_cli(SyncGroupCommand::Check(inner));
            assert!(result.is_ok(), "{result:?}");
        }
        _ => panic!("expected test"),
    }

    let preview_args = task_first_workspace_cli_args("preview", &workspace, Some(&live_file), None);
    match preview_args.command {
        SyncGroupCommand::Preview(inner) => {
            let result = run_sync_cli(SyncGroupCommand::Preview(ChangePreviewArgs {
                output: ChangeOutputArgs {
                    output_file: Some(preview_file.clone()),
                    ..inner.output.clone()
                },
                ..inner
            }));
            assert!(result.is_ok(), "{result:?}");
        }
        _ => panic!("expected preview"),
    }

    let discovered = discover_change_staged_inputs(Some(&workspace)).unwrap();
    let overview = execute_overview(&OverviewArgs {
        dashboard_export_dir: discovered.dashboard_export_dir.clone(),
        dashboard_provisioning_dir: None,
        datasource_export_dir: None,
        datasource_provisioning_file: discovered.datasource_provisioning_file.clone(),
        access_user_export_dir: discovered.access_user_export_dir.clone(),
        access_team_export_dir: discovered.access_team_export_dir.clone(),
        access_org_export_dir: discovered.access_org_export_dir.clone(),
        access_service_account_export_dir: discovered.access_service_account_export_dir.clone(),
        desired_file: discovered.desired_file.clone(),
        source_bundle: discovered.source_bundle.clone(),
        target_inventory: discovered.target_inventory.clone(),
        alert_export_dir: discovered.alert_export_dir.clone(),
        availability_file: discovered.availability_file.clone(),
        mapping_file: discovered.mapping_file.clone(),
        output_format: OverviewOutputFormat::Json,
    })
    .unwrap();
    assert_eq!(overview.summary.artifact_count, 7);
    assert_eq!(overview.summary.dashboard_export_count, 1);
    assert_eq!(overview.summary.datasource_export_count, 1);
    assert_eq!(overview.summary.alert_export_count, 1);
    assert_eq!(overview.summary.access_user_export_count, 1);
    assert_eq!(overview.summary.access_team_export_count, 1);
    assert_eq!(overview.summary.access_org_export_count, 1);
    assert_eq!(overview.summary.access_service_account_export_count, 1);
    assert!(overview
        .project_status
        .domains
        .iter()
        .any(|domain| domain.id == "dashboard"));
    assert!(overview
        .project_status
        .domains
        .iter()
        .any(|domain| domain.id == "datasource"));
    assert!(overview
        .project_status
        .domains
        .iter()
        .any(|domain| domain.id == "access"));
    assert!(overview
        .project_status
        .domains
        .iter()
        .any(|domain| domain.id == "alert"));

    let project_status = execute_project_status_staged(&ProjectStatusStagedArgs {
        dashboard_export_dir: discovered.dashboard_export_dir.clone(),
        dashboard_provisioning_dir: None,
        datasource_export_dir: None,
        datasource_provisioning_file: discovered.datasource_provisioning_file.clone(),
        access_user_export_dir: discovered.access_user_export_dir.clone(),
        access_team_export_dir: discovered.access_team_export_dir.clone(),
        access_org_export_dir: discovered.access_org_export_dir.clone(),
        access_service_account_export_dir: discovered.access_service_account_export_dir.clone(),
        desired_file: discovered.desired_file.clone(),
        source_bundle: discovered.source_bundle.clone(),
        target_inventory: discovered.target_inventory.clone(),
        alert_export_dir: discovered.alert_export_dir.clone(),
        availability_file: discovered.availability_file.clone(),
        mapping_file: discovered.mapping_file.clone(),
        output_format: ProjectStatusOutputFormat::Json,
    })
    .unwrap();
    assert_eq!(project_status.scope, "staged-only");
    assert_eq!(project_status.domains.len(), 4);
    assert!(project_status
        .domains
        .iter()
        .any(|domain| domain.id == "dashboard"));
    assert!(project_status
        .domains
        .iter()
        .any(|domain| domain.id == "datasource"));
    assert!(project_status
        .domains
        .iter()
        .any(|domain| domain.id == "access"));
    assert!(project_status
        .domains
        .iter()
        .any(|domain| domain.id == "alert"));

    let preview_document: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&preview_file).unwrap()).unwrap();
    assert_eq!(preview_document["kind"], json!("grafana-utils-sync-plan"));
    assert_eq!(
        preview_document["discovery"]["workspaceRoot"],
        json!(workspace.display().to_string())
    );
    assert!(preview_document["operations"].as_array().is_some());
    assert!(preview_document["operations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|operation| operation["kind"] == json!("dashboard")));
    assert!(preview_document["operations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|operation| operation["kind"] == json!("datasource")));
    assert!(preview_document["operations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|operation| operation["kind"] == json!("alert")));
    assert_eq!(
        preview_document["alertAssessment"]["kind"],
        json!("grafana-utils-alert-sync-plan")
    );
}
