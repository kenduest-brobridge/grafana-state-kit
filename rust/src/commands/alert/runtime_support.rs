//! Build alert compare/import/delete plan documents from live Grafana payloads.
//!
//! Responsibilities:
//! - Gather and normalize alert resources through shared request helpers.
//! - Produce plan and delete-preview documents used by diff/import execution flows.
//! - Preserve request semantics so CLI/runtime callers receive a stable sync-ready
//!   shape across execution paths.

use crate::common::{message, tool_version, value_as_object, Result};
use crate::review_contract::{review_apply_result_entry, ReviewApplyResult};
use reqwest::Method;
use serde_json::{json, Map, Value};

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::alert_support::{
    build_contact_point_scaffold_document, build_managed_policy_route_preview,
    normalize_compare_payload, remove_managed_policy_subtree, upsert_managed_policy_subtree,
};
#[allow(unused_imports)]
use super::{
    build_compare_document, build_contact_point_import_payload, build_import_operation,
    build_mute_timing_import_payload, build_new_contact_point_scaffold_document,
    build_new_rule_scaffold_document, build_new_template_scaffold_document,
    build_policies_import_payload, build_resource_identity, build_rule_import_payload,
    build_template_import_payload, discover_alert_resource_files, init_alert_managed_dir,
    load_alert_resource_file, resource_subdir_by_kind, strip_server_managed_fields,
    write_alert_resource_file, CONTACT_POINT_KIND, MUTE_TIMING_KIND, POLICIES_KIND, RULE_KIND,
    TEMPLATE_KIND,
};

pub const ALERT_MANAGED_POLICY_PREVIEW_SCHEMA_VERSION: i64 = 1;
type AlertDesiredOperation = (PathBuf, String, Map<String, Value>);

#[allow(unused_imports)]
pub use super::alert_runtime_plan_document::{
    build_alert_delete_preview_document, build_alert_diff_document,
    build_alert_import_dry_run_document, build_alert_plan_document, ALERT_DELETE_PREVIEW_KIND,
    ALERT_DELETE_PREVIEW_SCHEMA_VERSION, ALERT_IMPORT_DRY_RUN_KIND,
    ALERT_IMPORT_DRY_RUN_SCHEMA_VERSION, ALERT_PLAN_KIND, ALERT_PLAN_SCHEMA_VERSION,
};
pub(crate) use super::alert_runtime_plan_document::{
    build_plan_row, build_rule_review_hints, runtime_schema, PlanRowInput,
};
#[allow(unused_imports)]
pub(crate) use super::alert_runtime_review::{
    build_alert_plan_review_envelope, build_alert_plan_review_projection, AlertPlanReviewProjection,
};

fn row_object<'a>(row: &'a Value, label: &str) -> Result<&'a Map<String, Value>> {
    value_as_object(row, label)
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[allow(unused_imports)]
use crate::grafana_api::alert_live::{
    apply_create_with_request, apply_delete_with_request, apply_update_with_request,
    fetch_live_compare_document_with_request, request_array_with_request,
    request_live_resources_by_kind_with_request, request_optional_object_with_request,
};

pub fn load_alert_desired_operations(dir: &Path) -> Result<Vec<AlertDesiredOperation>> {
    let resource_files = discover_alert_resource_files(dir)?;
    let mut operations = Vec::new();
    for path in resource_files {
        let document = load_alert_resource_file(&path, "Alerting resource")?;
        let (kind, payload) = build_import_operation(&document)?;
        operations.push((path, kind, payload));
    }
    Ok(operations)
}

