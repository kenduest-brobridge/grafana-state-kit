use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use reqwest::Method;
use serde_json::{json, Map, Value};

use crate::alert::{CONTACT_POINT_KIND, MUTE_TIMING_KIND, RULE_KIND, TEMPLATE_KIND};
use crate::common::Result;
use crate::grafana_api::{GrafanaApiClient, GrafanaConnection};

const IDENTITY: &str = "ops/服務?filter#name %";
const ENCODED_IDENTITY: &str = "ops%2F%E6%9C%8D%E5%8B%99%3Ffilter%23name%20%25";

fn capture_wire_requests<F>(count: usize, exercise: F) -> Option<Vec<(String, Option<Value>)>>
where
    F: FnOnce(&GrafanaApiClient),
{
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => listener,
        Err(error) if error.kind() == ErrorKind::PermissionDenied => return None,
        Err(error) => panic!("failed to bind test listener: {error}"),
    };
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for _ in 0..count {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0_u8; 1024];
            let (header_end, body_size) = loop {
                let read = stream.read(&mut buffer).unwrap();
                assert!(read > 0, "request ended before headers");
                bytes.extend_from_slice(&buffer[..read]);
                if let Some(header_end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..header_end]);
                    let body_size = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap_or(0);
                    break (header_end + 4, body_size);
                }
            };
            while bytes.len() < header_end + body_size {
                let read = stream.read(&mut buffer).unwrap();
                assert!(read > 0, "request ended before body");
                bytes.extend_from_slice(&buffer[..read]);
            }
            let request_line = String::from_utf8_lossy(&bytes[..header_end])
                .lines()
                .next()
                .unwrap()
                .to_string();
            let body = (body_size > 0).then(|| {
                serde_json::from_slice(&bytes[header_end..header_end + body_size]).unwrap()
            });
            requests.push((request_line, body));
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").unwrap();
        }
        requests
    });
    let api = GrafanaApiClient::from_connection(GrafanaConnection::new(
        format!("http://{address}"),
        vec![],
        5,
        false,
        None,
        "unknown".to_string(),
    ))
    .unwrap();
    exercise(&api);
    Some(handle.join().unwrap())
}

#[test]
fn resource_uid_and_name_routes_encode_segments_on_wire() {
    let payload =
        json!({"uid": IDENTITY, "name": IDENTITY, "template": "{{ .Status }}", "version": "v3"});
    let Some(requests) = capture_wire_requests(14, |api| {
        let alert = api.alerting();
        let object = payload.as_object().unwrap();
        alert.get_dashboard(IDENTITY).unwrap();
        alert.get_alert_rule(IDENTITY).unwrap();
        alert.update_alert_rule(IDENTITY, object).unwrap();
        alert.delete_alert_rule(IDENTITY).unwrap();
        alert.update_contact_point(IDENTITY, object).unwrap();
        alert.delete_contact_point(IDENTITY).unwrap();
        alert.update_mute_timing(IDENTITY, object).unwrap();
        alert.delete_mute_timing(IDENTITY).unwrap();
        alert.get_template(IDENTITY).unwrap();
        alert.update_template(IDENTITY, object).unwrap();
        alert.delete_template(IDENTITY).unwrap();
        api.datasource()
            .update_datasource_by_uid(IDENTITY, object)
            .unwrap();
        api.datasource().delete_datasource_by_uid(IDENTITY).unwrap();
        crate::datasource::fetch_datasource_by_uid_if_exists(api.http_client(), IDENTITY).unwrap();
    }) else {
        return;
    };
    let expected = [
        ("GET", "/api/dashboards/uid", ""),
        ("GET", "/api/v1/provisioning/alert-rules", ""),
        ("PUT", "/api/v1/provisioning/alert-rules", ""),
        ("DELETE", "/api/v1/provisioning/alert-rules", ""),
        ("PUT", "/api/v1/provisioning/contact-points", ""),
        ("DELETE", "/api/v1/provisioning/contact-points", ""),
        ("PUT", "/api/v1/provisioning/mute-timings", ""),
        ("DELETE", "/api/v1/provisioning/mute-timings", "?version="),
        ("GET", "/api/v1/provisioning/templates", ""),
        ("PUT", "/api/v1/provisioning/templates", ""),
        ("DELETE", "/api/v1/provisioning/templates", "?version="),
        ("PUT", "/api/datasources/uid", ""),
        ("DELETE", "/api/datasources/uid", ""),
        ("GET", "/api/datasources/uid", ""),
    ];
    for ((request_line, body), (method, base, query)) in requests.iter().zip(expected) {
        assert_eq!(
            request_line,
            &format!("{method} {base}/{ENCODED_IDENTITY}{query} HTTP/1.1")
        );
        if let Some(body) = body {
            assert_eq!(body["uid"], json!(IDENTITY));
            if base.ends_with("templates") {
                assert!(body.get("name").is_none());
                assert_eq!(body["version"], json!("v3"));
            } else {
                assert_eq!(body["name"], json!(IDENTITY));
            }
        }
    }
}

