use super::{DashboardCliArgs, DashboardCommand, DashboardImportInputFormat};
use clap::Parser;
use serde_json::{json, Value};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use tempfile::tempdir;

use super::import::{
    dashboard_v2_managed_source, dashboard_v2_resources_equal, discover_dashboard_v2_resources,
    import_dashboards_with_client, import_dashboards_with_request,
    normalize_dashboard_v2_create_payload, normalize_dashboard_v2_update_payload,
    validate_import_output_columns, validate_resource_v2_import_args,
};

fn spawn_resource_v2_server(
    responses: Vec<(&'static str, &'static str)>,
) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&requests);
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                request.extend_from_slice(&buffer[..count]);
                let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n")
                else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&request[..header_end + 4]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length: ")
                            .map(str::to_string)
                    })
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(0);
                if request.len() >= header_end + 4 + content_length {
                    break;
                }
            }
            captured
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(&request).into_owned());
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (format!("http://{address}"), requests, handle)
}

fn parse_resource_v2_import(extra: &[&str]) -> super::ImportArgs {
    let mut argv = vec![
        "grafana-util",
        "import",
        "--input-dir",
        "./resource-v2",
        "--input-format",
        "resource-v2",
    ];
    argv.extend_from_slice(extra);
    match DashboardCliArgs::try_parse_from(argv).unwrap().command {
        DashboardCommand::Import(args) => args,
        _ => panic!("expected dashboard import"),
    }
}

#[test]
fn comparison_ignores_server_added_deprecated_internal_id_label() {
    let desired = json!({"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard","metadata":{"name":"cpu","namespace":"default"},"spec":{"title":"CPU","elements":{}}});
    let mut live = desired.clone();
    live["metadata"]["labels"] = json!({"grafana.app/deprecatedInternalID":"123"});
    live["metadata"]["resourceVersion"] = json!("9");
    assert!(dashboard_v2_resources_equal(&desired, &live).unwrap());
}

#[test]
fn mutation_payload_strips_server_owned_deprecated_internal_id_label() {
    let source = json!({
        "apiVersion":"dashboard.grafana.app/v2",
        "kind":"Dashboard",
        "metadata":{
            "name":"cpu",
            "labels":{
                "grafana.app/deprecatedInternalID":"123",
                "example.com/team":"observability"
            }
        },
        "spec":{"title":"CPU","elements":{}}
    });

    let payload = normalize_dashboard_v2_create_payload(&source, "default", None, "").unwrap();
    assert_eq!(
        payload.pointer("/metadata/labels/example.com~1team"),
        Some(&json!("observability"))
    );
    assert!(payload
        .pointer("/metadata/labels/grafana.app~1deprecatedInternalID")
        .is_none());
}

#[test]
fn comparison_ignores_grafana_v2_schema_defaults_added_after_create() {
    let desired = json!({
        "apiVersion":"dashboard.grafana.app/v2",
        "kind":"Dashboard",
        "metadata":{"name":"cpu","namespace":"default"},
        "spec":{"title":"CPU","elements":{}}
    });
    let live = json!({
        "apiVersion":"dashboard.grafana.app/v2",
        "kind":"Dashboard",
        "metadata":{"name":"cpu","namespace":"default","resourceVersion":"9"},
        "spec":{
            "annotations":null,
            "cursorSync":"",
            "elements":{},
            "layout":{"kind":"GridLayout","spec":{"items":null}},
            "links":null,
            "preload":false,
            "tags":null,
            "timeSettings":{
                "from":"",
                "to":"",
                "autoRefresh":"",
                "autoRefreshIntervals":null,
                "hideTimepicker":false,
                "fiscalYearStartMonth":0
            },
            "title":"CPU",
            "variables":null
        }
    });

    assert!(dashboard_v2_resources_equal(&desired, &live).unwrap());
}

