//! Datasource import validation and import loader tests.

use crate::datasource::{
    load_import_records, run_datasource_cli, DatasourceCliArgs, DatasourceImportInputFormat,
};
use std::fs;
use tempfile::tempdir;

use super::*;

#[test]
fn routed_datasource_import_text_dry_run_never_writes() {
    use std::io::{ErrorKind, Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    for (org_exists, create_missing_orgs) in [(true, false), (false, false), (false, true)] {
        let temp = tempdir().unwrap();
        let input_dir = write_multi_org_import_fixture(
            temp.path(),
            &[(
                3,
                "Ops",
                vec![json!({
                    "uid": "prom-main", "name": "Prometheus", "type": "prometheus",
                    "access": "proxy", "url": "http://prometheus:9090", "isDefault": false,
                    "org": "Ops", "orgId": "3"
                })],
            )],
        );
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(error) if error.kind() == ErrorKind::PermissionDenied => return,
            Err(error) => panic!("failed to bind test listener: {error}"),
        };
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let done = Arc::new(AtomicBool::new(false));
        let server_done = done.clone();
        let server = thread::spawn(move || {
            let mut requests = Vec::new();
            let deadline = Instant::now() + Duration::from_secs(10);
            while !server_done.load(Ordering::Acquire) && Instant::now() < deadline {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("failed to accept test request: {error}"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut buffer = [0_u8; 1024];
                let request_line = loop {
                    let count = stream.read(&mut buffer).unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&buffer[..count]);
                    if bytes.windows(4).any(|part| part == b"\r\n\r\n") {
                        break String::from_utf8_lossy(&bytes)
                            .lines()
                            .next()
                            .unwrap()
                            .to_string();
                    }
                };
                let response = if request_line.starts_with("GET /api/orgs ") {
                    if org_exists {
                        json!([{"id": 3, "name": "Ops"}])
                    } else {
                        json!([])
                    }
                } else if request_line.starts_with("GET /api/datasources ") {
                    json!([])
                } else if request_line.starts_with("GET /api/org ") {
                    json!({"id": 3, "name": "Ops"})
                } else {
                    json!({})
                };
                requests.push(request_line);
                let body = response.to_string();
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
            requests
        });
        let mut args = vec![
            "grafana-util",
            "import",
            "--url",
            &url,
            "--basic-user",
            "admin",
            "--basic-password",
            "admin",
            "--input-dir",
            input_dir.to_str().unwrap(),
            "--use-export-org",
            "--dry-run",
        ];
        if create_missing_orgs {
            args.push("--create-missing-orgs");
        }
        let DatasourceGroupCommand::Import(args) =
            DatasourceCliArgs::parse_normalized_from(args).command
        else {
            unreachable!();
        };
        let result = crate::datasource::import_datasources_by_export_org(&args);
        done.store(true, Ordering::Release);
        let requests = server.join().unwrap();

        assert!(
            requests.iter().all(|request| request.starts_with("GET ")),
            "dry-run wrote: {requests:?}"
        );
        assert_eq!(result.unwrap(), 0);
        assert!(!requests.is_empty());
    }
}

#[test]
fn datasource_import_rejects_output_columns_without_table_output() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("datasources");
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(
        input_dir.join("datasources.json"),
        serde_json::to_string_pretty(&json!([])).unwrap(),
    )
    .unwrap();

    let error = run_datasource_cli(
        DatasourceCliArgs::parse_normalized_from([
            "grafana-util",
            "import",
            "--input-dir",
            input_dir.to_str().unwrap(),
            "--token",
            "token",
            "--dry-run",
            "--output-columns",
            "uid",
        ])
        .command,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("--output-columns is only supported with --dry-run --table"));
}

#[test]
fn datasource_import_rejects_use_export_org_interactive_review_output() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("datasources");
    fs::create_dir_all(&input_dir).unwrap();

    let error = run_datasource_cli(
        DatasourceCliArgs::parse_normalized_from([
            "grafana-util",
            "import",
            "--input-dir",
            input_dir.to_str().unwrap(),
            "--basic-user",
            "admin",
            "--basic-password",
            "admin",
            "--dry-run",
            "--use-export-org",
            "--output-format",
            "interactive",
        ])
        .command,
    )
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("Use datasource plan --use-export-org --output-format interactive"));
}

