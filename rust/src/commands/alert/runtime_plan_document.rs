//! Alert runtime plan, delete-preview, import dry-run, and diff document construction.

use crate::common::{build_shared_diff_document, tool_version, SharedDiffSummary};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::path::Path;

pub const ALERT_PLAN_KIND: &str = "grafana-util-alert-plan";
pub const ALERT_PLAN_SCHEMA_VERSION: i64 = 1;
pub const ALERT_DELETE_PREVIEW_KIND: &str = "grafana-util-alert-delete-preview";
pub const ALERT_DELETE_PREVIEW_SCHEMA_VERSION: i64 = 1;
pub const ALERT_IMPORT_DRY_RUN_KIND: &str = "grafana-util-alert-import-dry-run";
pub const ALERT_IMPORT_DRY_RUN_SCHEMA_VERSION: i64 = 1;

pub(crate) mod runtime_schema {
    pub(crate) mod document {
        pub(crate) const KIND: &str = "kind";
        pub(crate) const SCHEMA_VERSION: &str = "schemaVersion";
        pub(crate) const TOOL_VERSION: &str = "toolVersion";
        pub(crate) const REVIEW_REQUIRED: &str = "reviewRequired";
        pub(crate) const REVIEWED: &str = "reviewed";
        pub(crate) const ALLOW_PRUNE: &str = "allowPrune";
        pub(crate) const ALLOW_POLICY_RESET: &str = "allowPolicyReset";
        pub(crate) const SUMMARY: &str = "summary";
        pub(crate) const ROWS: &str = "rows";
    }

    pub(crate) mod row {
        pub(crate) const DOMAIN: &str = "domain";
        pub(crate) const RESOURCE_KIND: &str = "resourceKind";
        pub(crate) const KIND: &str = "kind";
        pub(crate) const IDENTITY: &str = "identity";
        pub(crate) const ACTION_ID: &str = "actionId";
        pub(crate) const ACTION: &str = "action";
        pub(crate) const STATUS: &str = "status";
        pub(crate) const REASON: &str = "reason";
        pub(crate) const BLOCKED_REASON: &str = "blockedReason";
        pub(crate) const REVIEW_HINTS: &str = "reviewHints";
        pub(crate) const CHANGED_FIELDS: &str = "changedFields";
        pub(crate) const CHANGES: &str = "changes";
        pub(crate) const PATH: &str = "path";
        pub(crate) const DESIRED: &str = "desired";
        pub(crate) const LIVE: &str = "live";
    }

    pub(crate) mod summary {
        pub(crate) const PROCESSED: &str = "processed";
        pub(crate) const CREATE: &str = "create";
        pub(crate) const UPDATE: &str = "update";
        pub(crate) const NOOP: &str = "noop";
        pub(crate) const DELETE: &str = "delete";
        pub(crate) const BLOCKED: &str = "blocked";
        pub(crate) const WARNING: &str = "warning";
        pub(crate) const WOULD_CREATE: &str = "wouldCreate";
        pub(crate) const WOULD_UPDATE: &str = "wouldUpdate";
        pub(crate) const WOULD_FAIL_EXISTING: &str = "wouldFailExisting";
    }
}

fn count_rows_by_action(rows: &[Value], action: &str) -> usize {
    rows.iter()
        .filter(|row| row.get(runtime_schema::row::ACTION).and_then(Value::as_str) == Some(action))
        .count()
}

fn plan_summary(rows: &[Value]) -> Value {
    let warning = rows
        .iter()
        .filter(|row| {
            row.get(runtime_schema::row::STATUS).and_then(Value::as_str) == Some("warning")
        })
        .count();
    json!({
        (runtime_schema::summary::PROCESSED): rows.len(),
        (runtime_schema::summary::CREATE): count_rows_by_action(rows, "create"),
        (runtime_schema::summary::UPDATE): count_rows_by_action(rows, "update"),
        (runtime_schema::summary::NOOP): count_rows_by_action(rows, "noop"),
        (runtime_schema::summary::DELETE): count_rows_by_action(rows, "delete"),
        (runtime_schema::summary::BLOCKED): count_rows_by_action(rows, "blocked"),
        (runtime_schema::summary::WARNING): warning,
    })
}

fn plan_action_id(kind: &str, identity: &str, action: &str) -> String {
    format!("{kind}::{identity}::{action}")
}

fn field_change(field: &str, before: Option<&Value>, after: Option<&Value>) -> Value {
    json!({
        "field": field,
        "before": before.cloned().unwrap_or(Value::Null),
        "after": after.cloned().unwrap_or(Value::Null),
    })
}