#[test]
fn ownership_detection_uses_explicit_metadata_signals_not_arbitrary_text() {
    let ordinary = json!({
        "metadata": {
            "annotations": {"example.com/note": "repository migration dashboard"}
        }
    });
    assert_eq!(dashboard_v2_managed_source(&ordinary), None);

    let git_sync = json!({
        "metadata": {
            "annotations": {"grafana.app/managedBy": "repository"}
        }
    });
    assert_eq!(
        dashboard_v2_managed_source(&git_sync),
        Some("Git Sync-managed")
    );

    let provisioned = json!({"metadata": {"provisioned": true}});
    assert_eq!(
        dashboard_v2_managed_source(&provisioned),
        Some("file-provisioned")
    );

    let plugin_managed = json!({"metadata": {"managedBy": {"kind": "plugin"}}});
    assert_eq!(
        dashboard_v2_managed_source(&plugin_managed),
        Some("managed-unknown")
    );
}

#[test]
fn parser_accepts_resource_v2_input_format_and_namespace() {
    let args = parse_resource_v2_import(&["--resource-namespace", "stacks-123"]);
    assert_eq!(args.input_format, DashboardImportInputFormat::ResourceV2);
    assert_eq!(args.resource_namespace.as_deref(), Some("stacks-123"));
}

#[test]
fn parser_defaults_dashboard_import_to_auto() {
    let args =
        match DashboardCliArgs::try_parse_from(["grafana-util", "import", "--input-dir", "./raw"])
            .unwrap()
            .command
        {
            DashboardCommand::Import(args) => args,
            _ => panic!("expected dashboard import"),
        };
    assert_eq!(args.input_format, DashboardImportInputFormat::Auto);
    assert_eq!(args.resource_namespace, None);
}

#[test]
fn parser_preserves_explicit_classic_import_mode() {
    let args = match DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        "./raw",
        "--input-format",
        "raw",
    ])
    .unwrap()
    .command
    {
        DashboardCommand::Import(args) => args,
        _ => panic!("expected dashboard import"),
    };

    assert_eq!(args.input_format, DashboardImportInputFormat::Raw);
}

#[test]
fn output_format_requires_dry_run_at_parser_and_runtime_boundaries() {
    let parser_error = DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        "./raw",
        "--output-format",
        "json",
    ])
    .unwrap_err()
    .to_string();
    assert!(parser_error.contains("--dry-run"), "{parser_error}");

    let mut args = parse_resource_v2_import(&["--dry-run", "--output-format", "json"]);
    args.dry_run = false;
    let mut requested = false;
    let runtime_error = import_dashboards_with_request(
        |_, _, _, _| {
            requested = true;
            Ok(None)
        },
        &args,
    )
    .unwrap_err()
    .to_string();
    assert!(runtime_error.contains("--output-format"), "{runtime_error}");
    assert!(!requested);
}

#[test]
fn import_output_columns_are_validated_for_the_selected_input_lane() {
    let resource_args =
        parse_resource_v2_import(&["--dry-run", "--table", "--output-columns", "uid"]);
    let resource_error = validate_import_output_columns(&resource_args)
        .unwrap_err()
        .to_string();
    assert!(resource_error.contains("uid"), "{resource_error}");
    assert!(resource_error.contains("resource-v2"), "{resource_error}");

    let classic_args = match DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        "./raw",
        "--dry-run",
        "--table",
        "--output-columns",
        "namespace",
    ])
    .unwrap()
    .command
    {
        DashboardCommand::Import(args) => args,
        _ => panic!("expected dashboard import"),
    };
    let classic_error = validate_import_output_columns(&classic_args)
        .unwrap_err()
        .to_string();
    assert!(classic_error.contains("namespace"), "{classic_error}");
    assert!(classic_error.contains("classic"), "{classic_error}");
}

#[test]
fn resource_v2_rejects_each_incompatible_classic_flag() {
    let cases: &[(&[&str], &str)] = &[
        (&["--org-id", "2"], "--org-id"),
        (&["--use-export-org"], "--use-export-org"),
        (&["--use-export-org", "--only-org-id", "2"], "--only-org-id"),
        (
            &["--use-export-org", "--create-missing-orgs"],
            "--create-missing-orgs",
        ),
        (
            &["--require-matching-export-org"],
            "--require-matching-export-org",
        ),
        (
            &["--require-matching-folder-path"],
            "--require-matching-folder-path",
        ),
        (&["--ensure-folders"], "--ensure-folders"),
        (&["--strict-schema"], "--strict-schema"),
        (
            &["--strict-schema", "--target-schema-version", "42"],
            "--target-schema-version",
        ),
    ];

    for (flags, expected_flag) in cases {
        let args = parse_resource_v2_import(flags);
        let error = validate_resource_v2_import_args(&args)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains(expected_flag),
            "expected {expected_flag} in {error}"
        );
    }
}

