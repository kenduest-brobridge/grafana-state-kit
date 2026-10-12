use grafana_utils_rust::alert::{build_alert_plan_with_request, RULE_KIND};
use reqwest::Method;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::tempdir;

#[test]
fn alert_apply_binary_reports_write_failures_and_exits_nonzero() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../tests/fixtures/alert_recreate_contract_cases.json"
    ))
    .unwrap();
    let source = &fixture["recreateCases"][0]["payload"];

    for failed_write in [1, 2] {
        let temp = tempdir().unwrap();
        let desired_dir = temp.path().join("desired");
        std::fs::create_dir_all(desired_dir.join("rules")).unwrap();
        let identities = ["first-rule", "second-rule", "third-rule"];
        let live = identities.map(|identity| {
            let mut rule = source.clone();
            rule["uid"] = json!(identity);
            let mut desired = rule.clone();
            desired["condition"] = json!("B");
            std::fs::write(
                desired_dir.join("rules").join(format!("{identity}.json")),
                serde_json::to_string(&desired).unwrap(),
            )
            .unwrap();
            rule
        });
        let plan = build_alert_plan_with_request(
            |method, path, _params, _payload| {
                assert_eq!(method, Method::GET);
                if let Some(identity) = path.strip_prefix("/api/v1/provisioning/alert-rules/") {
                    return Ok(live.iter().find(|rule| rule["uid"] == identity).cloned());
                }
                match path {
                    "/api/v1/provisioning/alert-rules" => Ok(Some(json!(live))),
                    "/api/v1/provisioning/policies" => Ok(None),
                    "/api/v1/provisioning/contact-points"
                    | "/api/v1/provisioning/mute-timings"
                    | "/api/v1/provisioning/templates" => Ok(Some(json!([]))),
                    _ => panic!("unexpected planning request {path}"),
                }
            },
            &desired_dir,
            false,
        )
        .unwrap();
        assert_eq!(plan["schemaVersion"], 1);
        assert_eq!(plan["summary"]["update"], 3);
        assert_eq!(plan["rows"][0]["kind"], RULE_KIND);
        let plan_path = temp.path().join("reviewed-plan.json");
        std::fs::write(&plan_path, serde_json::to_string(&plan).unwrap()).unwrap();

        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listener,
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                eprintln!("Skipping Alert apply HTTP fixture: localhost listener is denied.");
                return;
            }
            Err(error) => panic!("{error}"),
        };
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let done = Arc::new(AtomicBool::new(false));
        let server_done = Arc::clone(&done);
        let server = std::thread::spawn(move || {
            let mut requests = Vec::new();
            let mut writes = 0;
            let deadline = Instant::now() + Duration::from_secs(5);
            while !server_done.load(Ordering::Relaxed) && Instant::now() < deadline {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(error) => panic!("{error}"),
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                let parts = request_line.split_whitespace().collect::<Vec<_>>();
                let method = parts[0];
                let path = parts[1];
                let mut content_length = 0;
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    if header == "\r\n" || header.is_empty() {
                        break;
                    }
                    if let Some(length) = header.to_lowercase().strip_prefix("content-length:") {
                        content_length = length.trim().parse::<usize>().unwrap();
                    }
                }
                let mut payload = vec![0; content_length];
                reader.read_exact(&mut payload).unwrap();
                requests.push(format!("{method} {path}"));
                let identity = path.rsplit('/').next().unwrap();
                let (status, response) = match method {
                    "GET" => (
                        "200 OK",
                        live.iter()
                            .find(|rule| rule["uid"] == identity)
                            .unwrap()
                            .clone(),
                    ),
                    "PUT" => {
                        writes += 1;
                        let payload: Value = serde_json::from_slice(&payload).unwrap();
                        assert_eq!(payload["uid"], identity);
                        assert_eq!(payload["condition"], "B");
                        if writes == failed_write {
                            (
                                "503 Service Unavailable",
                                json!({"message": "secret-response-body"}),
                            )
                        } else {
                            ("200 OK", json!({"uid": identity}))
                        }
                    }
                    _ => panic!("unexpected apply request {method} {path}"),
                };
                let body = serde_json::to_string(&response).unwrap();
                write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
            requests
        });
        let output = Command::new(env!("CARGO_BIN_EXE_grafana-util"))
            .args(["alert", "apply", "--url", &url, "--token", "test-token"])
            .arg("--plan-file")
            .arg(&plan_path)
            .args(["--approve", "--output-format", "json"])
            .output()
            .unwrap();
        done.store(true, Ordering::Relaxed);
        let requests = server.join().unwrap();
        assert_eq!(output.status.code(), Some(1));
        let result: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(result["appliedCount"], failed_write - 1);
        assert_eq!(result["failedCount"], 1);
        assert_eq!(result["notAttemptedCount"], 3 - failed_write);
        assert_eq!(
            result["status"],
            if failed_write == 1 {
                "failed"
            } else {
                "partial"
            }
        );
        assert_eq!(requests.len(), 3 + failed_write);
        assert!(requests[..3]
            .iter()
            .all(|request| request.starts_with("GET ")));
        assert_eq!(result["results"][failed_write - 1]["status"], "failed");
        assert_eq!(
            result["results"][failed_write - 1]["identity"],
            identities[failed_write - 1]
        );
        for result in result["results"]
            .as_array()
            .unwrap()
            .iter()
            .skip(failed_write)
        {
            assert_eq!(result["status"], "not-attempted");
        }
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(identities[failed_write - 1]));
        assert!(!result.to_string().contains("secret-response-body"));
        assert!(!stderr.contains("secret-response-body"));
    }
}