#[test]
fn datasource_import_rejects_extra_secret_or_server_managed_fields() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("datasources");
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(
        input_dir.join("export-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-datasource-export-index",
            "variant": "root",
            "scopeKind": "org-root",
            "resource": "datasource",
            "datasourcesFile": "datasources.json",
            "indexFile": "index.json",
            "datasourceCount": 1,
            "format": "grafana-datasource-inventory-v1"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        input_dir.join("datasources.json"),
        serde_json::to_string_pretty(&json!([{
            "uid": "prom-main",
            "name": "Prometheus Main",
            "type": "prometheus",
            "access": "proxy",
            "url": "http://prometheus:9090",
            "isDefault": true,
            "org": "Main Org.",
            "orgId": "1",
            "id": 7,
            "secureJsonData": {"httpHeaderValue1": "secret"}
        }]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        input_dir.join("index.json"),
        serde_json::to_string_pretty(&json!({"items": []})).unwrap(),
    )
    .unwrap();

    let error =
        load_import_records(&input_dir, DatasourceImportInputFormat::Inventory).unwrap_err();

    assert!(error
        .to_string()
        .contains("unsupported datasource field(s): id, secureJsonData"));
}

#[test]
fn datasource_import_loads_provisioning_from_export_root_without_metadata() {
    let temp = tempdir().unwrap();
    let export_root = temp.path().join("datasources");
    let provisioning_dir = export_root.join("provisioning");
    fs::create_dir_all(&provisioning_dir).unwrap();
    fs::write(
        provisioning_dir.join("datasources.yaml"),
        r#"apiVersion: 1
datasources:
  - uid: prom-main
    name: Prometheus Main
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    orgId: 7
"#,
    )
    .unwrap();

    let (metadata, records) =
        load_import_records(&export_root, DatasourceImportInputFormat::Provisioning).unwrap();

    assert_eq!(metadata.variant, "provisioning");
    assert_eq!(metadata.datasources_file, "provisioning/datasources.yaml");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].uid, "prom-main");
    assert_eq!(records[0].name, "Prometheus Main");
    assert_eq!(records[0].datasource_type, "prometheus");
    assert_eq!(records[0].access, "proxy");
    assert_eq!(records[0].url, "http://prometheus:9090");
    assert!(records[0].is_default);
    assert_eq!(records[0].org_id, "7");
}

#[test]
fn datasource_import_loads_provisioning_from_directory_or_yaml_file() {
    let temp = tempdir().unwrap();
    let provisioning_dir = temp.path().join("provisioning");
    fs::create_dir_all(&provisioning_dir).unwrap();
    let provisioning_file = provisioning_dir.join("datasources.yaml");
    fs::write(
        &provisioning_file,
        r#"apiVersion: 1
datasources:
  - uid: loki-main
    name: Loki Main
    type: loki
    access: proxy
    url: http://loki:3100
    isDefault: false
    orgId: 9
"#,
    )
    .unwrap();

    let (dir_metadata, dir_records) =
        load_import_records(&provisioning_dir, DatasourceImportInputFormat::Provisioning).unwrap();
    let (file_metadata, file_records) = load_import_records(
        &provisioning_file,
        DatasourceImportInputFormat::Provisioning,
    )
    .unwrap();

    assert_eq!(dir_metadata.datasources_file, "datasources.yaml");
    assert_eq!(file_metadata.datasources_file, "datasources.yaml");
    assert_eq!(dir_records.len(), 1);
    assert_eq!(dir_records[0].uid, "loki-main");
    assert_eq!(file_records, dir_records);
}