#[test]
fn incompatible_resource_v2_flags_fail_before_discovery_or_network() {
    let args = parse_resource_v2_import(&["--org-id", "2"]);
    let mut requests = 0;
    let error = import_dashboards_with_request(
        |_method, _path, _params, _payload| {
            requests += 1;
            Ok(None)
        },
        &args,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("--org-id"));
    assert_eq!(requests, 0);
}

fn write_resource(path: &std::path::Path, name: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        serde_json::to_string_pretty(&json!({
            "apiVersion": "dashboard.grafana.app/v2",
            "kind": "Dashboard",
            "metadata": {"name": name},
            "spec": {"title": name, "elements": {}}
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn discovers_standalone_resource_v2_directory_in_path_order() {
    let temp = tempdir().unwrap();
    write_resource(&temp.path().join("objects/z.json"), "z-dashboard");
    write_resource(&temp.path().join("objects/a.json"), "a-dashboard");
    fs::write(temp.path().join("_folder.json"), "{}").unwrap();
    fs::create_dir_all(temp.path().join("history")).unwrap();
    write_resource(&temp.path().join("history/old.json"), "old-dashboard");
    fs::write(
        temp.path().join("objects/folder.json"),
        serde_json::to_string(&json!({
            "apiVersion": "folder.grafana.app/v1",
            "kind": "Folder",
            "metadata": {"name": "ignored"},
            "spec": {}
        }))
        .unwrap(),
    )
    .unwrap();

    let resources = discover_dashboard_v2_resources(temp.path()).unwrap();
    assert_eq!(
        resources
            .iter()
            .map(|resource| resource.name.as_str())
            .collect::<Vec<_>>(),
        vec!["a-dashboard", "z-dashboard"]
    );
}

#[test]
fn discovers_combined_export_root_from_resource_v2_index_paths_only() {
    let temp = tempdir().unwrap();
    write_resource(
        &temp.path().join("resource-v2/objects/cpu.json"),
        "cpu-dashboard",
    );
    fs::create_dir_all(temp.path().join("raw")).unwrap();
    fs::write(
        temp.path().join("raw/classic.json"),
        "{\"title\":\"Classic\"}",
    )
    .unwrap();
    fs::write(
        temp.path().join("index.json"),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "kind": super::ROOT_INDEX_KIND,
            "items": [{
                "uid": "cpu-dashboard",
                "title": "CPU",
                "folderTitle": "General",
                "org": "Main",
                "orgId": "1",
                "raw_path": temp.path().join("raw/classic.json"),
                "resource_v2_path": "resource-v2/objects/cpu.json"
            }],
            "variants": {"raw": "raw/index.json", "prompt": null, "provisioning": null, "resource-v2": "resource-v2/index.json"}
        }))
        .unwrap(),
    )
    .unwrap();

    let resources = discover_dashboard_v2_resources(temp.path()).unwrap();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name, "cpu-dashboard");
}