fn alert_payloads() -> Vec<(&'static str, &'static str, Map<String, Value>)> {
    vec![
        (RULE_KIND, "alert-rules", json!({"uid": IDENTITY, "title": "CPU", "folderUID": "general", "ruleGroup": "default", "condition": "A", "data": []})),
        (CONTACT_POINT_KIND, "contact-points", json!({"uid": IDENTITY, "name": IDENTITY, "type": "email", "settings": {"addresses": "ops@example.com"}})),
        (MUTE_TIMING_KIND, "mute-timings", json!({"name": IDENTITY, "time_intervals": []})),
        (TEMPLATE_KIND, "templates", json!({"name": IDENTITY, "template": "{{ .Status }}"})),
    ].into_iter().map(|(kind, route, payload)| (kind, route, payload.as_object().unwrap().clone())).collect()
}

#[test]
fn alert_apply_routes_encode_identity_and_preserve_body() {
    use crate::grafana_api::alert_live::{
        apply_create_with_request, apply_delete_with_request, apply_update_with_request,
    };
    for (kind, route, payload) in alert_payloads() {
        let path = format!("/api/v1/provisioning/{route}/{ENCODED_IDENTITY}");
        let mut request = |method: Method,
                           actual: &str,
                           params: &[(String, String)],
                           body: Option<&Value>|
         -> Result<Option<Value>> {
            assert_eq!(actual, path);
            match method {
                Method::GET => Ok(Some(json!({"version": "current-version"}))),
                Method::PUT => {
                    let body = body.unwrap();
                    if kind == RULE_KIND || kind == CONTACT_POINT_KIND {
                        assert_eq!(body["uid"], json!(IDENTITY));
                    } else if kind == MUTE_TIMING_KIND {
                        assert_eq!(body["name"], json!(IDENTITY));
                    } else {
                        assert!(body.get("name").is_none());
                        assert_eq!(body["version"], json!("current-version"));
                    }
                    Ok(Some(json!({})))
                }
                Method::DELETE => {
                    if kind == TEMPLATE_KIND || kind == MUTE_TIMING_KIND {
                        assert_eq!(params, &[("version".to_string(), String::new())]);
                    } else {
                        assert!(params.is_empty());
                    }
                    Ok(None)
                }
                _ => panic!("unexpected request method"),
            }
        };
        apply_update_with_request(&mut request, kind, IDENTITY, &payload).unwrap();
        apply_delete_with_request(&mut request, kind, IDENTITY, false).unwrap();
        if kind == TEMPLATE_KIND {
            apply_create_with_request(
                &mut |method, actual, _params, body| {
                    assert_eq!(method, Method::PUT);
                    assert_eq!(actual, path);
                    assert_eq!(body.unwrap()["version"], json!(""));
                    Ok(None)
                },
                kind,
                &payload,
            )
            .unwrap();
        }
    }
}

#[test]
fn alert_import_and_compare_routes_encode_identity() {
    use crate::grafana_api::alert_live::{
        determine_import_action_with_request, fetch_live_compare_document_with_request,
        import_resource_document_with_request,
    };
    for (kind, route, payload) in alert_payloads() {
        let path = format!("/api/v1/provisioning/{route}/{ENCODED_IDENTITY}");
        let mut request = |method: Method,
                           actual: &str,
                           _params: &[(String, String)],
                           _body: Option<&Value>|
         -> Result<Option<Value>> {
            if method == Method::PUT || kind == RULE_KIND || kind == TEMPLATE_KIND {
                assert_eq!(actual, path);
            } else {
                assert_eq!(actual, format!("/api/v1/provisioning/{route}"));
            }
            let mut live = payload.clone();
            live.insert("version".to_string(), json!("current-version"));
            if method == Method::GET && kind != RULE_KIND && kind != TEMPLATE_KIND {
                Ok(Some(json!([live])))
            } else {
                Ok(Some(Value::Object(live)))
            }
        };
        assert!(
            fetch_live_compare_document_with_request(&mut request, kind, &payload)
                .unwrap()
                .is_some()
        );
        assert_eq!(
            determine_import_action_with_request(&mut request, kind, &payload, true).unwrap(),
            "would-update"
        );
        assert_eq!(
            import_resource_document_with_request(&mut request, kind, &payload, true)
                .unwrap()
                .0,
            "updated"
        );
    }
}
