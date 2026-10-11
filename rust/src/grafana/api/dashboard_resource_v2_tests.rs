use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::grafana_api::{AuthInputs, GrafanaApiClient, GrafanaConnection};
use serde_json::json;

type TestServer = (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>);

fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn spawn_server(responses: Vec<String>) -> Option<TestServer> {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(error) if error.kind() == ErrorKind::PermissionDenied => return None,
        Err(error) => panic!("failed to bind test listener: {error}"),
    };
    let address = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&requests);
    let handle = thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let count = stream.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..count]);
                let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n")
                else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&request[..header_end + 4]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("content-length: ")
                            .or_else(|| line.strip_prefix("Content-Length: "))
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
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    Some((format!("http://{address}"), requests, handle))
}

fn test_api(base_url: String) -> GrafanaApiClient {
    GrafanaApiClient::from_connection(
        GrafanaConnection::resolve(
            None,
            crate::profile_config::ConnectionMergeInput {
                url: &base_url,
                url_default: "http://localhost:3000",
                api_token: Some("token"),
                username: None,
                password: None,
                org_id: None,
                timeout: 5,
                timeout_default: 5,
                verify_ssl: false,
                insecure: false,
                ca_cert: None::<&std::path::Path>,
            },
            AuthInputs {
                api_token: Some("token"),
                username: None,
                password: None,
                prompt_password: false,
                prompt_token: false,
            },
            false,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn v2_probe_encodes_namespace_as_one_path_segment() {
    let Some((base_url, requests, handle)) = spawn_server(vec![http_response(
        "200 OK",
        r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"DashboardList","items":[],"metadata":{}}"#,
    )]) else {
        return;
    };
    let api = test_api(base_url);

    assert!(api
        .dashboard()
        .probe_dashboard_resource_v2("team/ops space")
        .unwrap());
    handle.join().unwrap();

    let requests = requests.lock().unwrap();
    assert!(requests[0].starts_with(
        "GET /apis/dashboard.grafana.app/v2/namespaces/team%2Fops%20space/dashboards?limit=1 "
    ));
}

#[test]
fn v2_probe_rejects_a_collection_without_items() {
    let Some((base_url, _, handle)) = spawn_server(vec![http_response(
        "200 OK",
        r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"DashboardList","metadata":{}}"#,
    )]) else {
        return;
    };
    let api = test_api(base_url);

    let error = api
        .dashboard()
        .probe_dashboard_resource_v2("default")
        .expect_err("dashboard discovery must validate the collection items field");
    handle.join().unwrap();

    assert!(error.to_string().contains("items array"));
}

#[test]
fn v2_probe_treats_only_not_found_as_unsupported() {
    let Some((base_url, _, handle)) = spawn_server(vec![
        http_response("404 Not Found", r#"{"message":"not found"}"#),
        http_response("401 Unauthorized", r#"{"message":"unauthorized"}"#),
        http_response("403 Forbidden", r#"{"message":"forbidden"}"#),
    ]) else {
        return;
    };
    let api = test_api(base_url);

    assert!(!api
        .dashboard()
        .probe_dashboard_resource_v2("default")
        .unwrap());
    assert_eq!(
        api.dashboard()
            .probe_dashboard_resource_v2("default")
            .unwrap_err()
            .status_code(),
        Some(401)
    );
    assert_eq!(
        api.dashboard()
            .probe_dashboard_resource_v2("default")
            .unwrap_err()
            .status_code(),
        Some(403)
    );
    handle.join().unwrap();
}

#[test]
fn v2_get_if_exists_encodes_identity_and_returns_none_only_for_404() {
    let Some((base_url, requests, handle)) = spawn_server(vec![
        http_response(
            "200 OK",
            r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard","metadata":{"namespace":"team/ops","name":"cpu/main one"},"spec":{"title":"CPU"}}"#,
        ),
        http_response("404 Not Found", r#"{"message":"not found"}"#),
    ]) else {
        return;
    };
    let api = test_api(base_url);

    let found = api
        .dashboard()
        .fetch_dashboard_resource_v2_if_exists("team/ops", "cpu/main one")
        .unwrap()
        .unwrap();
    let missing = api
        .dashboard()
        .fetch_dashboard_resource_v2_if_exists("team/ops", "missing")
        .unwrap();
    handle.join().unwrap();

    assert_eq!(found["metadata"]["name"], "cpu/main one");
    assert!(missing.is_none());
    let requests = requests.lock().unwrap();
    assert!(requests[0].starts_with(
        "GET /apis/dashboard.grafana.app/v2/namespaces/team%2Fops/dashboards/cpu%2Fmain%20one "
    ));
}

#[test]
fn v2_get_rejects_malformed_or_mismatched_resource_responses() {
    let Some((base_url, _, handle)) = spawn_server(vec![
        http_response("200 OK", r#"[]"#),
        http_response(
            "200 OK",
            r#"{"apiVersion":"dashboard.grafana.app/v1","kind":"Dashboard","metadata":{"namespace":"default","name":"cpu"},"spec":{}}"#,
        ),
        http_response(
            "200 OK",
            r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"Folder","metadata":{"namespace":"default","name":"cpu"},"spec":{}}"#,
        ),
        http_response(
            "200 OK",
            r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard","metadata":{"namespace":"default","name":"memory"},"spec":{}}"#,
        ),
    ]) else {
        return;
    };
    let api = test_api(base_url);

    for _ in 0..4 {
        let error = api
            .dashboard()
            .fetch_dashboard_resource_v2_if_exists("default", "cpu")
            .unwrap_err();
        assert!(error.to_string().contains("cpu"));
    }
    handle.join().unwrap();
}

#[test]
fn v2_create_posts_collection_payload_and_validates_response() {
    let response = r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard","metadata":{"namespace":"team/ops","name":"cpu/main"},"spec":{"title":"CPU"}}"#;
    let Some((base_url, requests, handle)) =
        spawn_server(vec![http_response("201 Created", response)])
    else {
        return;
    };
    let api = test_api(base_url);
    let payload = json!({
        "apiVersion": "dashboard.grafana.app/v2",
        "kind": "Dashboard",
        "metadata": {"name": "cpu/main"},
        "spec": {"title": "CPU"}
    });

    let created = api
        .dashboard()
        .create_dashboard_resource_v2("team/ops", &payload)
        .unwrap();
    handle.join().unwrap();

    assert_eq!(created["metadata"]["name"], "cpu/main");
    let requests = requests.lock().unwrap();
    assert!(requests[0]
        .starts_with("POST /apis/dashboard.grafana.app/v2/namespaces/team%2Fops/dashboards "));
    assert!(requests[0].contains(r#""title":"CPU""#));
}

#[test]
fn v2_update_puts_named_payload_and_preserves_conflict_status() {
    let response = r#"{"apiVersion":"dashboard.grafana.app/v2","kind":"Dashboard","metadata":{"namespace":"team/ops","name":"cpu/main"},"spec":{"title":"CPU v2"}}"#;
    let Some((base_url, requests, handle)) = spawn_server(vec![
        http_response("200 OK", response),
        http_response("409 Conflict", r#"{"message":"resource version conflict"}"#),
    ]) else {
        return;
    };
    let api = test_api(base_url);
    let payload = json!({
        "apiVersion": "dashboard.grafana.app/v2",
        "kind": "Dashboard",
        "metadata": {"name": "cpu/main", "resourceVersion": "9"},
        "spec": {"title": "CPU v2"}
    });

    api.dashboard()
        .update_dashboard_resource_v2("team/ops", "cpu/main", &payload)
        .unwrap();
    let conflict = api
        .dashboard()
        .update_dashboard_resource_v2("team/ops", "cpu/main", &payload)
        .unwrap_err();
    handle.join().unwrap();

    assert_eq!(conflict.status_code(), Some(409));
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].starts_with(
        "PUT /apis/dashboard.grafana.app/v2/namespaces/team%2Fops/dashboards/cpu%2Fmain "
    ));
    assert!(requests[1].starts_with("PUT "));
}

#[test]
fn classic_dashboard_api_encodes_reserved_uid_delimiters() {
    let uid = "cpu/main?orgId=2#frag %one";
    let Some((base_url, requests, handle)) = spawn_server(vec![
        http_response("200 OK", &json!({"dashboard": {"uid": uid}}).to_string()),
        http_response("200 OK", &json!({"uid": uid, "title": "Infra"}).to_string()),
        http_response("200 OK", &json!({"result": {"uid": uid}}).to_string()),
        http_response("200 OK", "[]"),
        http_response("200 OK", "[]"),
        http_response("200 OK", "{}"),
        http_response("200 OK", "{}"),
        http_response("200 OK", "{}"),
        http_response("200 OK", r#"[{"created":"2026-10-11T00:00:00Z"}]"#),
    ]) else {
        return;
    };
    let api = test_api(base_url);
    let dashboard = api.dashboard();
    assert_eq!(
        dashboard.fetch_dashboard(uid).unwrap()["dashboard"]["uid"],
        uid
    );
    dashboard.fetch_folder_if_exists(uid).unwrap();
    dashboard.fetch_library_panel(uid).unwrap();
    dashboard.fetch_dashboard_permissions(uid).unwrap();
    dashboard.fetch_folder_permissions(uid).unwrap();
    dashboard
        .update_folder_request(
            uid,
            &json!({"uid": uid, "title": "Infra"})
                .as_object()
                .unwrap()
                .clone(),
        )
        .unwrap();
    dashboard.delete_dashboard_request(uid).unwrap();
    dashboard.delete_folder_request(uid).unwrap();
    assert_eq!(
        dashboard.latest_dashboard_version_timestamp(&[json!({"uid": uid})
            .as_object()
            .unwrap()
            .clone()]),
        Some("2026-10-11T00:00:00Z".to_string())
    );
    handle.join().unwrap();

    let encoded_uid = "cpu%2Fmain%3ForgId%3D2%23frag%20%25one";
    let expected = [
        format!("GET /api/dashboards/uid/{encoded_uid} "),
        format!("GET /api/folders/{encoded_uid} "),
        format!("GET /api/library-elements/{encoded_uid} "),
        format!("GET /api/dashboards/uid/{encoded_uid}/permissions "),
        format!("GET /api/folders/{encoded_uid}/permissions "),
        format!("PUT /api/folders/{encoded_uid} "),
        format!("DELETE /api/dashboards/uid/{encoded_uid} "),
        format!("DELETE /api/folders/{encoded_uid} "),
        format!("GET /api/dashboards/uid/{encoded_uid}/versions?limit=1 "),
    ];
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), expected.len());
    for (request, expected) in requests.iter().zip(expected) {
        assert!(request.starts_with(&expected), "{request}");
    }
    assert!(requests[5].contains(&json!({"uid": uid, "title": "Infra"}).to_string()));
}
