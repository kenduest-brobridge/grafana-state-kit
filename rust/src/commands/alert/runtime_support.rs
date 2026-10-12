//! Build alert compare/import/delete plan documents from live Grafana payloads.
//!
//! Responsibilities:
//! - Gather and normalize alert resources through shared request helpers.
//! - Produce plan and delete-preview documents used by diff/import execution flows.
//! - Preserve request semantics so CLI/runtime callers receive a stable sync-ready
//!   shape across execution paths.

use crate::common::{message, tool_version, value_as_object, GrafanaCliError, Result};
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
const CONTROLLED_RULE_UPDATE_FIELDS: &[&str] =
    &["condition", "data", "for", "noDataState", "execErrState"];

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
    apply_create_with_request, apply_delete_with_request, apply_prepared_rule_update_with_request,
    apply_update_with_request, fetch_live_compare_document_with_request,
    request_array_with_request, request_live_resources_by_kind_with_request,
    request_optional_object_with_request,
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
        let live_payload = live_compare
            .as_ref()
            .and_then(|value| value.get("spec"))
            .and_then(Value::as_object);
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

fn row_changed_fields(row: &Map<String, Value>) -> Result<Vec<String>> {
    let fields = row
        .get(runtime_schema::row::CHANGED_FIELDS)
        .and_then(Value::as_array)
        .ok_or_else(|| message("Alert plan row is missing changedFields."))?;
    fields
        .iter()
        .map(|field| {
            field
                .as_str()
                .filter(|value| !value.trim().is_empty())
                .map(ToOwned::to_owned)
                .ok_or_else(|| message("Alert plan row changedFields must contain strings."))
        })
        .collect()
}

fn require_matching_rule_identity(
    identity: &str,
    desired: &Map<String, Value>,
    live: &Map<String, Value>,
) -> Result<()> {
    let desired_uid = desired
        .get("uid")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let live_uid = live.get("uid").and_then(Value::as_str).unwrap_or_default();
    if identity.is_empty() || desired_uid != identity || live_uid != identity {
        return Err(message(format!(
            "controlled alert live apply requires matching rule uid identity for {identity:?}."
        )));
    }
    Ok(())
}

struct ControlledRuleUpdate {
    identity: String,
    payload: Map<String, Value>,
    baseline: Value,
}