#[test]
fn resource_v2_discovery_rejects_duplicates_mixed_versions_and_malformed_documents() {
    let duplicate = tempdir().unwrap();
    write_resource(&duplicate.path().join("a.json"), "same");
    write_resource(&duplicate.path().join("b.json"), "same");
    assert!(discover_dashboard_v2_resources(duplicate.path())
        .unwrap_err()
        .to_string()
        .contains("Duplicate"));

    let mixed = tempdir().unwrap();
    write_resource(&mixed.path().join("v2.json"), "v2");
    fs::write(
        mixed.path().join("v1.json"),
        serde_json::to_string(&json!({
            "apiVersion": "dashboard.grafana.app/v1",
            "kind": "Dashboard",
            "metadata": {"name": "v1"},
            "spec": {}
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(discover_dashboard_v2_resources(mixed.path())
        .unwrap_err()
        .to_string()
        .contains("dashboard.grafana.app/v1"));

    let malformed = tempdir().unwrap();
    fs::write(
        malformed.path().join("bad.json"),
        serde_json::to_string(&json!({
            "apiVersion": "dashboard.grafana.app/v2",
            "kind": "Dashboard",
            "metadata": {"generateName": "generated-"},
            "spec": []
        }))
        .unwrap(),
    )
    .unwrap();
    let error = discover_dashboard_v2_resources(malformed.path())
        .unwrap_err()
        .to_string();
    assert!(error.contains("metadata.generateName") || error.contains("metadata.name"));

    let list_envelope = tempdir().unwrap();
    write_resource(&list_envelope.path().join("dashboard.json"), "dashboard");
    fs::write(
        list_envelope.path().join("list.json"),
        serde_json::to_string(&json!({
            "apiVersion": "dashboard.grafana.app/v2",
            "kind": "DashboardList",
            "items": []
        }))
        .unwrap(),
    )
    .unwrap();
    let resources = discover_dashboard_v2_resources(list_envelope.path()).unwrap();
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].name, "dashboard");
}

#[test]
fn combined_root_rejects_resource_paths_outside_the_selected_export_root() {
    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    write_resource(&outside.path().join("outside.json"), "outside");
    fs::write(
        root.path().join("index.json"),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "kind": super::ROOT_INDEX_KIND,
            "items": [{
                "uid": "outside",
                "resource_v2_path": outside.path().join("outside.json")
            }]
        }))
        .unwrap(),
    )
    .unwrap();

    let error = discover_dashboard_v2_resources(root.path())
        .unwrap_err()
        .to_string();
    assert!(error.contains("outside export root"), "{error}");
}

#[cfg(unix)]
#[test]
fn standalone_discovery_rejects_symlinked_resources_outside_input_root() {
    use std::os::unix::fs::symlink;

    let root = tempdir().unwrap();
    let outside = tempdir().unwrap();
    write_resource(&outside.path().join("outside.json"), "outside");
    symlink(
        outside.path().join("outside.json"),
        root.path().join("linked.json"),
    )
    .unwrap();

    let error = discover_dashboard_v2_resources(root.path())
        .unwrap_err()
        .to_string();
    assert!(error.contains("symbolic link"), "{error}");
}

#[test]
fn create_and_update_payloads_match_schema_preserving_golden() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../../tests/fixtures/dashboard_resource_v2_import_golden.json"
    ))
    .unwrap();
    let create = normalize_dashboard_v2_create_payload(
        &fixture["source"],
        "target",
        Some("target-folder"),
        "Promote v2 dashboard",
    )
    .unwrap();
    let update = normalize_dashboard_v2_update_payload(
        &fixture["source"],
        &fixture["live"],
        "target",
        Some("target-folder"),
        "Promote v2 dashboard",
    )
    .unwrap();

    assert_eq!(create, fixture["create"]);
    assert_eq!(update, fixture["update"]);
    assert_eq!(create["spec"], fixture["source"]["spec"]);
    assert_eq!(update["spec"], fixture["source"]["spec"]);
}

#[test]
fn resource_v2_dry_run_reads_without_mutation() {
    let temp = tempdir().unwrap();
    write_resource(&temp.path().join("cpu.json"), "cpu-dashboard");
    let args = match DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        temp.path().to_str().unwrap(),
        "--input-format",
        "resource-v2",
        "--resource-namespace",
        "default",
        "--dry-run",
        "--json",
    ])
    .unwrap()
    .command
    {
        DashboardCommand::Import(args) => args,
        _ => unreachable!(),
    };
    let mut methods = Vec::new();
    let count = import_dashboards_with_request(|method, path, _, _| {
        methods.push(method.clone());
        if path.ends_with("/dashboards") {
            return Ok(Some(json!({"kind":"DashboardList","apiVersion":"dashboard.grafana.app/v2","metadata":{},"items":[]})));
        }
        Err(crate::common::api_response(404, path.to_string(), "not found".to_string()))
    }, &args).unwrap();
    assert_eq!(count, 1);
    assert!(methods.iter().all(|method| *method == reqwest::Method::GET));
}

