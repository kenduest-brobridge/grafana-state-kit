use super::super::{alert_runtime_support, render_alert_action_text};
use super::{
    build_alert_plan_document, controlled_rule_update_row, execute_alert_plan_with_request,
    RULE_KIND,
};
use crate::common::api_response;
use reqwest::Method;
use serde_json::{json, Value};

#[test]
fn controlled_alert_apply_changed_fresh_baseline_sends_no_writes() {
    let first = controlled_rule_update_row("first-rule");
    let second = controlled_rule_update_row("second-rule");
    let mut changed_live = second["live"].clone();
    changed_live["title"] = json!("Changed after review");
    let plan = build_alert_plan_document(&[first.clone(), second], false);
    let mut calls = Vec::new();
    let error = execute_alert_plan_with_request(
        |method, path, _params, _payload| {
            calls.push((method.clone(), path.to_string()));
            match (method, path) {
                (Method::GET, "/api/v1/provisioning/alert-rules/first-rule") => {
                    Ok(Some(first["live"].clone()))
                }
                (Method::GET, "/api/v1/provisioning/alert-rules/second-rule") => {
                    Ok(Some(changed_live.clone()))
                }
                (Method::PUT, _) => Ok(Some(json!({"uid": "written-rule"}))),
                _ => panic!("unexpected request {path}"),
            }
        },
        &plan,
        false,
    )
    .unwrap_err();

    assert!(error.to_string().contains("baseline changed"));
    assert!(error.to_string().contains("second-rule"));
    assert_eq!(calls.len(), 2);
    assert!(calls.iter().all(|(method, _)| *method == Method::GET));
}

#[test]
fn controlled_alert_apply_reads_all_unchanged_baselines_before_first_write() {
    let first = controlled_rule_update_row("first-rule");
    let second = controlled_rule_update_row("second-rule");
    let mut first_live = first["live"].clone();
    first_live["id"] = json!(42);
    first_live["updated"] = json!("2026-10-12T00:00:00Z");
    first_live["provenance"] = json!("api");
    first_live["isPaused"] = json!(false);
    first_live["for"] = json!("0s");
    let plan = build_alert_plan_document(&[first.clone(), second.clone()], false);
    let mut calls = Vec::new();
    let result = execute_alert_plan_with_request(
        |method, path, params, payload| {
            assert!(params.is_empty());
            calls.push((method.clone(), path.to_string()));
            match (method, path) {
                (Method::GET, "/api/v1/provisioning/alert-rules/first-rule") => {
                    assert!(payload.is_none());
                    Ok(Some(first_live.clone()))
                }
                (Method::GET, "/api/v1/provisioning/alert-rules/second-rule") => {
                    assert!(payload.is_none());
                    Ok(Some(second["live"].clone()))
                }
                (Method::PUT, "/api/v1/provisioning/alert-rules/first-rule") => {
                    assert_eq!(payload, Some(&first["desired"]));
                    Ok(Some(json!({"uid": "first-rule"})))
                }
                (Method::PUT, "/api/v1/provisioning/alert-rules/second-rule") => {
                    assert_eq!(payload, Some(&second["desired"]));
                    Ok(Some(json!({"uid": "second-rule"})))
                }
                _ => panic!("unexpected request {path}"),
            }
        },
        &plan,
        false,
    )
    .unwrap();

    assert_eq!(
        calls,
        [
            (
                Method::GET,
                "/api/v1/provisioning/alert-rules/first-rule".to_string()
            ),
            (
                Method::GET,
                "/api/v1/provisioning/alert-rules/second-rule".to_string()
            ),
            (
                Method::PUT,
                "/api/v1/provisioning/alert-rules/first-rule".to_string()
            ),
            (
                Method::PUT,
                "/api/v1/provisioning/alert-rules/second-rule".to_string()
            ),
        ]
    );
    assert_eq!(result["status"], "completed");
    assert_eq!(result["appliedCount"], 2);
    assert_eq!(result["failedCount"], 0);
    assert_eq!(result["notAttemptedCount"], 0);
    assert_eq!(result["results"][0]["status"], "completed");
    assert_eq!(result["results"][1]["status"], "completed");
}

#[test]
fn controlled_alert_apply_missing_fresh_baseline_sends_no_writes() {
    let first = controlled_rule_update_row("first-rule");
    let second = controlled_rule_update_row("second-rule");
    let plan = build_alert_plan_document(&[first.clone(), second], false);
    let mut calls = Vec::new();
    let error = execute_alert_plan_with_request(
        |method, path, _params, _payload| {
            calls.push((method.clone(), path.to_string()));
            match (method, path) {
                (Method::GET, "/api/v1/provisioning/alert-rules/first-rule") => {
                    Ok(Some(first["live"].clone()))
                }
                (Method::GET, "/api/v1/provisioning/alert-rules/second-rule") => {
                    Err(api_response(404, "http://grafana/second-rule", "missing"))
                }
                _ => panic!("unexpected request {path}"),
            }
        },
        &plan,
        false,
    )
    .unwrap_err();

    assert!(error.to_string().contains("baseline changed or is missing"));
    assert!(error.to_string().contains("second-rule"));
    assert_eq!(calls.len(), 2);
    assert!(calls.iter().all(|(method, _)| *method == Method::GET));
}