fn compare_field_changes(
    desired: Option<&Map<String, Value>>,
    live: Option<&Map<String, Value>>,
) -> (Vec<String>, Vec<Value>) {
    let mut fields = BTreeSet::new();
    if let Some(object) = desired {
        fields.extend(object.keys().cloned());
    }
    if let Some(object) = live {
        fields.extend(object.keys().cloned());
    }

    let mut changed_fields = Vec::new();
    let mut changes = Vec::new();
    for field in fields {
        let desired_value = desired.and_then(|object| object.get(&field));
        let live_value = live.and_then(|object| object.get(&field));
        if desired_value != live_value {
            changed_fields.push(field.clone());
            changes.push(field_change(&field, live_value, desired_value));
        }
    }
    (changed_fields, changes)
}

fn review_hint(code: &str, field: &str, before: Option<&str>, after: Option<&str>) -> Value {
    json!({
        "code": code,
        "field": field,
        "before": before.unwrap_or(""),
        "after": after.unwrap_or(""),
    })
}

pub(crate) fn build_rule_review_hints(payload: &Map<String, Value>) -> Vec<Value> {
    let mut hints = Vec::new();
    let annotations = payload.get("annotations").and_then(Value::as_object);
    let dashboard_uid = annotations
        .and_then(|value| value.get("__dashboardUid__"))
        .and_then(Value::as_str);
    if let Some(dashboard_uid) = dashboard_uid {
        hints.push(review_hint(
            "linked-dashboard-reference",
            "annotations.__dashboardUid__",
            Some(dashboard_uid),
            Some(dashboard_uid),
        ));
    }

    let panel_id =
        annotations
            .and_then(|value| value.get("__panelId__"))
            .map(|value| match value {
                Value::String(text) => text.clone(),
                other => other.to_string(),
            });
    if let Some(panel_id) = panel_id {
        hints.push(review_hint(
            "linked-panel-reference",
            "annotations.__panelId__",
            Some(&panel_id),
            Some(&panel_id),
        ));
    }

    hints
}

fn plan_status(action: &str, blocked_reason: Option<&str>, review_hints: &[Value]) -> &'static str {
    if blocked_reason.is_some() {
        "blocked"
    } else if !review_hints.is_empty() {
        "warning"
    } else if action == "noop" {
        "same"
    } else {
        "ready"
    }
}

pub(crate) struct PlanRowInput<'a> {
    pub(crate) kind: &'a str,
    pub(crate) identity: &'a str,
    pub(crate) path: Option<&'a Path>,
    pub(crate) action: &'a str,
    pub(crate) reason: &'a str,
    pub(crate) blocked_reason: Option<&'a str>,
    pub(crate) desired: Option<&'a Map<String, Value>>,
    pub(crate) live: Option<&'a Map<String, Value>>,
    pub(crate) review_hints: Vec<Value>,
}

pub(crate) fn build_plan_row(input: PlanRowInput<'_>) -> Value {
    let (changed_fields, changes) = if input.action == "noop" {
        (Vec::new(), Vec::new())
    } else {
        compare_field_changes(input.desired, input.live)
    };
    let status = plan_status(input.action, input.blocked_reason, &input.review_hints);
    json!({
        (runtime_schema::row::DOMAIN): "alert",
        (runtime_schema::row::RESOURCE_KIND): input.kind,
        (runtime_schema::row::KIND): input.kind,
        (runtime_schema::row::IDENTITY): input.identity,
        (runtime_schema::row::ACTION_ID): plan_action_id(input.kind, input.identity, input.action),
        (runtime_schema::row::ACTION): input.action,
        (runtime_schema::row::STATUS): status,
        (runtime_schema::row::REASON): input.reason,
        (runtime_schema::row::BLOCKED_REASON): input.blocked_reason,
        (runtime_schema::row::REVIEW_HINTS): input.review_hints,
        (runtime_schema::row::CHANGED_FIELDS): changed_fields,
        (runtime_schema::row::CHANGES): changes,
        (runtime_schema::row::PATH): input.path.map(|path| path.to_string_lossy().to_string()),
        (runtime_schema::row::DESIRED): input.desired.map(|value| Value::Object(value.clone())).unwrap_or(Value::Null),
        (runtime_schema::row::LIVE): input.live.map(|value| Value::Object(value.clone())).unwrap_or(Value::Null),
    })
}