#[test]
fn resource_v2_live_create_uses_v2_collection_only() {
    let temp = tempdir().unwrap();
    write_resource(&temp.path().join("cpu.json"), "cpu-dashboard");
    let args = match DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        temp.path().to_str().unwrap(),
        "--input-format",
        "resource-v2",
        "--resource-namespace",
        "default",
    ])
    .unwrap()
    .command
    {
        DashboardCommand::Import(args) => args,
        _ => unreachable!(),
    };
    let mut writes = Vec::new();
    let count = import_dashboards_with_request(|method, path, _, payload| {
        if method == reqwest::Method::GET && path.ends_with("/dashboards") {
            return Ok(Some(json!({"kind":"DashboardList","apiVersion":"dashboard.grafana.app/v2","metadata":{},"items":[]})));
        }
        if method == reqwest::Method::GET {
            return Err(crate::common::api_response(404, path.to_string(), "not found".to_string()));
        }
        writes.push((method.clone(), path.to_string()));
        Ok(payload.cloned())
    }, &args).unwrap();
    assert_eq!(count, 1);
    assert_eq!(
        writes,
        vec![(
            reqwest::Method::POST,
            "/apis/dashboard.grafana.app/v2/namespaces/default/dashboards".to_string()
        )]
    );
}

#[test]
fn production_client_entrypoint_routes_resource_v2_to_native_api() {
    let temp = tempdir().unwrap();
    write_resource(&temp.path().join("cpu.json"), "cpu");
    let args = match DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        temp.path().to_str().unwrap(),
        "--input-format",
        "resource-v2",
    ])
    .unwrap()
    .command
    {
        DashboardCommand::Import(args) => args,
        _ => unreachable!(),
    };
    let (base_url, requests, handle) = spawn_resource_v2_server(vec![
        (
            "200 OK",
            r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"DashboardList","metadata":{},"items":[]}"#,
        ),
        ("404 Not Found", r#"{"message":"not found"}"#),
        (
            "201 Created",
            r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard","metadata":{"namespace":"default","name":"cpu"},"spec":{"title":"cpu","elements":{}}}"#,
        ),
    ]);
    let client = crate::http::JsonHttpClient::new(crate::http::JsonHttpClientConfig {
        base_url,
        headers: Vec::new(),
        timeout_secs: 5,
        verify_ssl: false,
    })
    .unwrap();

    assert_eq!(import_dashboards_with_client(&client, &args).unwrap(), 1);
    handle.join().unwrap();
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert!(requests
        .iter()
        .all(|request| request.contains("/apis/dashboard.grafana.app/v2/")));
    assert!(requests
        .iter()
        .all(|request| !request.contains("/api/dashboards/db")));
}

#[test]
fn resource_v2_partial_apply_preserves_status_and_reports_truthful_partition() {
    let temp = tempdir().unwrap();
    write_resource(&temp.path().join("a.json"), "alpha");
    write_resource(&temp.path().join("b.json"), "bravo");
    let args = match DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        temp.path().to_str().unwrap(),
        "--input-format",
        "resource-v2",
    ])
    .unwrap()
    .command
    {
        DashboardCommand::Import(args) => args,
        _ => unreachable!(),
    };
    let mut writes = Vec::new();
    let error = import_dashboards_with_request(
        |method, path, _, payload| {
            if method == reqwest::Method::GET && path.ends_with("/dashboards") {
                return Ok(Some(json!({
                    "kind":"DashboardList",
                    "apiVersion":"dashboard.grafana.app/v2",
                    "metadata":{},
                    "items":[]
                })));
            }
            if method == reqwest::Method::GET {
                return Err(crate::common::api_response(404, path, "not found"));
            }
            let name = payload
                .and_then(|value| value.pointer("/metadata/name"))
                .and_then(serde_json::Value::as_str)
                .unwrap()
                .to_string();
            writes.push(name.clone());
            if name == "bravo" {
                return Err(crate::common::api_response(
                    409,
                    path,
                    "resourceVersion conflict",
                ));
            }
            Ok(payload.cloned())
        },
        &args,
    )
    .unwrap_err();

    assert_eq!(writes, ["alpha", "bravo"]);
    assert_eq!(error.status_code(), Some(409));
    let rendered = error.to_string();
    assert!(rendered.contains("completed=[alpha]"), "{rendered}");
    assert!(rendered.contains("failed=bravo"), "{rendered}");
    assert!(rendered.contains("not-attempted=[]"), "{rendered}");
    assert!(rendered.contains("no rollback was attempted"), "{rendered}");
}