#[test]
fn controlled_alert_apply_failed_fresh_read_sends_no_writes_or_raw_error() {
    let plan = build_alert_plan_document(&[controlled_rule_update_row("first-rule")], false);
    let mut calls = Vec::new();
    let error = execute_alert_plan_with_request(
        |method, path, _params, _payload| {
            calls.push((method, path.to_string()));
            Err(api_response(
                401,
                "http://grafana/secret-token",
                "secret-body",
            ))
        },
        &plan,
        false,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("first-rule"));
    assert!(error.contains("HTTP 401"));
    assert!(error.contains("No updates were written"));
    assert!(!error.contains("secret"));
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, Method::GET);
}

fn run_controlled_alert_apply_failure(failed_index: usize) -> (Value, String) {
    let identities = ["first-rule", "second-rule", "third-rule"];
    let rows = identities.map(controlled_rule_update_row);
    let plan = build_alert_plan_document(&rows, false);
    let mut calls = Vec::new();
    let outcome = alert_runtime_support::execute_alert_plan_report_with_request(
        |method, path, _params, _payload| {
            calls.push((method.clone(), path.to_string()));
            let identity = path.rsplit('/').next().unwrap();
            let row = rows.iter().find(|row| row["identity"] == identity).unwrap();
            match method {
                Method::GET => Ok(Some(row["live"].clone())),
                Method::PUT if identity == identities[failed_index] => Err(api_response(
                    500,
                    "http://grafana/secret-token",
                    "secret-response-body",
                )),
                Method::PUT => Ok(Some(json!({"uid": identity}))),
                _ => panic!("unexpected request {path}"),
            }
        },
        &plan,
        false,
    )
    .unwrap();
    let result = outcome.document.clone();
    let error = outcome.into_result().unwrap_err().to_string();

    assert_eq!(calls.len(), identities.len() + failed_index + 1);
    assert!(calls[..identities.len()]
        .iter()
        .all(|(method, _)| *method == Method::GET));
    assert!(calls[identities.len()..]
        .iter()
        .all(|(method, _)| *method == Method::PUT));
    assert_eq!(result["appliedCount"], failed_index);
    assert_eq!(result["failedCount"], 1);
    assert_eq!(
        result["notAttemptedCount"],
        identities.len() - failed_index - 1
    );
    assert_eq!(
        result["results"].as_array().unwrap().len(),
        identities.len()
    );
    assert_eq!(result["results"][failed_index]["error"]["statusCode"], 500);
    assert!(!result.to_string().contains("secret"));
    assert!(!error.contains("secret"));
    (result, error)
}

#[test]
fn controlled_alert_apply_second_write_failure_reports_exact_partial_results() {
    let (result, error) = run_controlled_alert_apply_failure(1);

    assert_eq!(result["status"], "partial");
    assert_eq!(
        result["results"],
        json!([
            {"kind": RULE_KIND, "identity": "first-rule", "action": "update", "status": "completed", "response": {"uid": "first-rule"}},
            {"kind": RULE_KIND, "identity": "second-rule", "action": "update", "status": "failed", "response": null, "error": {
                "kind": "api-response", "statusCode": 500, "message": "Update request failed. Verify current live state before retrying."
            }},
            {"kind": RULE_KIND, "identity": "third-rule", "action": "update", "status": "not-attempted", "response": null}
        ])
    );
    assert!(error
        .contains("Completed: [first-rule]; failed: [second-rule]; not-attempted: [third-rule]"));
    let text = render_alert_action_text("Alert apply", &result).join("\n");
    assert!(text.contains("status=partial completed=1 failed=1 not-attempted=1"));
    assert!(text.contains("first-rule action=update status=completed"));
    assert!(text.contains("second-rule action=update status=failed"));
    assert!(text.contains("third-rule action=update status=not-attempted"));
}

#[test]
fn controlled_alert_apply_first_write_failure_reports_no_completed_updates() {
    let (result, error) = run_controlled_alert_apply_failure(0);

    assert_eq!(result["status"], "failed");
    assert_eq!(result["results"][0]["identity"], "first-rule");
    assert_eq!(result["results"][0]["status"], "failed");
    assert_eq!(result["results"][1]["identity"], "second-rule");
    assert_eq!(result["results"][1]["status"], "not-attempted");
    assert_eq!(result["results"][2]["identity"], "third-rule");
    assert_eq!(result["results"][2]["status"], "not-attempted");
    assert!(error
        .contains("Completed: []; failed: [first-rule]; not-attempted: [second-rule, third-rule]"));
}

#[test]
fn controlled_alert_apply_rejects_unsupported_stored_plan_contract_before_requests() {
    let base = build_alert_plan_document(&[controlled_rule_update_row("first-rule")], false);
    let mut no_version = base.clone();
    no_version.as_object_mut().unwrap().remove("schemaVersion");
    let mut unsupported_version = base.clone();
    unsupported_version["schemaVersion"] = json!(2);
    let mut unknown_action = base.clone();
    unknown_action["rows"][0]["action"] = json!("upsert");
    for plan in [no_version, unsupported_version, unknown_action] {
        let mut calls = 0;
        let error = execute_alert_plan_with_request(
            |_method, _path, _params, _payload| {
                calls += 1;
                Ok(None)
            },
            &plan,
            false,
        )
        .unwrap_err();
        assert!(error.to_string().contains("not supported"));
        assert_eq!(calls, 0);
    }
}

#[test]
fn controlled_alert_apply_rejects_duplicate_rule_identity_before_requests() {
    let row = controlled_rule_update_row("first-rule");
    let plan = build_alert_plan_document(&[row.clone(), row], false);
    let error = execute_alert_plan_with_request(
        |_method, _path, _params, _payload| panic!("validation must precede requests"),
        &plan,
        false,
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("Duplicate alert plan rule identity"));
}