#[test]
fn datasource_import_loads_inventory_recovery_bundle_passthrough_fields() {
    let temp = tempdir().unwrap();
    let input_dir = temp.path().join("datasources");
    fs::create_dir_all(&input_dir).unwrap();
    fs::write(
        input_dir.join("export-metadata.json"),
        serde_json::to_string_pretty(&json!({
            "schemaVersion": 1,
            "kind": "grafana-utils-datasource-export-index",
            "variant": "root",
            "scopeKind": "org-root",
            "resource": "datasource",
            "datasourcesFile": "datasources.json",
            "indexFile": "index.json",
            "datasourceCount": 1,
            "format": "grafana-datasource-inventory-v1"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        input_dir.join("datasources.json"),
        serde_json::to_string_pretty(&json!([{
            "uid": "loki-main",
            "name": "Loki Main",
            "type": "loki",
            "access": "proxy",
            "url": "http://loki:3100",
            "isDefault": true,
            "org": "Main Org.",
            "orgId": 7,
            "basicAuth": true,
            "basicAuthUser": "loki-user",
            "database": "logs-main",
            "jsonData": {
                "httpMethod": "POST",
                "httpHeaderName1": "X-Scope-OrgID"
            },
            "secureJsonDataPlaceholders": {
                "basicAuthPassword": "${secret:loki-basic-auth}",
                "httpHeaderValue1": "${secret:loki-tenant-token}"
            },
            "user": "query-user",
            "withCredentials": true
        }]))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        input_dir.join("index.json"),
        serde_json::to_string_pretty(&json!({"items": []})).unwrap(),
    )
    .unwrap();

    let (_, records) =
        load_import_records(&input_dir, DatasourceImportInputFormat::Inventory).unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].uid, "loki-main");
    assert_eq!(records[0].org_id, "7");
    assert_eq!(records[0].basic_auth, Some(true));
    assert_eq!(records[0].basic_auth_user, "loki-user");
    assert_eq!(records[0].database, "logs-main");
    assert_eq!(records[0].user, "query-user");
    assert_eq!(records[0].with_credentials, Some(true));
    assert_eq!(
        records[0].json_data.as_ref().unwrap()["httpMethod"],
        json!("POST")
    );
    assert_eq!(
        records[0].json_data.as_ref().unwrap()["httpHeaderName1"],
        json!("X-Scope-OrgID")
    );
    assert_eq!(
        records[0].secure_json_data_placeholders.as_ref().unwrap()["basicAuthPassword"],
        json!("${secret:loki-basic-auth}")
    );
    assert_eq!(
        records[0].secure_json_data_placeholders.as_ref().unwrap()["httpHeaderValue1"],
        json!("${secret:loki-tenant-token}")
    );
}

#[test]
fn datasource_import_loads_provisioning_recovery_bundle_passthrough_fields() {
    let temp = tempdir().unwrap();
    let provisioning_dir = temp.path().join("provisioning");
    fs::create_dir_all(&provisioning_dir).unwrap();
    let provisioning_file = provisioning_dir.join("datasources.yaml");
    fs::write(
        &provisioning_file,
        r#"apiVersion: 1
datasources:
  - uid: loki-main
    name: Loki Main
    type: loki
    access: proxy
    url: http://loki:3100
    isDefault: false
    orgId: 9
    basicAuth: true
    basicAuthUser: loki-user
    database: logs-main
    user: query-user
    withCredentials: true
    jsonData:
      httpMethod: POST
      httpHeaderName1: X-Scope-OrgID
    secureJsonDataPlaceholders:
      basicAuthPassword: ${secret:loki-basic-auth}
      httpHeaderValue1: ${secret:loki-tenant-token}
"#,
    )
    .unwrap();

    let (_, records) = load_import_records(
        &provisioning_file,
        DatasourceImportInputFormat::Provisioning,
    )
    .unwrap();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].uid, "loki-main");
    assert_eq!(records[0].org_id, "9");
    assert_eq!(records[0].basic_auth, Some(true));
    assert_eq!(records[0].basic_auth_user, "loki-user");
    assert_eq!(records[0].database, "logs-main");
    assert_eq!(records[0].user, "query-user");
    assert_eq!(records[0].with_credentials, Some(true));
    assert_eq!(
        records[0].json_data.as_ref().unwrap()["httpMethod"],
        json!("POST")
    );
    assert_eq!(
        records[0].json_data.as_ref().unwrap()["httpHeaderName1"],
        json!("X-Scope-OrgID")
    );
    assert_eq!(
        records[0].secure_json_data_placeholders.as_ref().unwrap()["basicAuthPassword"],
        json!("${secret:loki-basic-auth}")
    );
    assert_eq!(
        records[0].secure_json_data_placeholders.as_ref().unwrap()["httpHeaderValue1"],
        json!("${secret:loki-tenant-token}")
    );
}