fn prepare_controlled_alert_live_apply_row(
    row: &Map<String, Value>,
) -> Result<ControlledRuleUpdate> {
    let action = row
        .get(runtime_schema::row::ACTION)
        .and_then(Value::as_str)
        .unwrap_or_default();
    let kind = row
        .get(runtime_schema::row::KIND)
        .and_then(Value::as_str)
        .ok_or_else(|| message("Alert plan row is missing kind."))?;
    let identity = row
        .get(runtime_schema::row::IDENTITY)
        .and_then(Value::as_str)
        .unwrap_or_default();
    if action != "update" || kind != RULE_KIND {
        return Err(message(format!(
            "controlled alert live apply only supports narrow alert-rule updates; refusing {action} for {kind} {identity}."
        )));
    }
    let status = row
        .get(runtime_schema::row::STATUS)
        .and_then(Value::as_str)
        .unwrap_or_default();
    if status != "ready" {
        return Err(message(format!(
            "controlled alert live apply requires ready plan rows; refusing {kind} {identity} with status {status:?}."
        )));
    }
    let desired =
        build_rule_import_payload(payload_object_from_row(row, runtime_schema::row::DESIRED)?)?;
    let live = build_rule_import_payload(payload_object_from_row(row, runtime_schema::row::LIVE)?)?;
    require_matching_rule_identity(identity, &desired, &live)?;

    let changed_fields = row_changed_fields(row)?;
    if changed_fields.is_empty() {
        return Err(message(format!(
            "controlled alert live apply requires explicit changedFields for {kind} {identity}."
        )));
    }
    let unsupported = changed_fields
        .iter()
        .filter(|field| !CONTROLLED_RULE_UPDATE_FIELDS.contains(&field.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unsupported.is_empty() {
        return Err(message(format!(
            "controlled alert live apply unsupported rule fields for {identity}: {}.",
            unsupported.join(", ")
        )));
    }
    let actual_changed_fields = desired
        .keys()
        .chain(live.keys())
        .filter(|field| desired.get(*field) != live.get(*field))
        .cloned()
        .collect::<BTreeSet<_>>();
    let unsupported = actual_changed_fields
        .iter()
        .filter(|field| !CONTROLLED_RULE_UPDATE_FIELDS.contains(&field.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unsupported.is_empty() {
        return Err(message(format!(
            "controlled alert live apply unsupported rule fields for {identity}: {}.",
            unsupported.join(", ")
        )));
    }
    if actual_changed_fields != changed_fields.into_iter().collect::<BTreeSet<_>>() {
        return Err(message(format!(
            "controlled alert live apply changedFields does not match payload changes for {identity}."
        )));
    }
    Ok(ControlledRuleUpdate {
        identity: identity.to_string(),
        payload: desired,
        baseline: build_compare_document(RULE_KIND, &normalize_compare_payload(RULE_KIND, &live)),
    })
}

#[derive(Debug)]
pub(super) struct AlertPlanApplyOutcome {
    pub(super) document: Value,
    failure: Option<GrafanaCliError>,
}

impl AlertPlanApplyOutcome {
    pub(super) fn into_result(self) -> Result<Value> {
        match self.failure {
            Some(error) => Err(error),
            None => Ok(self.document),
        }
    }
}

pub(super) fn execute_alert_plan_report_with_request<F>(
    mut request_json: F,
    plan_document: &Value,
    allow_policy_reset: bool,
) -> Result<AlertPlanApplyOutcome>
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
    if plan
        .get(runtime_schema::document::SCHEMA_VERSION)
        .and_then(Value::as_i64)
        != Some(ALERT_PLAN_SCHEMA_VERSION)
    {
        return Err(message(
            "Alert plan document schemaVersion is not supported.",
        ));
    }
    let rows = plan
        .get(runtime_schema::document::ROWS)
        .and_then(Value::as_array)
        .ok_or_else(|| message("Alert plan document is missing rows."))?;

    let mut prepared = Vec::new();
    let mut identities = BTreeSet::new();
    for row in rows {
        let row = row_object(row, "Alert plan row")?;
        let action = row
            .get(runtime_schema::row::ACTION)
            .and_then(Value::as_str)
            .unwrap_or("");
        match action {
            "noop" | "blocked" => continue,
            "create" | "update" | "delete" => {}
            _ => {
                return Err(message(format!(
                    "Alert plan row action {action:?} is not supported."
                )))
            }
        }
        let update = prepare_controlled_alert_live_apply_row(row)?;
        if !identities.insert(update.identity.clone()) {
            return Err(message(format!(
                "Duplicate alert plan rule identity {:?}.",
                update.identity
            )));
        }
        prepared.push(update);
    }

    for update in &prepared {
        let fresh = fetch_live_compare_document_with_request(
            &mut request_json,
            RULE_KIND,
            &update.payload,
        )
        .map_err(|error| {
            let status = error
                .status_code()
                .map(|code| format!(" HTTP {code}"))
                .unwrap_or_default();
            message(format!(
                "Alert apply could not verify live baseline for {} ({}{status}). No updates were written.",
                update.identity,
                error.kind()
            ))
        })?;
        if fresh.as_ref() != Some(&update.baseline) {
            return Err(message(format!(
                "Alert rule {} baseline changed or is missing. No updates were written. Rebuild and review the plan before applying.",
                update.identity
            )));
        }
    }

    let mut apply_result = ReviewApplyResult::new("apply");
    let mut unfinished_results = Vec::new();
    let mut failure = None;
    for (index, update) in prepared.iter().enumerate() {
        match apply_prepared_rule_update_with_request(
            &mut request_json,
            &update.identity,
            &update.payload,
        ) {
            Ok(response) => {
                let mut result =
                    review_apply_result_entry(RULE_KIND, &update.identity, "update", response);
                result["status"] = Value::String("completed".to_string());
                apply_result.push_result(result);
            }
            Err(error) => {
                let mut failed =
                    review_apply_result_entry(RULE_KIND, &update.identity, "update", Value::Null);
                failed["status"] = Value::String("failed".to_string());
                failed["error"] = json!({
                    "kind": error.kind(),
                    "statusCode": error.status_code(),
                    "message": "Update request failed. Verify current live state before retrying.",
                });
                unfinished_results.push(failed);
                for remaining in prepared.iter().skip(index + 1) {
                    let mut result = review_apply_result_entry(
                        RULE_KIND,
                        &remaining.identity,
                        "update",
                        Value::Null,
                    );
                    result["status"] = Value::String("not-attempted".to_string());
                    unfinished_results.push(result);
                }
                let completed = prepared[..index]
                    .iter()
                    .map(|item| item.identity.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                let not_attempted = prepared[index + 1..]
                    .iter()
                    .map(|item| item.identity.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                failure = Some(message(format!(
                    "Alert apply stopped after an update request failed. Completed: [{completed}]; failed: [{}]; not-attempted: [{not_attempted}]. Verify current live state before retrying.",
                    update.identity
                )));
                break;
            }
        }
    }

    let status = if failure.is_none() {
        "completed"
    } else if apply_result.results.is_empty() {
        "failed"
    } else {
        "partial"
    };
    let mut document = apply_result.into_value_with_fields([
        (
            "kind",
            Value::String("grafana-util-alert-apply-result".to_string()),
        ),
        ("allowPolicyReset", Value::Bool(allow_policy_reset)),
        ("status", Value::String(status.to_string())),
        ("failedCount", json!(usize::from(failure.is_some()))),
        (
            "notAttemptedCount",
            json!(unfinished_results.len().saturating_sub(1)),
        ),
    ]);
    document["results"]
        .as_array_mut()
        .expect("apply result contains results")
        .extend(unfinished_results);
    Ok(AlertPlanApplyOutcome { document, failure })
}

pub fn execute_alert_plan_with_request<F>(
    request_json: F,
    plan_document: &Value,
    allow_policy_reset: bool,
) -> Result<Value>
where
    F: FnMut(Method, &str, &[(String, String)], Option<&Value>) -> Result<Option<Value>>,
{
    execute_alert_plan_report_with_request(request_json, plan_document, allow_policy_reset)?
        .into_result()
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
