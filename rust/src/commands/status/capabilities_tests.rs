use super::*;
use clap::{CommandFactory, Parser};
use serde_json::json;

#[derive(Parser)]
struct ParserFixture {
    #[command(flatten)]
    args: StatusCapabilitiesArgs,
}

fn list(version: &str) -> Value {
    json!({"apiVersion": format!("dashboard.grafana.app/{version}"), "kind":"DashboardList", "items":[]})
}

#[test]
fn capabilities_parser_requires_namespace_and_exposes_connection_output_flags() {
    assert!(ParserFixture::try_parse_from(["test"]).is_err());
    for output in ["text", "json", "yaml"] {
        let args = ParserFixture::try_parse_from([
            "test",
            "--namespace",
            "team-ops",
            "--output-format",
            output,
            "--org-id",
            "2",
        ])
        .unwrap();
        assert_eq!(args.args.namespace, "team-ops");
        assert_eq!(args.args.org_id, Some(2));
        assert_eq!(args.args.common.url, "");
    }
    assert!(ParserFixture::try_parse_from([
        "test",
        "--namespace",
        "default",
        "--insecure",
        "--verify-ssl"
    ])
    .is_err());
    assert!(ParserFixture::try_parse_from([
        "test",
        "--namespace",
        "default",
        "--output-format",
        "interactive"
    ])
    .is_err());
    let help = ParserFixture::command().render_long_help().to_string();
    for flag in [
        "--namespace",
        "--profile",
        "--token",
        "--basic-user",
        "--ca-cert",
        "--timeout",
        "--output-format",
    ] {
        assert!(help.contains(flag), "missing {flag}");
    }
}

#[test]
fn capabilities_probes_only_three_bounded_gets_and_validates_empty_lists() {
    let mut requests = Vec::new();
    let report = build_capabilities("team/ops", &mut |method, path, params, payload| {
        assert_eq!(method, Method::GET);
        assert!(payload.is_none());
        requests.push((path.to_string(), params.to_vec()));
        Ok(Some(if path == "/api/search" {
            json!([])
        } else if path.contains("/v1/") {
            list("v1")
        } else {
            list("v2")
        }))
    })
    .unwrap();
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[0].0, "/api/search");
    assert_eq!(
        requests[0].1,
        vec![
            ("type".into(), "dash-db".into()),
            ("limit".into(), "1".into()),
            ("page".into(), "1".into())
        ]
    );
    assert_eq!(
        requests[1].0,
        "/apis/dashboard.grafana.app/v1/namespaces/team%2Fops/dashboards"
    );
    assert_eq!(
        requests[2].0,
        "/apis/dashboard.grafana.app/v2/namespaces/team%2Fops/dashboards"
    );
    assert_eq!(requests[1].1, vec![("limit".into(), "1".into())]);
    assert_eq!(requests[2].1, requests[1].1);
    assert!(report
        .probes
        .iter()
        .all(|probe| probe.status == "available"));
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["namespace"], "team/ops");
    assert!(!render_text(&report).join("\n").contains("supported"));
    assert!(render_yaml(&report).unwrap().contains("probes:"));
}

#[test]
fn capabilities_classifies_http_errors_without_exposing_server_details() {
    for (code, expected) in [
        (401, "unauthorized"),
        (403, "forbidden"),
        (404, "not-served"),
        (429, "failed"),
        (500, "failed"),
        (503, "failed"),
    ] {
        let report = build_capabilities("default", &mut |_, _, _, _| {
            Err(crate::common::api_response(
                code,
                "https://secret:password@server",
                "token=secret",
            ))
        })
        .unwrap();
        assert_eq!(report.probes.len(), 3);
        assert!(report
            .probes
            .iter()
            .all(|probe| probe.status == expected && probe.http_status == Some(code)));
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("secret"));
        assert!(!json.contains("password"));
        assert!(json.contains("undetermined"));
    }
}

#[test]
fn capabilities_rejects_malformed_success_contracts_and_no_body() {
    for payload in [
        None,
        Some(json!(null)),
        Some(json!({})),
        Some(json!({"items":[]})),
        Some(json!("secret")),
    ] {
        let report = build_capabilities("default", &mut |_, _, _, _| Ok(payload.clone())).unwrap();
        assert!(report
            .probes
            .iter()
            .all(|probe| probe.status == "malformed-response"));
    }
    assert!(!valid_list("v1", Some(list("v2")), "default"));
    assert!(!valid_list("v2", Some(list("v1")), "default"));
    assert!(!valid_list(
        "classic",
        Some(json!([{"uid":"folder", "type":"dash-folder"}])),
        "default"
    ));
}