pub fn build_alert_plan_with_request<F>(
    mut request_json: F,
    desired_dir: &Path,
    allow_prune: bool,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let desired_operations = load_alert_desired_operations(desired_dir)?;
    let mut rows = Vec::new();
    let mut desired_keys = BTreeSet::new();

    for (path, kind, payload) in desired_operations {
        let identity = build_resource_identity(&kind, &payload);
        let key = (kind.clone(), identity.clone());
        if !desired_keys.insert(key.clone()) {
            return Err(message(format!(
                "Duplicate alert desired identity detected for kind={} id={}.",
                kind, identity
            )));
        }
        let review_hints = if kind == RULE_KIND {
            build_rule_review_hints(&payload)
        } else {
            Vec::new()
        };
        let desired_compare =
            build_compare_document(&kind, &normalize_compare_payload(&kind, &payload));
        let live_compare =
            fetch_live_compare_document_with_request(&mut request_json, &kind, &payload)?;
        let action = match live_compare.as_ref() {
            None => "create",
            Some(live) if live == &desired_compare => "noop",
            Some(_) => "update",
        };
        let row_reason = match action {
            "create" => "missing-live",
            "noop" => "in-sync",
            "update" => "drift-detected",
            _ => unreachable!(),
        };
        let live_payload = live_compare.as_ref().and_then(Value::as_object);
        rows.push(build_plan_row(PlanRowInput {
            kind: &kind,
            identity: &identity,
            path: Some(&path),
            action,
            reason: row_reason,
            blocked_reason: None,
            desired: Some(&payload),
            live: live_payload,
            review_hints,
        }));
    }

    for kind in resource_subdir_by_kind().keys() {
        let mut live_items = request_live_resources_by_kind_with_request(&mut request_json, kind)?
            .into_iter()
            .map(|item| {
                let payload = strip_server_managed_fields(kind, &item);
                let identity = build_resource_identity(kind, &payload);
                (identity, payload)
            })
            .collect::<Vec<(String, Map<String, Value>)>>();
        live_items.sort_by(|left, right| left.0.cmp(&right.0));
        for (identity, payload) in live_items {
            if desired_keys.contains(&(kind.to_string(), identity.clone())) {
                continue;
            }
            let action = if allow_prune { "delete" } else { "blocked" };
            let reason = if allow_prune {
                "missing-from-desired-state"
            } else {
                "prune-required"
            };
            let blocked_reason = if allow_prune {
                None
            } else {
                Some("prune-required")
            };
            rows.push(build_plan_row(PlanRowInput {
                kind,
                identity: &identity,
                path: None,
                action,
                reason,
                blocked_reason,
                desired: None,
                live: Some(&payload),
                review_hints: Vec::new(),
            }));
        }
    }

    Ok(build_alert_plan_document(&rows, allow_prune))
}

pub fn build_alert_delete_preview_from_files(
    resource_files: &[PathBuf],
    allow_policy_reset: bool,
) -> Result<Value> {
    let mut rows = Vec::new();
    for path in resource_files {
        let document = load_alert_resource_file(path, "Alerting delete target")?;
        let (kind, payload) = build_import_operation(&document)?;
        let identity = build_resource_identity(&kind, &payload);
        let blocked = kind == POLICIES_KIND && !allow_policy_reset;
        rows.push(json!({
            (runtime_schema::row::PATH): path_string(path),
            (runtime_schema::row::KIND): kind,
            (runtime_schema::row::IDENTITY): identity,
            (runtime_schema::row::ACTION): if blocked { "blocked" } else { "delete" },
            (runtime_schema::row::REASON): if blocked {
                "policy-reset-requires-allow-policy-reset"
            } else {
                "explicit-delete-request"
            },
            (runtime_schema::row::DESIRED): Value::Object(payload),
        }));
    }
    Ok(build_alert_delete_preview_document(
        &rows,
        allow_policy_reset,
    ))
}

pub fn build_alert_delete_preview_from_dir(
    desired_dir: &Path,
    allow_policy_reset: bool,
) -> Result<Value> {
    build_alert_delete_preview_from_files(
        &discover_alert_resource_files(desired_dir)?,
        allow_policy_reset,
    )
}

fn payload_object_from_row<'a>(
    row: &'a Map<String, Value>,
    field: &str,
) -> Result<&'a Map<String, Value>> {
    row.get(field)
        .ok_or_else(|| message(format!("Alert plan row is missing {field}.")))
        .and_then(|value| value_as_object(value, &format!("Alert plan row field {field}")))
}