pub fn build_alert_plan_document(rows: &[Value], allow_prune: bool) -> Value {
    json!({
        (runtime_schema::document::KIND): ALERT_PLAN_KIND,
        (runtime_schema::document::SCHEMA_VERSION): ALERT_PLAN_SCHEMA_VERSION,
        (runtime_schema::document::TOOL_VERSION): tool_version(),
        (runtime_schema::document::REVIEW_REQUIRED): true,
        (runtime_schema::document::REVIEWED): false,
        (runtime_schema::document::ALLOW_PRUNE): allow_prune,
        (runtime_schema::document::SUMMARY): plan_summary(rows),
        (runtime_schema::document::ROWS): rows,
    })
}

pub fn build_alert_delete_preview_document(rows: &[Value], allow_policy_reset: bool) -> Value {
    let summary = json!({
        (runtime_schema::summary::PROCESSED): rows.len(),
        (runtime_schema::summary::DELETE): count_rows_by_action(rows, "delete"),
        (runtime_schema::summary::BLOCKED): count_rows_by_action(rows, "blocked"),
    });
    json!({
        (runtime_schema::document::KIND): ALERT_DELETE_PREVIEW_KIND,
        (runtime_schema::document::SCHEMA_VERSION): ALERT_DELETE_PREVIEW_SCHEMA_VERSION,
        (runtime_schema::document::TOOL_VERSION): tool_version(),
        (runtime_schema::document::REVIEW_REQUIRED): true,
        (runtime_schema::document::REVIEWED): false,
        (runtime_schema::document::ALLOW_POLICY_RESET): allow_policy_reset,
        (runtime_schema::document::SUMMARY): summary,
        (runtime_schema::document::ROWS): rows,
    })
}

pub fn build_alert_import_dry_run_document(rows: &[Value]) -> Value {
    let processed = rows.len();
    let would_create = rows
        .iter()
        .filter(|row| {
            row.get(runtime_schema::row::ACTION).and_then(Value::as_str) == Some("would-create")
        })
        .count();
    let would_update = rows
        .iter()
        .filter(|row| {
            row.get(runtime_schema::row::ACTION).and_then(Value::as_str) == Some("would-update")
        })
        .count();
    let would_fail_existing = rows
        .iter()
        .filter(|row| {
            row.get(runtime_schema::row::ACTION).and_then(Value::as_str)
                == Some("would-fail-existing")
        })
        .count();

    let summary = json!({
        (runtime_schema::summary::PROCESSED): processed,
        (runtime_schema::summary::WOULD_CREATE): would_create,
        (runtime_schema::summary::WOULD_UPDATE): would_update,
        (runtime_schema::summary::WOULD_FAIL_EXISTING): would_fail_existing,
    });
    json!({
        (runtime_schema::document::KIND): ALERT_IMPORT_DRY_RUN_KIND,
        (runtime_schema::document::SCHEMA_VERSION): ALERT_IMPORT_DRY_RUN_SCHEMA_VERSION,
        (runtime_schema::document::TOOL_VERSION): tool_version(),
        (runtime_schema::document::REVIEW_REQUIRED): true,
        (runtime_schema::document::REVIEWED): false,
        (runtime_schema::document::SUMMARY): summary,
        (runtime_schema::document::ROWS): rows,
    })
}

pub fn build_alert_diff_document(rows: &[Value]) -> Value {
    let checked = rows.len();
    let same = rows
        .iter()
        .filter(|row| {
            row.get("status")
                .and_then(Value::as_str)
                .or_else(|| row.get("action").and_then(Value::as_str))
                == Some("same")
        })
        .count();
    let different = rows
        .iter()
        .filter(|row| {
            row.get("status")
                .and_then(Value::as_str)
                .or_else(|| row.get("action").and_then(Value::as_str))
                == Some("different")
        })
        .count();
    let missing_remote = rows
        .iter()
        .filter(|row| {
            row.get("status")
                .and_then(Value::as_str)
                .or_else(|| row.get("action").and_then(Value::as_str))
                == Some("missing-remote")
        })
        .count();

    let mut document = match build_shared_diff_document(
        "grafana-util-alert-diff",
        1,
        SharedDiffSummary {
            checked,
            same,
            different,
            missing_remote,
            extra_remote: 0,
            ambiguous: 0,
        },
        rows,
    ) {
        Value::Object(map) => map,
        _ => unreachable!("shared diff document must be an object"),
    };
    document.insert("reviewRequired".to_string(), Value::Bool(true));
    document.insert("reviewed".to_string(), Value::Bool(false));
    Value::Object(document)
}