#[test]
fn resource_v2_rejects_malformed_probe_and_mismatched_item_identity() {
    let temp = tempdir().unwrap();
    write_resource(&temp.path().join("cpu.json"), "cpu");
    let args = match DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        temp.path().to_str().unwrap(),
        "--input-format",
        "resource-v2",
        "--dry-run",
    ])
    .unwrap()
    .command
    {
        DashboardCommand::Import(args) => args,
        _ => unreachable!(),
    };

    let malformed_probe = import_dashboards_with_request(
        |_method, _path, _, _| {
            Ok(Some(json!({
                "apiVersion":"dashboard.grafana.app/v1",
                "kind":"DashboardList",
                "items":[]
            })))
        },
        &args,
    )
    .unwrap_err()
    .to_string();
    assert!(malformed_probe.contains("discovery"), "{malformed_probe}");

    let mismatched_item = import_dashboards_with_request(
        |_method, path, _, _| {
            if path.ends_with("/dashboards") {
                return Ok(Some(json!({
                    "apiVersion":"dashboard.grafana.app/v2",
                    "kind":"DashboardList",
                    "items":[]
                })));
            }
            Ok(Some(json!({
                "apiVersion":"dashboard.grafana.app/v2",
                "kind":"Dashboard",
                "metadata":{"name":"memory","resourceVersion":"1"},
                "spec":{}
            })))
        },
        &args,
    )
    .unwrap_err()
    .to_string();
    assert!(mismatched_item.contains("cpu"), "{mismatched_item}");
    assert!(mismatched_item.contains("Unexpected"), "{mismatched_item}");
}

#[test]
fn resource_v2_validates_write_response_identity_and_redacts_failure_secrets() {
    let temp = tempdir().unwrap();
    write_resource(&temp.path().join("cpu.json"), "cpu");
    let args = match DashboardCliArgs::try_parse_from([
        "grafana-util",
        "import",
        "--input-dir",
        temp.path().to_str().unwrap(),
        "--input-format",
        "resource-v2",
    ])
    .unwrap()
    .command
    {
        DashboardCommand::Import(args) => args,
        _ => unreachable!(),
    };

    let mismatched = import_dashboards_with_request(
        |method, path, _, _| {
            if method == reqwest::Method::GET && path.ends_with("/dashboards") {
                return Ok(Some(json!({
                    "apiVersion":"dashboard.grafana.app/v2",
                    "kind":"DashboardList",
                    "items":[]
                })));
            }
            if method == reqwest::Method::GET {
                return Err(crate::common::api_response(404, path, "not found"));
            }
            Ok(Some(json!({
                "apiVersion":"dashboard.grafana.app/v2",
                "kind":"Dashboard",
                "metadata":{"name":"memory"},
                "spec":{}
            })))
        },
        &args,
    )
    .unwrap_err()
    .to_string();
    assert!(mismatched.contains("cpu"), "{mismatched}");

    let secret_error = import_dashboards_with_request(
        |method, path, _, _| {
            if method == reqwest::Method::GET && path.ends_with("/dashboards") {
                return Ok(Some(json!({
                    "apiVersion":"dashboard.grafana.app/v2",
                    "kind":"DashboardList",
                    "items":[]
                })));
            }
            if method == reqwest::Method::GET {
                return Err(crate::common::api_response(404, path, "not found"));
            }
            Err(crate::common::api_response(
                403,
                path,
                "Authorization: Bearer abc123 token=hunter2",
            ))
        },
        &args,
    )
    .unwrap_err();
    assert_eq!(secret_error.status_code(), Some(403));
    let rendered = secret_error.to_string();
    assert!(!rendered.contains("abc123"), "{rendered}");
    assert!(!rendered.contains("hunter2"), "{rendered}");
    assert!(
        rendered.to_ascii_lowercase().contains("redacted"),
        "{rendered}"
    );
}