#[test]
fn capabilities_validates_resource_items_for_each_version_and_namespace() {
    for version in ["v1", "v2"] {
        let mut valid = list(version);
        valid["items"] = json!([{"apiVersion":format!("dashboard.grafana.app/{version}"), "kind":"Dashboard", "metadata":{"name":"example", "namespace":"default"}, "spec":{}}]);
        assert!(valid_list(version, Some(valid.clone()), "default"));
        for (pointer, invalid) in [
            ("/items/0", json!(null)),
            ("/items/0/apiVersion", json!("dashboard.grafana.app/v9")),
            ("/items/0/kind", json!("Folder")),
            ("/items/0/metadata/name", json!("")),
            ("/items/0/metadata/namespace", json!("other")),
            ("/items/0/spec", json!(null)),
        ] {
            let mut malformed = valid.clone();
            *malformed.pointer_mut(pointer).unwrap() = invalid;
            assert!(
                !valid_list(version, Some(malformed), "default"),
                "accepted {pointer} for {version}"
            );
        }
    }
}

#[test]
fn capabilities_transport_and_invalid_json_errors_remain_indeterminate() {
    let report = build_capabilities("default", &mut |_, _, _, _| {
        Err(message("private transport details"))
    })
    .unwrap();
    assert!(report
        .probes
        .iter()
        .all(|probe| probe.status == "failed" && probe.http_status.is_none()));
    assert!(!serde_json::to_string(&report).unwrap().contains("private"));
    let report = build_capabilities("default", &mut |_, _, _, _| {
        Err(serde_json::from_str::<Value>("not-json")
            .unwrap_err()
            .into())
    })
    .unwrap();
    assert!(report
        .probes
        .iter()
        .all(|probe| probe.status == "malformed-response"));
}

#[test]
fn capabilities_rejects_blank_namespace_before_requests() {
    for namespace in ["", " ", " default", "default "] {
        assert!(
            build_capabilities(namespace, &mut |_, _, _, _| panic!("must not request")).is_err()
        );
    }
}

#[test]
fn capabilities_http_fixture_preserves_errors_and_continues_after_denial() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let mut requests = Vec::new();
        for (status, body) in [("403 Forbidden", "secret server body"), ("200 OK", "not-json"), ("200 OK", "{\"apiVersion\":\"dashboard.grafana.app/v2\",\"kind\":\"DashboardList\",\"items\":[]}")] {
            let (mut socket, _) = listener.accept().unwrap();
            socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 1024];
            while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                let read = socket.read(&mut buffer).unwrap();
                assert!(read > 0);
                request.extend_from_slice(&buffer[..read]);
            }
            requests.push(String::from_utf8(request).unwrap());
            write!(socket, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        }
        requests
    });
    let client = crate::http::JsonHttpClient::new(crate::http::JsonHttpClientConfig {
        base_url: format!("http://{address}"),
        headers: vec![("X-Grafana-Org-Id".into(), "2".into())],
        timeout_secs: 5,
        verify_ssl: true,
    })
    .unwrap();
    let report = build_capabilities("default", &mut |method, path, params, payload| {
        client.request_json(method, path, params, payload)
    })
    .unwrap();
    assert_eq!(
        report
            .probes
            .iter()
            .map(|probe| probe.status)
            .collect::<Vec<_>>(),
        ["forbidden", "malformed-response", "available"]
    );
    assert_eq!(report.probes[0].http_status, Some(403));
    assert!(!serde_json::to_string(&report).unwrap().contains("secret"));
    let requests = server.join().unwrap();
    assert!(requests[0].starts_with("GET /api/search?type=dash-db&limit=1&page=1 HTTP/1.1"));
    assert!(requests[1].starts_with(
        "GET /apis/dashboard.grafana.app/v1/namespaces/default/dashboards?limit=1 HTTP/1.1"
    ));
    assert!(requests[2].starts_with(
        "GET /apis/dashboard.grafana.app/v2/namespaces/default/dashboards?limit=1 HTTP/1.1"
    ));
    assert!(requests
        .iter()
        .all(|request| request.to_ascii_lowercase().contains("x-grafana-org-id: 2")));
}