pub fn execute_alert_plan_with_request<F>(
    mut request_json: F,
    plan_document: &Value,
    allow_policy_reset: bool,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    let plan = value_as_object(plan_document, "Alert plan document")?;
    if plan
        .get(runtime_schema::document::KIND)
        .and_then(Value::as_str)
        != Some(ALERT_PLAN_KIND)
    {
        return Err(message("Alert plan document kind is not supported."));
    }
    let rows = plan
        .get(runtime_schema::document::ROWS)
        .and_then(Value::as_array)
        .ok_or_else(|| message("Alert plan document is missing rows."))?;

    let mut apply_result = ReviewApplyResult::new("apply");
    for row in rows {
        let row = row_object(row, "Alert plan row")?;
        let action = row
            .get(runtime_schema::row::ACTION)
            .and_then(Value::as_str)
            .unwrap_or("");
        if !matches!(action, "create" | "update" | "delete") {
            continue;
        }
        let kind = row
            .get(runtime_schema::row::KIND)
            .and_then(Value::as_str)
            .ok_or_else(|| message("Alert plan row is missing kind."))?;
        let identity = row
            .get(runtime_schema::row::IDENTITY)
            .and_then(Value::as_str)
            .unwrap_or_default();
        let response = match action {
            "create" => {
                let desired = payload_object_from_row(row, runtime_schema::row::DESIRED)?;
                apply_create_with_request(&mut request_json, kind, desired)?
            }
            "update" => {
                let desired = payload_object_from_row(row, runtime_schema::row::DESIRED)?;
                apply_update_with_request(&mut request_json, kind, identity, desired)?
            }
            "delete" => {
                apply_delete_with_request(&mut request_json, kind, identity, allow_policy_reset)?
            }
            _ => unreachable!(),
        };
        apply_result.push_result(review_apply_result_entry(kind, identity, action, response));
    }

    Ok(apply_result.into_value_with_fields([
        (
            "kind",
            Value::String("grafana-util-alert-apply-result".to_string()),
        ),
        ("allowPolicyReset", Value::Bool(allow_policy_reset)),
    ]))
}

pub fn init_alert_runtime_layout(root: &Path) -> Result<Value> {
    let created = init_alert_managed_dir(root)?
        .into_iter()
        .map(|path| Value::String(path_string(&path)))
        .collect::<Vec<Value>>();
    Ok(json!({
        "kind": "grafana-util-alert-init",
        "root": path_string(root),
        "created": created,
    }))
}

pub fn write_new_rule_scaffold(path: &Path, name: &str, overwrite: bool) -> Result<Value> {
    let document = build_new_rule_scaffold_document(name);
    write_alert_resource_file(path, &document, overwrite)?;
    Ok(document)
}

pub fn write_new_contact_point_scaffold(path: &Path, name: &str, overwrite: bool) -> Result<Value> {
    let document = build_new_contact_point_scaffold_document(name);
    write_alert_resource_file(path, &document, overwrite)?;
    Ok(document)
}

#[allow(dead_code)]
pub fn write_contact_point_scaffold(
    path: &Path,
    name: &str,
    channel_type: &str,
    overwrite: bool,
) -> Result<Value> {
    let document = build_contact_point_scaffold_document(name, channel_type);
    write_alert_resource_file(path, &document, overwrite)?;
    Ok(document)
}

pub fn write_new_template_scaffold(path: &Path, name: &str, overwrite: bool) -> Result<Value> {
    let document = build_new_template_scaffold_document(name);
    write_alert_resource_file(path, &document, overwrite)?;
    Ok(document)
}

#[allow(dead_code)]
pub fn build_managed_policy_edit_preview_document(
    current_policy_document: &Value,
    route_name: &str,
    desired_route_document: Option<&Value>,
) -> Result<Value> {
    let current_policy = value_as_object(current_policy_document, "Current notification policies")?;
    let desired_route = match desired_route_document {
        Some(value) => Some(value_as_object(value, "Desired managed route")?),
        None => None,
    };
    Ok(json!({
        "kind": "grafana-util-alert-managed-policy-preview",
        "schemaVersion": ALERT_MANAGED_POLICY_PREVIEW_SCHEMA_VERSION,
        "toolVersion": tool_version(),
        "reviewRequired": true,
        "reviewed": false,
        "routeName": route_name,
        "preview": build_managed_policy_route_preview(current_policy, route_name, desired_route)?,
    }))
}

#[allow(dead_code)]
pub fn apply_managed_policy_subtree_edit_document(
    current_policy_document: &Value,
    route_name: &str,
    desired_route_document: Option<&Value>,
) -> Result<Value> {
    let current_policy = value_as_object(current_policy_document, "Current notification policies")?;
    let (next_policy, action) = match desired_route_document {
        Some(value) => upsert_managed_policy_subtree(
            current_policy,
            route_name,
            value_as_object(value, "Desired managed route")?,
        )?,
        None => remove_managed_policy_subtree(current_policy, route_name)?,
    };
    Ok(json!({
        "kind": POLICIES_KIND,
        "action": action,
        "spec": Value::Object(next_policy),
    }))
}
